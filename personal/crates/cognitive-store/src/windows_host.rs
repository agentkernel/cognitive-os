//! Personal-private Windows host / tray / background policy (P11-T02,
//! authority migration v34).
//!
//! Daemon-owned Personal Home `app/` + `data/` layout, typed lifecycle
//! (tray observes and requests; it does not write authority), close
//! background-or-pause honesty, offline/missed segments, ordered seven-step
//! wake/restart recovery, and same-disk restore points that are not disaster
//! backups. Native tray/ACL/sleep/SecretStore E2E remains
//! `Requires-environment` / `not-run` until `DEV-WINDOWS-NATIVE-OPC-01` is
//! qualified. This module is not a second credential plane and does not
//! embed DSH web as the host shell.

use crate::employee::EmployeeStore;
use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::{Arc, Mutex};

/// Personal-private Windows host envelope (P11-T02). Hidden capability, not chrome.
pub const WINDOWS_HOST_PROJECTION_ID: &str = "cognitiveos.personal.windows-host/0.1";

/// Ordered wake/restart steps. Skipping any step is rejected.
pub const WAKE_RECOVERY_STEPS: [&str; 7] = [
    "reload-authority-fresh-epoch",
    "reconcile-pending-unknown-effects",
    "reobserve-clock-fs-secretstore-provider-broker-dsh",
    "reauthorize-project-task-budget-binding",
    "rebuild-context",
    "classify-missed-ask-catchup",
    "resume-eligible-only",
];

/// Authority migration v34: Personal Home, daemon bind, DSH children,
/// offline/missed segments, ordered recovery, restore points.
pub const WINDOWS_HOST_SCHEMA_V34: &str = "
CREATE TABLE p11_windows_host_home (
  home_id TEXT PRIMARY KEY,
  install_root TEXT NOT NULL UNIQUE,
  app_dir TEXT NOT NULL,
  data_dir TEXT NOT NULL,
  acl_policy TEXT NOT NULL CHECK (acl_policy = 'owner-only-dacl'),
  data_preserved INTEGER NOT NULL CHECK (data_preserved IN (0,1)),
  app_replaced INTEGER NOT NULL CHECK (app_replaced IN (0,1)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_windows_host_daemon (
  daemon_id TEXT PRIMARY KEY,
  home_id TEXT NOT NULL UNIQUE REFERENCES p11_windows_host_home(home_id),
  epoch INTEGER NOT NULL CHECK (epoch >= 1),
  state TEXT NOT NULL CHECK (state IN (
    'bound','paused','offline','recovering','resumed'
  )),
  can_honor_background INTEGER NOT NULL CHECK (can_honor_background IN (0,1)),
  tray_role TEXT NOT NULL CHECK (tray_role = 'observe-and-request'),
  close_disposition TEXT CHECK (close_disposition IN ('background-honored','paused')),
  bound_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;
CREATE TABLE p11_windows_host_dsh_child (
  child_id TEXT PRIMARY KEY,
  daemon_id TEXT NOT NULL REFERENCES p11_windows_host_daemon(daemon_id),
  home_id TEXT NOT NULL REFERENCES p11_windows_host_home(home_id),
  state TEXT NOT NULL CHECK (state IN ('bound','exited','orphaned')),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p11_windows_host_dsh_home
  ON p11_windows_host_dsh_child(home_id, state);
CREATE TABLE p11_windows_host_offline_segment (
  segment_id TEXT PRIMARY KEY,
  home_id TEXT NOT NULL REFERENCES p11_windows_host_home(home_id),
  cause TEXT NOT NULL CHECK (cause IN (
    'sleep','shutdown','daemon-stop','network-loss','secretstore-locked','provider-outage'
  )),
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  missed_visible INTEGER NOT NULL CHECK (missed_visible = 1)
) STRICT;
CREATE INDEX p11_windows_host_offline_home
  ON p11_windows_host_offline_segment(home_id, started_at);
CREATE TABLE p11_windows_host_recovery (
  recovery_id TEXT PRIMARY KEY,
  home_id TEXT NOT NULL REFERENCES p11_windows_host_home(home_id),
  epoch INTEGER NOT NULL,
  current_step INTEGER NOT NULL CHECK (current_step BETWEEN 0 AND 7),
  catch_up_asked INTEGER NOT NULL CHECK (catch_up_asked IN (0,1)),
  resume_eligible INTEGER NOT NULL CHECK (resume_eligible IN (0,1)),
  skipped INTEGER NOT NULL CHECK (skipped = 0),
  started_at INTEGER NOT NULL,
  completed_at INTEGER
) STRICT;
CREATE TABLE p11_windows_host_restore_point (
  restore_point_id TEXT PRIMARY KEY,
  home_id TEXT NOT NULL REFERENCES p11_windows_host_home(home_id),
  kind TEXT NOT NULL CHECK (kind = 'local-restore-point'),
  same_disk INTEGER NOT NULL CHECK (same_disk = 1),
  created_at INTEGER NOT NULL
) STRICT;
";

/// v34 migration entry.
pub fn windows_host_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(34, WINDOWS_HOST_SCHEMA_V34)
}

/// Personal Home admit input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HomeAdmitSpec<'a> {
    pub install_root: &'a str,
    pub app_dir: &'a str,
    pub data_dir: &'a str,
    pub acl_policy: &'a str,
    pub argv: &'a [&'a str],
    pub env_pairs: &'a [(&'a str, &'a str)],
    pub now_ms: i64,
}

/// Daemon bind input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaemonBindSpec<'a> {
    pub home_id: &'a str,
    pub can_honor_background: bool,
    pub now_ms: i64,
}

