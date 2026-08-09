#![allow(clippy::expect_used, clippy::unwrap_used)]

//! P4-T04 daemon-private local Skill import and binding regressions.

use cognitive_domain::ObjectId;
use cognitive_kernel::ports::{
    SkillBindingRevocationRow, SkillBindingRow, SkillPackageRow, SkillRevisionRow,
    SkillRevisionSupersedeRequest, SkillStore, StorePortError,
};
use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore, prepare_personal_databases};

fn object_id(sequence: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
}

fn fresh_store() -> (tempfile::TempDir, SqliteAuthorityStore) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let layout = PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    );
    prepare_personal_databases(&layout).unwrap();
    (
        directory,
        SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap(),
    )
}

fn package_and_revision(scope: &str) -> (SkillPackageRow, SkillRevisionRow) {
    let package = SkillPackageRow {
        package_id: object_id(1),
        workspace_scope: scope.to_owned(),
        local_source_path: "skills/release-notes".to_owned(),
        provenance_ref: "file://workspace/skills/release-notes".to_owned(),
        manifest_digest: "sha256:manifest".to_owned(),
        canonical_json: "{}".to_owned(),
    };
    let revision = SkillRevisionRow {
        revision_id: object_id(2),
        package_id: package.package_id.clone(),
        content_digest: "sha256:revision".to_owned(),
        compatibility: "compatible".to_owned(),
        canonical_json: "{}".to_owned(),
    };
    (package, revision)
}

fn binding(revision_id: ObjectId, scope: &str, status: &str) -> SkillBindingRow {
    SkillBindingRow {
        binding_id: object_id(3),
        revision_id,
        workspace_scope: scope.to_owned(),
        target_kind: "task".to_owned(),
        target_ref: "task://tenant-a/42".to_owned(),
        status: status.to_owned(),
        canonical_json: "{}".to_owned(),
    }
}

#[test]
fn compatible_local_revision_binds_only_inside_its_workspace() {
    let (_directory, store) = fresh_store();
    let (package, revision) = package_and_revision("workspace://tenant-a/project");
    store.append_skill_import(&package, &revision).unwrap();
    let active_binding = binding(
        revision.revision_id.clone(),
        &package.workspace_scope,
        "active",
    );
    store.append_skill_binding(&active_binding).unwrap();
    assert_eq!(
        store
            .load_skill_binding(&active_binding.binding_id)
            .unwrap(),
        Some(active_binding)
    );

    let active_binding = binding(
        revision.revision_id.clone(),
        &package.workspace_scope,
        "active",
    );
    let revocation = SkillBindingRevocationRow {
        revocation_id: object_id(8),
        binding_id: active_binding.binding_id.clone(),
        reason: "workspace owner revoked task eligibility".to_owned(),
        canonical_json: "{}".to_owned(),
    };
    store.append_skill_binding_revocation(&revocation).unwrap();
    assert_eq!(
        store
            .load_active_skill_binding(&active_binding.binding_id)
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .load_skill_binding(&active_binding.binding_id)
            .unwrap(),
        Some(active_binding)
    );
    assert!(matches!(
        store.append_skill_binding_revocation(&revocation),
        Err(StorePortError::Conflict { .. })
    ));

    let cross_workspace_binding = binding(
        revision.revision_id,
        "workspace://tenant-b/project",
        "active",
    );
    assert!(matches!(
        store.append_skill_binding(&cross_workspace_binding),
        Err(StorePortError::Conflict { .. })
    ));
}

#[test]
fn unsafe_import_and_incompatible_or_revoked_bindings_fail_closed() {
    let (_directory, store) = fresh_store();
    let (mut unsafe_package, unsafe_revision) =
        package_and_revision("workspace://tenant-a/project");
    unsafe_package.local_source_path = "../outside/SKILL.md".to_owned();
    assert!(matches!(
        store.append_skill_import(&unsafe_package, &unsafe_revision),
        Err(StorePortError::Conflict { .. })
    ));

    let (package, mut incompatible_revision) = package_and_revision("workspace://tenant-a/project");
    incompatible_revision.revision_id = object_id(4);
    incompatible_revision.content_digest = "sha256:incompatible".to_owned();
    incompatible_revision.compatibility = "incompatible".to_owned();
    store
        .append_skill_import(&package, &incompatible_revision)
        .unwrap();
    assert!(matches!(
        store.append_skill_binding(&binding(
            incompatible_revision.revision_id,
            &package.workspace_scope,
            "active"
        )),
        Err(StorePortError::Conflict { .. })
    ));

    let (compatible_package, compatible_revision) =
        package_and_revision("workspace://tenant-a/other");
    let mut compatible_package = compatible_package;
    let mut compatible_revision = compatible_revision;
    compatible_package.package_id = object_id(6);
    compatible_revision.revision_id = object_id(7);
    compatible_revision.package_id = compatible_package.package_id.clone();
    compatible_revision.content_digest = "sha256:compatible-other".to_owned();
    store
        .append_skill_import(&compatible_package, &compatible_revision)
        .unwrap();
    let mut revoked_binding = binding(
        compatible_revision.revision_id,
        &compatible_package.workspace_scope,
        "revoked",
    );
    revoked_binding.binding_id = object_id(5);
    store.append_skill_binding(&revoked_binding).unwrap();
    assert_eq!(
        store
            .load_skill_binding(&revoked_binding.binding_id)
            .unwrap(),
        Some(revoked_binding)
    );
}

#[test]
fn revision_supersede_preserves_exact_pins_and_rejects_competing_lineage() {
    let (_directory, store) = fresh_store();
    let (package, revision) = package_and_revision("workspace://tenant-a/project");
    store.append_skill_import(&package, &revision).unwrap();
    let pinned_binding = binding(
        revision.revision_id.clone(),
        &package.workspace_scope,
        "active",
    );
    store.append_skill_binding(&pinned_binding).unwrap();

    let replacement = SkillRevisionRow {
        revision_id: object_id(9),
        package_id: package.package_id.clone(),
        content_digest: "sha256:replacement".to_owned(),
        compatibility: "compatible".to_owned(),
        canonical_json: "{}".to_owned(),
    };
    let supersede = SkillRevisionSupersedeRequest {
        previous_revision_id: revision.revision_id,
        replacement,
        canonical_json: "{}".to_owned(),
    };
    store.append_skill_revision_supersede(&supersede).unwrap();
    assert_eq!(
        store
            .load_active_skill_binding(&pinned_binding.binding_id)
            .unwrap(),
        Some(pinned_binding)
    );
    assert!(matches!(
        store.append_skill_revision_supersede(&supersede),
        Err(StorePortError::Conflict { .. })
    ));
}
