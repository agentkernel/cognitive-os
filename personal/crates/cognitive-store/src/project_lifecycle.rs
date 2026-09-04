//! P13-T09 Project lifecycle: copy / archive / delete / restore point / export.
//!
//! Uses existing `p11_project` states, `p13_routine_arming`, `p11_grant`,
//! `p11_employee`, and `p11_windows_host_*`. v41 is reserved in this module
//! and is not registered in `personal_db.rs` this slice (T11 holds that file
//! and v40). No physical DROP. Restore points are not backups. Export is
//! never authority and default-excludes secrets.

use super::{
    ConfirmCaller, ProjectAggregateError, ProjectAggregateStore, looks_like_secret, next_id,
    unavailable,
};
use crate::migration::MigrationPlanEntry;
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Reserved v41. Not applied until `personal_db.rs` registers this entry
/// after T11 releases that file. Runtime paths do not depend on these tables.
pub const PROJECT_LIFECYCLE_SCHEMA_V41: &str = "
CREATE TABLE p13_project_lifecycle_event (
  event_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  kind TEXT NOT NULL CHECK (kind IN (
    'copy','archive','delete-preview','delete-confirm','export','restore-point'
  )),
  is_backup INTEGER NOT NULL CHECK (is_backup = 0),
  is_authority INTEGER NOT NULL CHECK (is_authority = 0),
  payload_digest TEXT NOT NULL CHECK (length(payload_digest) = 64),
  created_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p13_project_lifecycle_event_project
  ON p13_project_lifecycle_event(project_id, created_at);
";

pub fn project_lifecycle_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(41, PROJECT_LIFECYCLE_SCHEMA_V41)
}