/// Close request. Tray/UI may ask; daemon honors only when it can.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseRequestSpec<'a> {
    pub home_id: &'a str,
    pub choice: &'a str,
    pub now_ms: i64,
}

/// Admitted Personal Home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsHostHome {
    pub home_id: String,
    pub install_root: String,
    pub app_dir: String,
    pub data_dir: String,
    pub data_preserved: bool,
    pub app_replaced: bool,
}

/// Bound daemon observation. Tray icon is not proof of work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsHostDaemon {
    pub daemon_id: String,
    pub home_id: String,
    pub epoch: i64,
    pub state: String,
    pub can_honor_background: bool,
    pub tray_role: String,
    pub tray_proves_work: bool,
    pub close_disposition: Option<String>,
}

/// Ordered recovery observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsHostRecovery {
    pub recovery_id: String,
    pub home_id: String,
    pub epoch: i64,
    pub current_step: i64,
    pub current_step_name: String,
    pub catch_up_asked: bool,
    pub resume_eligible: bool,
}

/// Redacted host status for tray/UI observation. Never contains secret material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsHostStatus {
    pub home_id: String,
    pub install_root: String,
    pub app_dir: String,
    pub data_dir: String,
    pub data_preserved: bool,
    pub daemon_id: Option<String>,
    pub epoch: i64,
    pub daemon_state: String,
    pub can_honor_background: bool,
    pub tray_role: String,
    pub tray_proves_work: bool,
    pub close_disposition: Option<String>,
    pub missed_segments: i64,
    pub recovery_step: i64,
    pub resume_eligible: bool,
    pub restore_kind: Option<String>,
}

/// Personal-private Windows host store on the authority writer.
#[derive(Clone)]
pub struct WindowsHostStore {
    conn: Arc<Mutex<Connection>>,
}

