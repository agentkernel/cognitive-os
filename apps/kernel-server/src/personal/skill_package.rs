//! Daemon-private admission boundary for local Skill package imports.
//!
//! The local path is provenance only. This module authenticates the caller on
//! the management channel before delegating immutable package/revision facts
//! to the authority store; it never executes package content or grants any
//! capability.

use super::auth::LocalSessionAuthority;
use cognitive_kernel::ports::{SkillPackageRow, SkillRevisionRow, SkillStore, StorePortError};
use std::time::Instant;

/// Admit one local Skill package import after owner-local management
/// authentication. A task, Pi, or worker bearer fails closed before storage.
pub(crate) fn import_local_skill_package<S>(
    store: &S,
    session_authority: &mut LocalSessionAuthority,
    management_bearer: &str,
    now: Instant,
    package: &SkillPackageRow,
    revision: &SkillRevisionRow,
) -> Result<(), StorePortError>
where
    S: SkillStore,
{
    session_authority
        .authorize_daemon_administrator(management_bearer, now)
        .map_err(|error| StorePortError::Conflict {
            detail: format!("local Skill import is not authorized: {}", error.code()),
        })?;
    store.append_skill_import(package, revision)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::personal::auth::{ChannelClass, SessionIssueRequest};
    use crate::personal::bounds::PersonalResourceBounds;
    use cognitive_domain::ObjectId;
    use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore, prepare_personal_databases};

    fn object_id(sequence: u64) -> ObjectId {
        ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
    }

    fn package_and_revision() -> (SkillPackageRow, SkillRevisionRow) {
        let package = SkillPackageRow {
            package_id: object_id(1),
            workspace_scope: "workspace://tenant-a/project".to_owned(),
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

    #[test]
    fn task_bearer_cannot_import_local_skill_package() {
        let temporary_directory = tempfile::tempdir().unwrap();
        let root = temporary_directory.path();
        let layout = PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        );
        prepare_personal_databases(&layout).unwrap();
        let store = SqliteAuthorityStore::open(&layout.authority_database_path()).unwrap();
        let now = Instant::now();
        let mut authority = LocalSessionAuthority::initialize(
            layout.runtime_dir().join("skill-import-bootstrap"),
            PersonalResourceBounds::default(),
        )
        .unwrap();
        let task_session = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Task,
                    principal_id: "principal://tenant-a/owner".to_owned(),
                    bootstrap_secret: authority.bootstrap_secret_for_tests().to_owned(),
                },
                now,
            )
            .unwrap();
        let (package, revision) = package_and_revision();
        assert!(matches!(
            import_local_skill_package(
                &store,
                &mut authority,
                &task_session.token,
                now,
                &package,
                &revision,
            ),
            Err(StorePortError::Conflict { .. })
        ));
        store.append_skill_import(&package, &revision).unwrap();
    }
}
mod skill_package;