#[derive(Debug, Clone, Copy)]
pub struct LifecycleCopySpec<'a> {
    pub caller: ConfirmCaller,
    pub source_project_id: &'a str,
    pub inherit_grants: bool,
    pub inherit_seats: bool,
    pub inherit_runtime: bool,
    pub now_ms: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct LifecycleArchiveSpec<'a> {
    pub caller: ConfirmCaller,
    pub project_id: &'a str,
    pub skip_stop_triggers: bool,
    pub now_ms: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct LifecycleDeletePreviewSpec<'a> {
    pub caller: ConfirmCaller,
    pub project_id: &'a str,
    pub now_ms: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct LifecycleDeleteConfirmSpec<'a> {
    pub caller: ConfirmCaller,
    pub project_id: &'a str,
    pub impact_digest: &'a str,
    pub second_confirm: bool,
    pub physical_delete: bool,
    pub now_ms: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct LifecycleExportSpec<'a> {
    pub caller: ConfirmCaller,
    pub project_id: &'a str,
    pub include_secrets: bool,
    pub now_ms: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct LifecycleRestoreSpec<'a> {
    pub caller: ConfirmCaller,
    pub project_id: &'a str,
    pub home_id: Option<&'a str>,
    pub claimed_as_backup: bool,
    pub now_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLifecycleView {
    pub project_id: String,
    pub state: String,
    pub tombstoned: bool,
    pub is_backup: bool,
    pub data_dir: Option<String>,
    pub paused_armings: i64,
    pub pending_impact_digest: Option<String>,
    pub restore_points: Vec<RestorePointView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePreviewView {
    pub project_id: String,
    pub state: String,
    pub routines: i64,
    pub members: i64,
    pub outputs: i64,
    pub grants: i64,
    pub armed_triggers: i64,
    pub impact_digest: String,
    pub tombstoned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExportView {
    pub project_id: String,
    pub path: String,
    pub is_authority: bool,
    pub is_backup: bool,
    pub include_secrets: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestorePointView {
    pub restore_point_id: String,
    pub project_id: String,
    pub home_id: String,
    pub kind: String,
    pub same_disk: bool,
    pub is_backup: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleEventView {
    pub event_id: String,
    pub project_id: String,
    pub kind: String,
    pub is_backup: bool,
    pub is_authority: bool,
}

#[derive(Clone)]
pub struct ProjectLifecycleStore {
    conn: Arc<Mutex<Connection>>,
}

impl ProjectLifecycleStore {
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
        }
    }

    pub fn open_path(path: &Path) -> Result<Self, ProjectAggregateError> {
        Ok(Self {
            conn: ProjectAggregateStore::open_path(path)?.conn,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    fn projects(&self) -> ProjectAggregateStore {
        ProjectAggregateStore {
            conn: Arc::clone(&self.conn),
        }
    }

    /// Copy as an inactive charter-only duplicate. Inherit flags fail closed.
    /// A seated/granted source may be copied; the 副本 itself never receives
    /// grants, seats, or armed Routines.
    pub fn copy_project(
        &self,
        spec: LifecycleCopySpec<'_>,
    ) -> Result<String, ProjectAggregateError> {
        ProjectAggregateStore::require_owner(spec.caller)?;
        if spec.inherit_grants || spec.inherit_seats || spec.inherit_runtime {
            return Err(ProjectAggregateError::Rejected {
                detail: "copy refuses inherited grant, seating, or runtime",
            });
        }
        let copy_id = self
            .projects()
            .copy_project(spec.source_project_id, spec.now_ms)?;
        {
            let conn = self.lock()?;
            let seated = count_named(
                &conn,
                "SELECT COUNT(*) FROM p11_employee
                  WHERE project_id = ?1 AND state IN ('seated','seating')",
                &copy_id,
                "copy seating",
            )?;
            let grants = count_named(
                &conn,
                "SELECT COUNT(*) FROM p11_grant WHERE project_id = ?1",
                &copy_id,
                "copy grants",
            )?;
            let armed = count_named(
                &conn,
                "SELECT COUNT(*) FROM p13_routine_arming WHERE project_id = ?1 AND state = 'armed'",
                &copy_id,
                "copy armed",
            )?;
            if seated > 0 || grants > 0 || armed > 0 {
                return Err(ProjectAggregateError::Rejected {
                    detail: "copy must not inherit grant, seating, or armed routines",
                });
            }
        }
        let _ = self.ensure_project_data_dir(&copy_id);
        Ok(copy_id)
    }

    pub fn archive_project(
        &self,
        spec: LifecycleArchiveSpec<'_>,
    ) -> Result<ProjectLifecycleView, ProjectAggregateError> {
        ProjectAggregateStore::require_owner(spec.caller)?;
        if spec.skip_stop_triggers {
            return Err(ProjectAggregateError::Rejected {
                detail: "archive refuses skip_stop_triggers; stop Routine/Trigger first",
            });
        }
        let conn = self.lock()?;
        let (state, plan_id): (String, Option<String>) = conn
            .query_row(
                "SELECT state, current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [spec.project_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("archive project"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        if is_tombstoned(&state, plan_id.as_deref()) {
            return Err(ProjectAggregateError::Rejected {
                detail: "tombstoned project cannot be archived",
            });
        }
        let paused_armings = conn
            .execute(
                "UPDATE p13_routine_arming
                    SET state = 'paused', apply_mode = 'pause', updated_at = ?1
                  WHERE project_id = ?2 AND state = 'armed'",
                params![spec.now_ms, spec.project_id],
            )
            .map_err(unavailable("pause armed routines"))? as i64;
        conn.execute(
            "UPDATE p11_project SET state = 'archived' WHERE project_id = ?1",
            [spec.project_id],
        )
        .map_err(unavailable("archive project state"))?;
        drop(conn);
        let data_dir = self.ensure_project_data_dir(spec.project_id).ok();
        Ok(ProjectLifecycleView {
            project_id: spec.project_id.to_owned(),
            state: "archived".to_owned(),
            tombstoned: false,
            is_backup: false,
            data_dir: data_dir.map(|path| path.to_string_lossy().into_owned()),
            paused_armings,
            pending_impact_digest: None,
            restore_points: Vec::new(),
        })
    }

    pub fn preview_delete(
        &self,
        spec: LifecycleDeletePreviewSpec<'_>,
    ) -> Result<DeletePreviewView, ProjectAggregateError> {
        ProjectAggregateStore::require_owner(spec.caller)?;
        let conn = self.lock()?;
        let (state, plan_id): (String, Option<String>) = conn
            .query_row(
                "SELECT state, current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [spec.project_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("delete preview project"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        if is_tombstoned(&state, plan_id.as_deref()) {
            return Err(ProjectAggregateError::Rejected {
                detail: "project already tombstoned",
            });
        }
        if state != "archived" && state != "deletion-preview" {
            return Err(ProjectAggregateError::Rejected {
                detail: "delete refused while live triggers remain or project is not archived",
            });
        }
        let armed = count_named(
            &conn,
            "SELECT COUNT(*) FROM p13_routine_arming WHERE project_id = ?1 AND state = 'armed'",
            spec.project_id,
            "armed triggers",
        )?;
        if armed > 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "delete refused while live triggers remain",
            });
        }
        conn.execute(
            "UPDATE p11_project SET state = 'deletion-preview' WHERE project_id = ?1",
            [spec.project_id],
        )
        .map_err(unavailable("mark deletion-preview"))?;
        impact_from_conn(&conn, spec.project_id, "deletion-preview", false)
    }

    pub fn confirm_delete(
        &self,
        spec: LifecycleDeleteConfirmSpec<'_>,
    ) -> Result<DeletePreviewView, ProjectAggregateError> {
        ProjectAggregateStore::require_owner(spec.caller)?;
        if spec.physical_delete {
            return Err(ProjectAggregateError::Rejected {
                detail: "delete is logical; physical delete is forbidden",
            });
        }
        if !spec.second_confirm {
            return Err(ProjectAggregateError::Rejected {
                detail: "delete requires a second confirmation",
            });
        }
        let conn = self.lock()?;
        let (state, plan_id): (String, Option<String>) = conn
            .query_row(
                "SELECT state, current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [spec.project_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("delete confirm project"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        if state != "deletion-preview" {
            return Err(ProjectAggregateError::Rejected {
                detail: "delete confirm requires deletion-preview",
            });
        }
        if is_tombstoned(&state, plan_id.as_deref()) {
            return Err(ProjectAggregateError::Conflict {
                detail: "project already tombstoned",
            });
        }
        let impact = impact_from_conn(&conn, spec.project_id, &state, false)?;
        if impact.impact_digest != spec.impact_digest {
            return Err(ProjectAggregateError::Stale {
                detail: "delete impact digest stale",
            });
        }
        if impact.armed_triggers > 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "delete refused while live triggers remain",
            });
        }
        let still_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_project WHERE project_id = ?1",
                [spec.project_id],
                |row| row.get(0),
            )
            .map_err(unavailable("delete confirm exists"))?;
        if still_exists != 1 {
            return Err(ProjectAggregateError::Unavailable {
                detail: "delete must not drop the project row".to_owned(),
            });
        }
        conn.execute(
            "UPDATE p11_project
                SET current_plan_revision_id = ?1, state = 'deletion-preview'
              WHERE project_id = ?2",
            params![TOMBSTONE_PLAN_REF, spec.project_id],
        )
        .map_err(unavailable("tombstone project"))?;
        impact_from_conn(&conn, spec.project_id, "deletion-preview", true)
    }

    pub fn export_project(
        &self,
        spec: LifecycleExportSpec<'_>,
    ) -> Result<ProjectExportView, ProjectAggregateError> {
        ProjectAggregateStore::require_owner(spec.caller)?;
        if spec.include_secrets {
            return Err(ProjectAggregateError::Invalid {
                detail: "export excludes secrets",
            });
        }
        let conn = self.lock()?;
        let (state, charter_id, plan_id): (String, String, Option<String>) = conn
            .query_row(
                "SELECT state, current_charter_revision_id, current_plan_revision_id
                   FROM p11_project WHERE project_id = ?1",
                [spec.project_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("export project"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        let charter_digest: String = conn
            .query_row(
                "SELECT content_digest FROM p11_charter_revision WHERE charter_revision_id = ?1",
                [&charter_id],
                |row| row.get(0),
            )
            .map_err(unavailable("export charter"))?;
        drop(conn);
        let payload = format!(
            "{{\"schema\":\"personal-project-export/v1\",\"is_authority\":false,\"is_backup\":false,\"include_secrets\":false,\"project_id\":{},\"state\":{},\"charter_digest\":{},\"plan_revision_id\":{},\"exported_at\":{}}}",
            json_string(spec.project_id),
            json_string(&state),
            json_string(&charter_digest),
            plan_id
                .as_deref()
                .map(json_string)
                .unwrap_or_else(|| "null".to_owned()),
            spec.now_ms
        );
        if looks_like_secret(&payload) {
            return Err(ProjectAggregateError::Invalid {
                detail: "export excludes secrets",
            });
        }
        let dir = self.ensure_project_data_dir(spec.project_id)?;
        let export_dir = dir.join("export");
        std::fs::create_dir_all(&export_dir).map_err(|source| {
            ProjectAggregateError::Unavailable {
                detail: format!("create export dir: {source}"),
            }
        })?;
        let path = export_dir.join(format!("project-{}.json", spec.now_ms));
        std::fs::write(&path, payload.as_bytes()).map_err(|source| {
            ProjectAggregateError::Unavailable {
                detail: format!("write export: {source}"),
            }
        })?;
        Ok(ProjectExportView {
            project_id: spec.project_id.to_owned(),
            path: path.to_string_lossy().into_owned(),
            is_authority: false,
            is_backup: false,
            include_secrets: false,
        })
    }

    pub fn record_restore_point(
        &self,
        spec: LifecycleRestoreSpec<'_>,
    ) -> Result<RestorePointView, ProjectAggregateError> {
        ProjectAggregateStore::require_owner(spec.caller)?;
        if spec.claimed_as_backup {
            return Err(ProjectAggregateError::Rejected {
                detail: "restore-as-backup claim rejected",
            });
        }
        let exists = self.projects().get_project(spec.project_id)?.ok_or(
            ProjectAggregateError::NotFound {
                detail: "project not found",
            },
        )?;
        let _ = exists;
        let home = self.admitted_home().ok();
        let restore_point_id = next_id("restore")?;
        if let Some((home_id, _)) = home.as_ref() {
            let conn = self.lock()?;
            conn.execute(
                "INSERT INTO p11_windows_host_restore_point (
                    restore_point_id, home_id, kind, same_disk, created_at
                 ) VALUES (?1,?2,'local-restore-point',1,?3)",
                params![restore_point_id, home_id, spec.now_ms],
            )
            .map_err(unavailable("insert restore point"))?;
        }
        let _ = self.ensure_project_data_dir(spec.project_id);
        Ok(RestorePointView {
            restore_point_id,
            project_id: spec.project_id.to_owned(),
            home_id: home
                .map(|(id, _)| id)
                .unwrap_or_else(|| "local-same-disk".to_owned()),
            kind: "local-restore-point".to_owned(),
            same_disk: true,
            is_backup: false,
        })
    }

    pub fn lifecycle_view(
        &self,
        project_id: &str,
    ) -> Result<ProjectLifecycleView, ProjectAggregateError> {
        let conn = self.lock()?;
        let (state, plan_id): (String, Option<String>) = conn
            .query_row(
                "SELECT state, current_plan_revision_id FROM p11_project WHERE project_id = ?1",
                [project_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("lifecycle view"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "project not found",
            })?;
        let paused_armings = count_named(
            &conn,
            "SELECT COUNT(*) FROM p13_routine_arming WHERE project_id = ?1 AND state = 'paused'",
            project_id,
            "paused armings",
        )?;
        let tombstoned = is_tombstoned(&state, plan_id.as_deref());
        let pending_impact_digest = if state == "deletion-preview" && !tombstoned {
            Some(impact_from_conn(&conn, project_id, &state, false)?.impact_digest)
        } else {
            None
        };
        let restore_points = list_restore_points(&conn, project_id)?;
        drop(conn);
        Ok(ProjectLifecycleView {
            project_id: project_id.to_owned(),
            state: state.clone(),
            tombstoned,
            is_backup: false,
            data_dir: self
                .project_data_dir(project_id)
                .ok()
                .filter(|path| path.exists())
                .map(|path| path.to_string_lossy().into_owned()),
            paused_armings,
            pending_impact_digest,
            restore_points,
        })
    }

    fn admitted_home(&self) -> Result<(String, String), ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT home_id, data_dir FROM p11_windows_host_home ORDER BY created_at LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(unavailable("admitted home"))?
        .ok_or(ProjectAggregateError::Rejected {
            detail: "export and restore points require an admitted Personal Home",
        })
    }

    fn project_data_dir(&self, project_id: &str) -> Result<PathBuf, ProjectAggregateError> {
        let data_dir = match self.admitted_home() {
            Ok((_, dir)) => PathBuf::from(dir),
            Err(_) => {
                let conn = self.lock()?;
                let file: String = conn
                    .query_row(
                        "SELECT file FROM pragma_database_list WHERE name = 'main'",
                        [],
                        |row| row.get(0),
                    )
                    .map_err(unavailable("pragma database list"))?;
                Path::new(&file).parent().map(Path::to_path_buf).ok_or(
                    ProjectAggregateError::Unavailable {
                        detail: "authority database has no parent data dir".to_owned(),
                    },
                )?
            }
        };
        Ok(data_dir.join("projects").join(sanitize_segment(project_id)))
    }

    fn ensure_project_data_dir(&self, project_id: &str) -> Result<PathBuf, ProjectAggregateError> {
        let dir = self.project_data_dir(project_id)?;
        std::fs::create_dir_all(&dir).map_err(|source| ProjectAggregateError::Unavailable {
            detail: format!("create project data dir: {source}"),
        })?;
        Ok(dir)
    }
}

const TOMBSTONE_PLAN_REF: &str = "tombstone";

fn is_tombstoned(state: &str, plan_id: Option<&str>) -> bool {
    state == "deletion-preview" && plan_id == Some(TOMBSTONE_PLAN_REF)
}

fn count_named(
    conn: &Connection,
    sql: &str,
    project_id: &str,
    operation: &'static str,
) -> Result<i64, ProjectAggregateError> {
    conn.query_row(sql, [project_id], |row| row.get(0))
        .map_err(unavailable(operation))
}

fn impact_from_conn(
    conn: &Connection,
    project_id: &str,
    state: &str,
    tombstoned: bool,
) -> Result<DeletePreviewView, ProjectAggregateError> {
    let routines = count_named(
        conn,
        "SELECT COUNT(*) FROM p11_routine WHERE project_id = ?1",
        project_id,
        "routine count",
    )?;
    let members = count_named(
        conn,
        "SELECT COUNT(*) FROM p11_employee WHERE project_id = ?1",
        project_id,
        "member count",
    )?;
    let outputs = count_named(
        conn,
        "SELECT COUNT(*) FROM p13_attempt_artifact WHERE project_id = ?1",
        project_id,
        "output count",
    )?;
    let grants = count_named(
        conn,
        "SELECT COUNT(*) FROM p11_grant WHERE project_id = ?1",
        project_id,
        "grant count",
    )?;
    let armed_triggers = count_named(
        conn,
        "SELECT COUNT(*) FROM p13_routine_arming WHERE project_id = ?1 AND state = 'armed'",
        project_id,
        "armed count",
    )?;
    let canonical = format!(
        "project={project_id}\nroutines={routines}\nmembers={members}\noutputs={outputs}\ngrants={grants}\narmed={armed_triggers}"
    );
    Ok(DeletePreviewView {
        project_id: project_id.to_owned(),
        state: state.to_owned(),
        routines,
        members,
        outputs,
        grants,
        armed_triggers,
        impact_digest: ProjectAggregateStore::digest_hex(canonical.as_bytes()),
        tombstoned,
    })
}

fn list_restore_points(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<RestorePointView>, ProjectAggregateError> {
    let mut statement = conn
        .prepare(
            "SELECT restore_point_id, home_id, kind, same_disk
               FROM p11_windows_host_restore_point
              ORDER BY created_at",
        )
        .map_err(unavailable("list restore points"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(RestorePointView {
                restore_point_id: row.get(0)?,
                project_id: project_id.to_owned(),
                home_id: row.get(1)?,
                kind: row.get(2)?,
                same_disk: row.get::<_, i64>(3)? == 1,
                is_backup: false,
            })
        })
        .map_err(unavailable("restore point query"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("restore point rows"))
}

fn sanitize_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "project".to_owned()
    } else {
        out
    }
}

fn json_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