impl WindowsHostStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
        }
    }

    /// Open the authority database path (tests).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let conn = Connection::open(path).map_err(unavailable("open"))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(unavailable("pragma"))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// Admit Personal Home `app/` + `data/`. Upgrades replace app and preserve data.
    pub fn admit_home(
        &self,
        caller: ConfirmCaller,
        spec: &HomeAdmitSpec<'_>,
    ) -> Result<WindowsHostHome, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        reject_secret_material(spec.argv, spec.env_pairs)?;
        reject_wrong_install_root(spec.install_root)?;
        reject_acl_escape(
            spec.install_root,
            spec.app_dir,
            spec.data_dir,
            spec.acl_policy,
        )?;

        let conn = self.lock()?;
        let existing: Option<(String, String)> = conn
            .query_row(
                "SELECT home_id, data_dir FROM p11_windows_host_home WHERE install_root = ?1",
                [spec.install_root],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("lookup home"))?;
        if let Some((home_id, data_dir)) = existing {
            if normalize_path(&data_dir) != normalize_path(spec.data_dir) {
                return Err(ProjectAggregateError::Rejected {
                    detail: "upgrade must preserve Personal Home data/",
                });
            }
            conn.execute(
                "UPDATE p11_windows_host_home
                    SET app_dir = ?1, data_preserved = 1, app_replaced = 1, updated_at = ?2
                  WHERE home_id = ?3",
                params![spec.app_dir, spec.now_ms, home_id],
            )
            .map_err(unavailable("upgrade home"))?;
            return Ok(WindowsHostHome {
                home_id,
                install_root: spec.install_root.to_owned(),
                app_dir: spec.app_dir.to_owned(),
                data_dir: spec.data_dir.to_owned(),
                data_preserved: true,
                app_replaced: true,
            });
        }

        let home_id = next_id("home")?;
        conn.execute(
            "INSERT INTO p11_windows_host_home (
                home_id, install_root, app_dir, data_dir, acl_policy,
                data_preserved, app_replaced, created_at, updated_at
             ) VALUES (?1,?2,?3,?4,'owner-only-dacl',0,0,?5,?5)",
            params![
                home_id,
                spec.install_root,
                spec.app_dir,
                spec.data_dir,
                spec.now_ms
            ],
        )
        .map_err(unavailable("insert home"))?;
        Ok(WindowsHostHome {
            home_id,
            install_root: spec.install_root.to_owned(),
            app_dir: spec.app_dir.to_owned(),
            data_dir: spec.data_dir.to_owned(),
            data_preserved: false,
            app_replaced: false,
        })
    }

    /// Bind the single daemon for a Personal Home. Tray/UI cannot write authority.
    pub fn bind_daemon(
        &self,
        caller: ConfirmCaller,
        spec: &DaemonBindSpec<'_>,
    ) -> Result<WindowsHostDaemon, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let conn = self.lock()?;
        require_home(&conn, spec.home_id)?;
        let existing: Option<(String, String, i64)> = conn
            .query_row(
                "SELECT daemon_id, state, epoch FROM p11_windows_host_daemon WHERE home_id = ?1",
                [spec.home_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(unavailable("lookup daemon"))?;
        let honor = i64::from(spec.can_honor_background);
        if let Some((daemon_id, state, epoch)) = existing {
            if state == "bound" || state == "recovering" || state == "resumed" {
                return Err(ProjectAggregateError::Conflict {
                    detail: "duplicate daemon rejected",
                });
            }
            let next_epoch = epoch + 1;
            conn.execute(
                "UPDATE p11_windows_host_daemon
                    SET epoch = ?1, state = 'bound', can_honor_background = ?2,
                        close_disposition = NULL, updated_at = ?3
                  WHERE daemon_id = ?4",
                params![next_epoch, honor, spec.now_ms, daemon_id],
            )
            .map_err(unavailable("rebind daemon"))?;
            return Ok(daemon_view(
                daemon_id,
                spec.home_id,
                next_epoch,
                "bound",
                spec.can_honor_background,
                None,
            ));
        }

        let daemon_id = next_id("daemon")?;
        conn.execute(
            "INSERT INTO p11_windows_host_daemon (
                daemon_id, home_id, epoch, state, can_honor_background, tray_role,
                close_disposition, bound_at, updated_at
             ) VALUES (?1,?2,1,'bound',?3,'observe-and-request',NULL,?4,?4)",
            params![daemon_id, spec.home_id, honor, spec.now_ms],
        )
        .map_err(unavailable("insert daemon"))?;
        Ok(daemon_view(
            daemon_id,
            spec.home_id,
            1,
            "bound",
            spec.can_honor_background,
            None,
        ))
    }

    /// Close asks background or pause only if the daemon can honor background.
    pub fn request_close(
        &self,
        caller: ConfirmCaller,
        spec: &CloseRequestSpec<'_>,
    ) -> Result<WindowsHostDaemon, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if spec.choice != "background" && spec.choice != "pause" {
            return Err(ProjectAggregateError::Invalid {
                detail: "close choice must be background or pause",
            });
        }
        let conn = self.lock()?;
        let (daemon_id, state, epoch, honor): (String, String, i64, i64) = conn
            .query_row(
                "SELECT daemon_id, state, epoch, can_honor_background
                   FROM p11_windows_host_daemon WHERE home_id = ?1",
                [spec.home_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(unavailable("lookup daemon for close"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "daemon not bound",
            })?;
        if state != "bound" && state != "resumed" {
            return Err(ProjectAggregateError::Rejected {
                detail: "fake background rejected",
            });
        }
        if spec.choice == "background" && honor == 0 {
            return Err(ProjectAggregateError::Rejected {
                detail: "fake background rejected",
            });
        }
        let (next_state, disposition) = if spec.choice == "background" {
            ("bound", "background-honored")
        } else {
            ("paused", "paused")
        };
        conn.execute(
            "UPDATE p11_windows_host_daemon
                SET state = ?1, close_disposition = ?2, updated_at = ?3
              WHERE daemon_id = ?4",
            params![next_state, disposition, spec.now_ms, daemon_id],
        )
        .map_err(unavailable("close daemon"))?;
        Ok(daemon_view(
            daemon_id,
            spec.home_id,
            epoch,
            next_state,
            honor == 1,
            Some(disposition.to_owned()),
        ))
    }

    /// Sleep / shutdown / daemon-stop / network / locked SecretStore / Provider outage.
    pub fn record_offline(
        &self,
        caller: ConfirmCaller,
        home_id: &str,
        cause: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        require_offline_cause(cause)?;
        let conn = self.lock()?;
        require_home(&conn, home_id)?;
        if let Some(daemon_id) = lookup_daemon_id(&conn, home_id)? {
            conn.execute(
                "UPDATE p11_windows_host_daemon
                    SET state = 'offline', updated_at = ?1
                  WHERE daemon_id = ?2",
                params![now_ms, daemon_id],
            )
            .map_err(unavailable("offline daemon"))?;
            conn.execute(
                "UPDATE p11_windows_host_dsh_child
                    SET state = 'orphaned', updated_at = ?1
                  WHERE daemon_id = ?2 AND state = 'bound'",
                params![now_ms, daemon_id],
            )
            .map_err(unavailable("orphan dsh on offline"))?;
        }
        let segment_id = next_id("offline")?;
        conn.execute(
            "INSERT INTO p11_windows_host_offline_segment (
                segment_id, home_id, cause, started_at, ended_at, missed_visible
             ) VALUES (?1,?2,?3,?4,NULL,1)",
            params![segment_id, home_id, cause, now_ms],
        )
        .map_err(unavailable("insert offline"))?;
        Ok(segment_id)
    }

    /// Bind a managed DSH child to the live daemon. Orphans are rejected.
    pub fn bind_dsh_child(
        &self,
        caller: ConfirmCaller,
        home_id: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        let conn = self.lock()?;
        let (daemon_id, state): (String, String) = conn
            .query_row(
                "SELECT daemon_id, state FROM p11_windows_host_daemon WHERE home_id = ?1",
                [home_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("lookup daemon for dsh"))?
            .ok_or(ProjectAggregateError::Rejected {
                detail: "orphan DSH rejected",
            })?;
        if state != "bound" && state != "resumed" {
            return Err(ProjectAggregateError::Rejected {
                detail: "orphan DSH rejected",
            });
        }
        let child_id = next_id("dshchild")?;
        conn.execute(
            "INSERT INTO p11_windows_host_dsh_child (
                child_id, daemon_id, home_id, state, created_at, updated_at
             ) VALUES (?1,?2,?3,'bound',?4,?4)",
            params![child_id, daemon_id, home_id, now_ms],
        )
        .map_err(unavailable("insert dsh child"))?;
        Ok(child_id)
    }

    /// Count DSH children in `orphaned` state.
    pub fn orphaned_dsh_count(&self, home_id: &str) -> Result<i64, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT COUNT(*) FROM p11_windows_host_dsh_child
              WHERE home_id = ?1 AND state = 'orphaned'",
            [home_id],
            |row| row.get(0),
        )
        .map_err(unavailable("count orphan dsh"))
    }

    /// Open a wake/restart recovery at step 0. Does not execute work.
    pub fn begin_recovery(
        &self,
        caller: ConfirmCaller,
        home_id: &str,
        host_awake: bool,
        now_ms: i64,
    ) -> Result<WindowsHostRecovery, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if !host_awake {
            return Err(ProjectAggregateError::Rejected {
                detail: "no execution while the host is off",
            });
        }
        let recovery_id = {
            let conn = self.lock()?;
            require_home(&conn, home_id)?;
            let daemon = conn
                .query_row(
                    "SELECT daemon_id, epoch FROM p11_windows_host_daemon WHERE home_id = ?1",
                    [home_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
                )
                .optional()
                .map_err(unavailable("lookup daemon for recovery"))?
                .ok_or(ProjectAggregateError::NotFound {
                    detail: "daemon not bound",
                })?;
            let open = conn
                .query_row(
                    "SELECT recovery_id FROM p11_windows_host_recovery
                      WHERE home_id = ?1 AND completed_at IS NULL
                      ORDER BY started_at DESC LIMIT 1",
                    [home_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(unavailable("lookup recovery"))?;
            if let Some(recovery_id) = open {
                recovery_id
            } else {
                let recovery_id = next_id("recovery")?;
                conn.execute(
                    "INSERT INTO p11_windows_host_recovery (
                        recovery_id, home_id, epoch, current_step, catch_up_asked,
                        resume_eligible, skipped, started_at, completed_at
                     ) VALUES (?1,?2,?3,0,0,0,0,?4,NULL)",
                    params![recovery_id, home_id, daemon.1, now_ms],
                )
                .map_err(unavailable("insert recovery"))?;
                conn.execute(
                    "UPDATE p11_windows_host_daemon
                        SET state = 'recovering', updated_at = ?1
                      WHERE daemon_id = ?2",
                    params![now_ms, daemon.0],
                )
                .map_err(unavailable("recovering daemon"))?;
                recovery_id
            }
        };
        self.load_recovery(&recovery_id)
    }

    /// Run the ordered seven-step wake/restart recovery. Steps cannot be skipped.
    pub fn run_ordered_recovery(
        &self,
        caller: ConfirmCaller,
        home_id: &str,
        host_awake: bool,
        now_ms: i64,
    ) -> Result<WindowsHostRecovery, ProjectAggregateError> {
        let started = self.begin_recovery(caller, home_id, host_awake, now_ms)?;
        for expected in (started.current_step + 1)..=7 {
            self.advance_recovery(caller, home_id, expected, now_ms)?;
        }
        let conn = self.lock()?;
        let recovery_id: String = conn
            .query_row(
                "SELECT recovery_id FROM p11_windows_host_recovery
                  WHERE home_id = ?1 ORDER BY started_at DESC LIMIT 1",
                [home_id],
                |row| row.get(0),
            )
            .map_err(unavailable("latest recovery"))?;
        drop(conn);
        self.load_recovery(&recovery_id)
    }

    /// Advance one recovery step. `expected_step` must be current+1.
    pub fn advance_recovery(
        &self,
        caller: ConfirmCaller,
        home_id: &str,
        expected_step: i64,
        now_ms: i64,
    ) -> Result<WindowsHostRecovery, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if !(1..=7).contains(&expected_step) {
            return Err(ProjectAggregateError::Invalid {
                detail: "recovery step skipped",
            });
        }
        let conn = self.lock()?;
        let (recovery_id, current_step, mut epoch, catch_up_asked): (String, i64, i64, i64) = conn
            .query_row(
                "SELECT recovery_id, current_step, epoch, catch_up_asked
                   FROM p11_windows_host_recovery
                  WHERE home_id = ?1 AND completed_at IS NULL
                  ORDER BY started_at DESC LIMIT 1",
                [home_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(unavailable("lookup open recovery"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "host not found",
            })?;
        if expected_step != current_step + 1 {
            return Err(ProjectAggregateError::Rejected {
                detail: "recovery step skipped",
            });
        }
        let mut next_catch_up = catch_up_asked;
        let mut resume_eligible = 0;
        let mut completed_at: Option<i64> = None;
        match expected_step {
            1 => {
                epoch += 1;
                conn.execute(
                    "UPDATE p11_windows_host_daemon SET epoch = ?1, updated_at = ?2
                      WHERE home_id = ?3",
                    params![epoch, now_ms, home_id],
                )
                .map_err(unavailable("fresh epoch"))?;
                conn.execute(
                    "UPDATE p11_windows_host_offline_segment
                        SET ended_at = ?1
                      WHERE home_id = ?2 AND ended_at IS NULL",
                    params![now_ms, home_id],
                )
                .map_err(unavailable("close offline"))?;
            }
            6 => next_catch_up = 1,
            7 => {
                if next_catch_up != 1 {
                    return Err(ProjectAggregateError::Rejected {
                        detail: "resume only eligible work",
                    });
                }
                resume_eligible = 1;
                completed_at = Some(now_ms);
                conn.execute(
                    "UPDATE p11_windows_host_daemon
                        SET state = 'resumed', updated_at = ?1
                      WHERE home_id = ?2",
                    params![now_ms, home_id],
                )
                .map_err(unavailable("resume daemon"))?;
            }
            _ => {}
        }
        conn.execute(
            "UPDATE p11_windows_host_recovery
                SET current_step = ?1, epoch = ?2, catch_up_asked = ?3,
                    resume_eligible = ?4, completed_at = ?5
              WHERE recovery_id = ?6",
            params![
                expected_step,
                epoch,
                next_catch_up,
                resume_eligible,
                completed_at,
                recovery_id
            ],
        )
        .map_err(unavailable("advance recovery"))?;
        drop(conn);
        self.load_recovery(&recovery_id)
    }

    /// Same-disk automatic versions are local restore points, not backups.
    pub fn record_restore_point(
        &self,
        caller: ConfirmCaller,
        home_id: &str,
        claimed_as_backup: bool,
        kind: &str,
        now_ms: i64,
    ) -> Result<String, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if claimed_as_backup || kind != "local-restore-point" {
            return Err(ProjectAggregateError::Rejected {
                detail: "restore-as-backup claim rejected",
            });
        }
        let conn = self.lock()?;
        require_home(&conn, home_id)?;
        let restore_point_id = next_id("restore")?;
        conn.execute(
            "INSERT INTO p11_windows_host_restore_point (
                restore_point_id, home_id, kind, same_disk, created_at
             ) VALUES (?1,?2,'local-restore-point',1,?3)",
            params![restore_point_id, home_id, now_ms],
        )
        .map_err(unavailable("insert restore point"))?;
        Ok(restore_point_id)
    }

    /// Tray/UI observation. Secret-shaped log text is rejected; status never echoes secrets.
    pub fn observe_status(
        &self,
        home_id: &str,
        log_line: Option<&str>,
    ) -> Result<WindowsHostStatus, ProjectAggregateError> {
        if let Some(line) = log_line {
            reject_secret_shape(line)?;
        }
        let conn = self.lock()?;
        let (install_root, app_dir, data_dir, data_preserved): (String, String, String, i64) = conn
            .query_row(
                "SELECT install_root, app_dir, data_dir, data_preserved
                   FROM p11_windows_host_home WHERE home_id = ?1",
                [home_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(unavailable("status home"))?
            .ok_or(ProjectAggregateError::NotFound {
                detail: "host not found",
            })?;
        let daemon: Option<(String, i64, String, i64, Option<String>)> = conn
            .query_row(
                "SELECT daemon_id, epoch, state, can_honor_background, close_disposition
                   FROM p11_windows_host_daemon WHERE home_id = ?1",
                [home_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(unavailable("status daemon"))?;
        let missed_segments: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM p11_windows_host_offline_segment WHERE home_id = ?1",
                [home_id],
                |row| row.get(0),
            )
            .map_err(unavailable("status missed"))?;
        let recovery: Option<(i64, i64)> = conn
            .query_row(
                "SELECT current_step, resume_eligible FROM p11_windows_host_recovery
                  WHERE home_id = ?1 ORDER BY started_at DESC LIMIT 1",
                [home_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("status recovery"))?;
        let restore_kind: Option<String> = conn
            .query_row(
                "SELECT kind FROM p11_windows_host_restore_point
                  WHERE home_id = ?1 ORDER BY created_at DESC LIMIT 1",
                [home_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("status restore"))?;
        Ok(WindowsHostStatus {
            home_id: home_id.to_owned(),
            install_root,
            app_dir,
            data_dir,
            data_preserved: data_preserved == 1,
            daemon_id: daemon.as_ref().map(|row| row.0.clone()),
            epoch: daemon.as_ref().map(|row| row.1).unwrap_or(0),
            daemon_state: daemon
                .as_ref()
                .map(|row| row.2.clone())
                .unwrap_or_else(|| "unbound".to_owned()),
            can_honor_background: daemon.as_ref().is_some_and(|row| row.3 == 1),
            tray_role: "observe-and-request".to_owned(),
            tray_proves_work: false,
            close_disposition: daemon.and_then(|row| row.4),
            missed_segments,
            recovery_step: recovery.map(|row| row.0).unwrap_or(0),
            resume_eligible: recovery.is_some_and(|row| row.1 == 1),
            restore_kind,
        })
    }

    fn load_recovery(
        &self,
        recovery_id: &str,
    ) -> Result<WindowsHostRecovery, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT recovery_id, home_id, epoch, current_step, catch_up_asked, resume_eligible
               FROM p11_windows_host_recovery WHERE recovery_id = ?1",
            [recovery_id],
            map_recovery,
        )
        .map_err(unavailable("load recovery"))
    }
}

fn map_recovery(row: &rusqlite::Row<'_>) -> rusqlite::Result<WindowsHostRecovery> {
    let current_step: i64 = row.get(3)?;
    let name = if current_step == 0 {
        "not-started".to_owned()
    } else {
        WAKE_RECOVERY_STEPS
            .get(usize::try_from(current_step - 1).unwrap_or(0))
            .unwrap_or(&"unknown")
            .to_string()
    };
    let catch_up: i64 = row.get(4)?;
    let eligible: i64 = row.get(5)?;
    Ok(WindowsHostRecovery {
        recovery_id: row.get(0)?,
        home_id: row.get(1)?,
        epoch: row.get(2)?,
        current_step,
        current_step_name: name,
        catch_up_asked: catch_up == 1,
        resume_eligible: eligible == 1,
    })
}

fn daemon_view(
    daemon_id: String,
    home_id: &str,
    epoch: i64,
    state: &str,
    can_honor_background: bool,
    close_disposition: Option<String>,
) -> WindowsHostDaemon {
    WindowsHostDaemon {
        daemon_id,
        home_id: home_id.to_owned(),
        epoch,
        state: state.to_owned(),
        can_honor_background,
        tray_role: "observe-and-request".to_owned(),
        tray_proves_work: false,
        close_disposition,
    }
}

fn require_home(conn: &Connection, home_id: &str) -> Result<(), ProjectAggregateError> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM p11_windows_host_home WHERE home_id = ?1",
            [home_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("require home"))?;
    if found.is_some() {
        Ok(())
    } else {
        Err(ProjectAggregateError::NotFound {
            detail: "host not found",
        })
    }
}

fn lookup_daemon_id(
    conn: &Connection,
    home_id: &str,
) -> Result<Option<String>, ProjectAggregateError> {
    conn.query_row(
        "SELECT daemon_id FROM p11_windows_host_daemon WHERE home_id = ?1",
        [home_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(unavailable("lookup daemon id"))
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn reject_wrong_install_root(install_root: &str) -> Result<(), ProjectAggregateError> {
    let normalized = normalize_path(install_root);
    if normalized.starts_with("/home/")
        || normalized.starts_with("/usr/")
        || normalized.starts_with("/var/")
        || normalized.contains("/wsl/")
        || normalized.contains("wsl.localhost")
        || normalized.contains("/.local/share/cognitiveos")
        || normalized.contains("\\wsl$")
    {
        return Err(ProjectAggregateError::Rejected {
            detail: "GNU/WSL/Linux is not a Windows product host",
        });
    }
    let trimmed = install_root.replace('\\', "/");
    let trimmed = trimmed.trim_end_matches('/');
    if !trimmed.ends_with("Personal Home") {
        return Err(ProjectAggregateError::Rejected {
            detail: "wrong install root rejected",
        });
    }
    Ok(())
}

fn reject_acl_escape(
    install_root: &str,
    app_dir: &str,
    data_dir: &str,
    acl_policy: &str,
) -> Result<(), ProjectAggregateError> {
    if acl_policy != "owner-only-dacl" {
        return Err(ProjectAggregateError::Rejected {
            detail: "ACL escape rejected",
        });
    }
    for candidate in [install_root, app_dir, data_dir] {
        if candidate.contains("..") {
            return Err(ProjectAggregateError::Rejected {
                detail: "ACL escape rejected",
            });
        }
    }
    let root = normalize_path(install_root);
    let app = normalize_path(app_dir);
    let data = normalize_path(data_dir);
    if app != format!("{root}/app") || data != format!("{root}/data") {
        return Err(ProjectAggregateError::Rejected {
            detail: "ACL escape rejected",
        });
    }
    Ok(())
}

fn require_offline_cause(cause: &str) -> Result<(), ProjectAggregateError> {
    match cause {
        "sleep" | "shutdown" | "daemon-stop" | "network-loss" | "secretstore-locked"
        | "provider-outage" => Ok(()),
        _ => Err(ProjectAggregateError::Invalid {
            detail: "offline cause is not recognized",
        }),
    }
}

fn reject_secret_material(
    argv: &[&str],
    env_pairs: &[(&str, &str)],
) -> Result<(), ProjectAggregateError> {
    for (key, value) in env_pairs {
        if secret_shaped_key(key) || secret_shaped_value(value) {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret must not enter env or argv",
            });
        }
    }
    for arg in argv {
        if secret_shaped_key(arg) || secret_shaped_value(arg) {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret must not enter env or argv",
            });
        }
    }
    Ok(())
}

fn reject_secret_shape(body: &str) -> Result<(), ProjectAggregateError> {
    if secret_shaped_key(body) || secret_shaped_value(body) {
        return Err(ProjectAggregateError::Invalid {
            detail: "secret-shaped material is rejected at registration",
        });
    }
    Ok(())
}

fn secret_shaped_key(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("password")
        || lowered.contains("authorization")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered.contains("api-key")
}

fn secret_shaped_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
}

fn next_id(prefix: &str) -> Result<String, ProjectAggregateError> {
    let generated = uuid::Uuid::now_v7().as_hyphenated().to_string();
    Ok(format!("{prefix}-{generated}"))
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
