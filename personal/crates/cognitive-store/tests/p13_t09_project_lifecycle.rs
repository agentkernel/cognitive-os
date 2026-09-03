#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T09 failure-first: copy lands inactive and never inherits grant /
//! seating / runtime; archive stops triggers; delete is preview + second
//! confirmation (no physical drop); export excludes secrets; restore points
//! are not backups.

use cognitive_store::project_aggregate::{
    LifecycleArchiveSpec, LifecycleCopySpec, LifecycleDeleteConfirmSpec,
    LifecycleDeletePreviewSpec, LifecycleExportSpec, LifecycleRestoreSpec, ProjectLifecycleStore,
};
use cognitive_store::{
    ConfirmCaller, EmployeeStore, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    RosterProposal, StageSpec, prepare_personal_databases,
};
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    ProjectAggregateStore,
    EmployeeStore,
    ProjectLifecycleStore,
) {
    let temporary = TempDir::new().expect("temp");
    let root = temporary.path();
    let layout = PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    );
    prepare_personal_databases(&layout).expect("prepare");
    let path = layout.authority_database_path();
    let projects = ProjectAggregateStore::open_path(&path).expect("projects");
    let employees = EmployeeStore::open_path(&path).expect("employees");
    let lifecycle = ProjectLifecycleStore::open_path(&path).expect("lifecycle");
    (temporary, projects, employees, lifecycle)
}

fn stage(id: &str, title: &str, slot: &str) -> StageSpec {
    StageSpec {
        stage_id: id.to_owned(),
        title: title.to_owned(),
        objective: format!("{title} objective"),
        output_contract_digest: ProjectAggregateStore::digest_hex(format!("out-{id}").as_bytes()),
        acceptance_spec_ref: Some(format!("cas:spec-{id}")),
        cadence_json: Some(r#"{"kind":"manual"}"#.to_owned()),
        responsible_slot: slot.to_owned(),
        blocking_gap: None,
    }
}

fn activate(projects: &ProjectAggregateStore) -> String {
    let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
    projects
        .put_draft_charter(&draft_id, b"charter-body-v1", 11)
        .expect("charter");
    let (preview_id, preview_digest) = projects
        .request_preview("activation", &draft_id, b"activation-preview", 12)
        .expect("preview");
    projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .expect("G1")
        .new_ref
}

fn plan_two_slots(projects: &ProjectAggregateStore, project_id: &str) -> String {
    projects
        .apply_plan_revision(
            project_id,
            project_id,
            &[
                stage("s1", "Manage", "manager"),
                stage("s2", "Research", "researcher"),
            ],
            20,
        )
        .expect("plan")
}

fn seat_manager(projects: &ProjectAggregateStore, employees: &EmployeeStore, project_id: &str) {
    let plan_id = plan_two_slots(projects, project_id);
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            project_id,
            &plan_id,
            &[
                RosterProposal {
                    slot: "manager".to_owned(),
                    specialization: "project-manager".to_owned(),
                    prompt: "coordinate".to_owned(),
                    tools_declared: vec!["workspace-write".to_owned()],
                },
                RosterProposal {
                    slot: "researcher".to_owned(),
                    specialization: "member".to_owned(),
                    prompt: "research".to_owned(),
                    tools_declared: vec!["workspace-write".to_owned()],
                },
            ],
            21,
        )
        .expect("roster");
    employees
        .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
        .expect("seating");
    employees
        .confirm_seating(
            ConfirmCaller::OwnerManagement,
            &ids[0],
            Some("flash"),
            true,
            31,
        )
        .expect("seat");
}

fn copy_spec<'a>(source: &'a str, now_ms: i64) -> LifecycleCopySpec<'a> {
    LifecycleCopySpec {
        caller: ConfirmCaller::OwnerManagement,
        source_project_id: source,
        inherit_grants: false,
        inherit_seats: false,
        inherit_runtime: false,
        now_ms,
    }
}

#[test]
fn p13_t09_copy_refuses_inherit_flags_but_copies_seated_source_as_inactive() {
    let (_tmp, projects, employees, lifecycle) = stores();
    let project_id = activate(&projects);
    seat_manager(&projects, &employees, &project_id);

    let inherit = lifecycle
        .copy_project(LifecycleCopySpec {
            inherit_seats: true,
            ..copy_spec(&project_id, 90)
        })
        .expect_err("inherit seats");
    assert!(
        matches!(inherit, ProjectAggregateError::Rejected { detail } if detail.contains("inherit")),
        "{inherit}"
    );
    assert_eq!(projects.list_projects(16).expect("list").len(), 1);

    let copy_id = lifecycle
        .copy_project(copy_spec(&project_id, 91))
        .expect("seated source may copy");
    let copy = projects.get_project(&copy_id).expect("get").expect("row");
    assert_eq!(copy.state, "inactive");
    assert!(copy.accepted_at.is_none());
    assert!(copy.current_plan_revision_id.is_none());
    assert_eq!(projects.list_projects(16).expect("list").len(), 2);

    let view = lifecycle.lifecycle_view(&copy_id).expect("view");
    assert!(!view.tombstoned);
    assert_eq!(view.is_backup, false);
    assert!(view.data_dir.is_some(), "every Project gets data/");
}

#[test]
fn p13_t09_archive_delete_export_restore_negatives_and_round_trip() {
    let (_tmp, projects, _employees, lifecycle) = stores();
    let project_id = activate(&projects);

    let skip = lifecycle
        .archive_project(LifecycleArchiveSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &project_id,
            skip_stop_triggers: true,
            now_ms: 94,
        })
        .expect_err("skip stop");
    assert!(
        matches!(skip, ProjectAggregateError::Rejected { detail } if detail.contains("skip_stop_triggers")),
        "{skip}"
    );

    let live = lifecycle
        .preview_delete(LifecycleDeletePreviewSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &project_id,
            now_ms: 95,
        })
        .expect_err("delete while live");
    assert!(
        format!("{live}").contains("live triggers") || format!("{live}").contains("not archived")
    );

    let archived = lifecycle
        .archive_project(LifecycleArchiveSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &project_id,
            skip_stop_triggers: false,
            now_ms: 96,
        })
        .expect("archive");
    assert_eq!(archived.state, "archived");
    assert_eq!(archived.is_backup, false);

    let preview = lifecycle
        .preview_delete(LifecycleDeletePreviewSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &project_id,
            now_ms: 97,
        })
        .expect("preview");
    assert_eq!(preview.armed_triggers, 0);

    let no_second = lifecycle
        .confirm_delete(LifecycleDeleteConfirmSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &project_id,
            impact_digest: &preview.impact_digest,
            second_confirm: false,
            physical_delete: false,
            now_ms: 98,
        })
        .expect_err("second confirm");
    assert!(format!("{no_second}").contains("second confirmation"));

    let physical = lifecycle
        .confirm_delete(LifecycleDeleteConfirmSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &project_id,
            impact_digest: &preview.impact_digest,
            second_confirm: true,
            physical_delete: true,
            now_ms: 99,
        })
        .expect_err("physical");
    assert!(format!("{physical}").contains("physical"));

    let confirmed = lifecycle
        .confirm_delete(LifecycleDeleteConfirmSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &project_id,
            impact_digest: &preview.impact_digest,
            second_confirm: true,
            physical_delete: false,
            now_ms: 100,
        })
        .expect("confirm");
    assert!(confirmed.tombstoned);
    let still = projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert_eq!(still.state, "deletion-preview");
    assert_eq!(still.current_plan_revision_id.as_deref(), Some("tombstone"));

    let copy_id = activate(&projects);
    let secret = lifecycle
        .export_project(LifecycleExportSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &copy_id,
            include_secrets: true,
            now_ms: 101,
        })
        .expect_err("export secrets");
    assert!(matches!(secret, ProjectAggregateError::Invalid { .. }));

    let exported = lifecycle
        .export_project(LifecycleExportSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &copy_id,
            include_secrets: false,
            now_ms: 102,
        })
        .expect("export");
    assert!(!exported.is_authority);
    assert!(!exported.is_backup);
    assert!(!exported.include_secrets);
    let payload = std::fs::read_to_string(&exported.path).expect("read export");
    assert!(!payload.to_ascii_lowercase().contains("api_key"));
    assert!(!payload.contains("sk-"));

    let backup = lifecycle
        .record_restore_point(LifecycleRestoreSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &copy_id,
            home_id: None,
            claimed_as_backup: true,
            now_ms: 103,
        })
        .expect_err("restore-as-backup");
    assert!(
        matches!(backup, ProjectAggregateError::Rejected { detail } if detail.contains("backup")),
        "{backup}"
    );

    let point = lifecycle
        .record_restore_point(LifecycleRestoreSpec {
            caller: ConfirmCaller::OwnerManagement,
            project_id: &copy_id,
            home_id: None,
            claimed_as_backup: false,
            now_ms: 104,
        })
        .expect("restore point");
    assert!(!point.is_backup);
    assert!(point.same_disk);
    assert_eq!(point.kind, "local-restore-point");
}
