#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! P11-T02 D01: Windows host Personal Home / lifecycle / missed / recovery.
//!
//! Failure-first: wrong install root, ACL escape, raw secret env/argv,
//! duplicate daemon, orphan DSH, fake background, restore-as-backup,
//! secret-shaped logs. Green path proves app/data upgrade preserve, close
//! honesty, offline missed facts, and ordered seven-step wake recovery.
//! Native tray/ACL/sleep E2E is not claimed here.

use cognitive_store::{
    ConfirmCaller, DaemonBindSpec, HomeAdmitSpec, PersonalDataLayout, ProjectAggregateError,
    SqliteAuthorityStore, WindowsHostStore, prepare_personal_databases,
};
use tempfile::TempDir;

fn stores() -> (TempDir, SqliteAuthorityStore, WindowsHostStore) {
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
    let authority = SqliteAuthorityStore::open(&path).expect("authority");
    let host = WindowsHostStore::from_authority_store(&authority);
    (temporary, authority, host)
}

fn valid_root() -> &'static str {
    r"C:\Users\owner\Personal Home"
}

fn valid_app() -> &'static str {
    r"C:\Users\owner\Personal Home\app"
}

fn valid_data() -> &'static str {
    r"C:\Users\owner\Personal Home\data"
}

fn admit(host: &WindowsHostStore) -> String {
    host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root: valid_root(),
            app_dir: valid_app(),
            data_dir: valid_data(),
            acl_policy: "owner-only-dacl",
            argv: &["--personal"],
            env_pairs: &[("PATH", r"C:\Windows\System32")],
            now_ms: 10,
        },
    )
    .expect("admit")
    .home_id
}

#[test]
fn p11_t02_wrong_install_root_is_rejected() {
    let (_tmp, _authority, host) = stores();
    let program_files = host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root: r"C:\Program Files\CognitiveOS",
            app_dir: r"C:\Program Files\CognitiveOS\app",
            data_dir: r"C:\Program Files\CognitiveOS\data",
            acl_policy: "owner-only-dacl",
            argv: &[],
            env_pairs: &[],
            now_ms: 11,
        },
    );
    match program_files {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("wrong install root"), "{detail}");
        }
        other => panic!("expected wrong install root, got {other:?}"),
    }
    let linux = host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root: "/home/wuz/.local/share/cognitiveos",
            app_dir: "/home/wuz/.local/share/cognitiveos/app",
            data_dir: "/home/wuz/.local/share/cognitiveos/data",
            acl_policy: "owner-only-dacl",
            argv: &[],
            env_pairs: &[],
            now_ms: 12,
        },
    );
    match linux {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(
                detail.contains("GNU/WSL/Linux is not a Windows product host"),
                "{detail}"
            );
        }
        other => panic!("expected linux-as-windows reject, got {other:?}"),
    }
}

#[test]
fn p11_t02_acl_escape_is_rejected() {
    let (_tmp, _authority, host) = stores();
    let traversal = host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root: valid_root(),
            app_dir: r"C:\Users\owner\Personal Home\..\Windows\app",
            data_dir: valid_data(),
            acl_policy: "owner-only-dacl",
            argv: &[],
            env_pairs: &[],
            now_ms: 20,
        },
    );
    match traversal {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("ACL escape"), "{detail}");
        }
        other => panic!("expected ACL escape, got {other:?}"),
    }
    let world = host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root: valid_root(),
            app_dir: valid_app(),
            data_dir: valid_data(),
            acl_policy: "everyone-full-control",
            argv: &[],
            env_pairs: &[],
            now_ms: 21,
        },
    );
    match world {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("ACL escape"), "{detail}");
        }
        other => panic!("expected world ACL reject, got {other:?}"),
    }
}

#[test]
fn p11_t02_raw_secret_env_argv_is_rejected() {
    let (_tmp, _authority, host) = stores();
    let env = host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root: valid_root(),
            app_dir: valid_app(),
            data_dir: valid_data(),
            acl_policy: "owner-only-dacl",
            argv: &["--personal"],
            env_pairs: &[("OPENAI_API_KEY", "sk-test-leak")],
            now_ms: 30,
        },
    );
    match env {
        Err(ProjectAggregateError::Invalid { detail }) => {
            assert!(
                detail.contains("secret must not enter env or argv"),
                "{detail}"
            );
        }
        other => panic!("expected env secret reject, got {other:?}"),
    }
    let argv = host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root: valid_root(),
            app_dir: valid_app(),
            data_dir: valid_data(),
            acl_policy: "owner-only-dacl",
            argv: &["--token", "ssv1:abc"],
            env_pairs: &[],
            now_ms: 31,
        },
    );
    match argv {
        Err(ProjectAggregateError::Invalid { detail }) => {
            assert!(
                detail.contains("secret must not enter env or argv"),
                "{detail}"
            );
        }
        other => panic!("expected argv secret reject, got {other:?}"),
    }
}

#[test]
fn p11_t02_duplicate_daemon_is_rejected() {
    let (_tmp, _authority, host) = stores();
    let home_id = admit(&host);
    host.bind_daemon(
        ConfirmCaller::OwnerManagement,
        &DaemonBindSpec {
            home_id: &home_id,
            can_honor_background: true,
            now_ms: 40,
        },
    )
    .expect("first bind");
    let duplicate = host.bind_daemon(
        ConfirmCaller::OwnerManagement,
        &DaemonBindSpec {
            home_id: &home_id,
            can_honor_background: true,
            now_ms: 41,
        },
    );
    match duplicate {
        Err(ProjectAggregateError::Conflict { detail }) => {
            assert!(detail.contains("duplicate daemon"), "{detail}");
        }
        other => panic!("expected duplicate daemon, got {other:?}"),
    }
}

#[test]
fn p11_t02_orphan_dsh_is_rejected() {
    let (_tmp, _authority, host) = stores();
    let home_id = admit(&host);
    let orphan = host.bind_dsh_child(ConfirmCaller::OwnerManagement, &home_id, 50);
    match orphan {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("orphan DSH"), "{detail}");
        }
        other => panic!("expected orphan DSH, got {other:?}"),
    }
    host.bind_daemon(
        ConfirmCaller::OwnerManagement,
        &DaemonBindSpec {
            home_id: &home_id,
            can_honor_background: false,
            now_ms: 51,
        },
    )
    .expect("bind");
    let child = host
        .bind_dsh_child(ConfirmCaller::OwnerManagement, &home_id, 52)
        .expect("child");
    assert!(child.starts_with("dshchild-"));
    host.record_offline(ConfirmCaller::OwnerManagement, &home_id, "daemon-stop", 53)
        .expect("offline");
    assert_eq!(host.orphaned_dsh_count(&home_id).expect("count"), 1);
    let after_stop = host.bind_dsh_child(ConfirmCaller::OwnerManagement, &home_id, 54);
    match after_stop {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("orphan DSH"), "{detail}");
        }
        other => panic!("expected orphan after stop, got {other:?}"),
    }
}

#[test]
fn p11_t02_fake_background_is_rejected() {
    let (_tmp, _authority, host) = stores();
    let home_id = admit(&host);
    host.bind_daemon(
        ConfirmCaller::OwnerManagement,
        &DaemonBindSpec {
            home_id: &home_id,
            can_honor_background: false,
            now_ms: 60,
        },
    )
    .expect("bind");
    let fake = host.request_close(
        ConfirmCaller::OwnerManagement,
        &cognitive_store::CloseRequestSpec {
            home_id: &home_id,
            choice: "background",
            now_ms: 61,
        },
    );
    match fake {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("fake background"), "{detail}");
        }
        other => panic!("expected fake background, got {other:?}"),
    }
    let paused = host
        .request_close(
            ConfirmCaller::OwnerManagement,
            &cognitive_store::CloseRequestSpec {
                home_id: &home_id,
                choice: "pause",
                now_ms: 62,
            },
        )
        .expect("pause");
    assert_eq!(paused.state, "paused");
    assert_eq!(paused.close_disposition.as_deref(), Some("paused"));
    assert!(!paused.tray_proves_work);
}

#[test]
fn p11_t02_restore_as_backup_claim_is_rejected() {
    let (_tmp, _authority, host) = stores();
    let home_id = admit(&host);
    let backup = host.record_restore_point(
        ConfirmCaller::OwnerManagement,
        &home_id,
        true,
        "local-restore-point",
        70,
    );
    match backup {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("restore-as-backup"), "{detail}");
        }
        other => panic!("expected restore-as-backup, got {other:?}"),
    }
    let disaster = host.record_restore_point(
        ConfirmCaller::OwnerManagement,
        &home_id,
        false,
        "disaster-backup",
        71,
    );
    match disaster {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("restore-as-backup"), "{detail}");
        }
        other => panic!("expected disaster-backup reject, got {other:?}"),
    }
}

#[test]
fn p11_t02_secrets_are_not_in_status_or_logs() {
    let (_tmp, _authority, host) = stores();
    let home_id = admit(&host);
    let leaked = host.observe_status(&home_id, Some("bearer sk-leaked"));
    match leaked {
        Err(ProjectAggregateError::Invalid { detail }) => {
            assert!(detail.contains("secret-shaped"), "{detail}");
        }
        other => panic!("expected secret log reject, got {other:?}"),
    }
    let status = host
        .observe_status(&home_id, Some("daemon bound"))
        .expect("status");
    let encoded = format!("{status:?}");
    assert!(!encoded.contains("sk-"));
    assert!(!encoded.contains("ssv1:"));
    assert!(!encoded.contains("bearer "));
    assert!(!status.tray_proves_work);
    assert_eq!(status.tray_role, "observe-and-request");
}

#[test]
fn p11_t02_upgrade_offline_and_ordered_recovery() {
    let (_tmp, _authority, host) = stores();
    let first = host
        .admit_home(
            ConfirmCaller::OwnerManagement,
            &HomeAdmitSpec {
                install_root: valid_root(),
                app_dir: valid_app(),
                data_dir: valid_data(),
                acl_policy: "owner-only-dacl",
                argv: &["--personal"],
                env_pairs: &[],
                now_ms: 80,
            },
        )
        .expect("admit");
    let upgraded = host
        .admit_home(
            ConfirmCaller::OwnerManagement,
            &HomeAdmitSpec {
                install_root: valid_root(),
                app_dir: valid_app(),
                data_dir: valid_data(),
                acl_policy: "owner-only-dacl",
                argv: &["--personal"],
                env_pairs: &[],
                now_ms: 81,
            },
        )
        .expect("upgrade");
    assert_eq!(upgraded.home_id, first.home_id);
    assert!(upgraded.data_preserved);
    assert!(upgraded.app_replaced);

    let daemon = host
        .bind_daemon(
            ConfirmCaller::OwnerManagement,
            &DaemonBindSpec {
                home_id: &first.home_id,
                can_honor_background: true,
                now_ms: 82,
            },
        )
        .expect("bind");
    assert_eq!(daemon.epoch, 1);
    let closed = host
        .request_close(
            ConfirmCaller::OwnerManagement,
            &cognitive_store::CloseRequestSpec {
                home_id: &first.home_id,
                choice: "background",
                now_ms: 83,
            },
        )
        .expect("background honor");
    assert_eq!(
        closed.close_disposition.as_deref(),
        Some("background-honored")
    );
    assert!(!closed.tray_proves_work);

    host.record_offline(ConfirmCaller::OwnerManagement, &first.home_id, "sleep", 84)
        .expect("sleep");
    let asleep =
        host.run_ordered_recovery(ConfirmCaller::OwnerManagement, &first.home_id, false, 85);
    match asleep {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("host is off"), "{detail}");
        }
        other => panic!("expected host-off reject, got {other:?}"),
    }

    let begun = host
        .begin_recovery(ConfirmCaller::OwnerManagement, &first.home_id, true, 86)
        .expect("begin");
    assert_eq!(begun.current_step, 0);
    let skip = host.advance_recovery(ConfirmCaller::OwnerManagement, &first.home_id, 3, 87);
    match skip {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("recovery step skipped"), "{detail}");
        }
        other => panic!("expected skip reject, got {other:?}"),
    }
    host.advance_recovery(ConfirmCaller::OwnerManagement, &first.home_id, 1, 88)
        .expect("step 1");
    let skip_later = host.advance_recovery(ConfirmCaller::OwnerManagement, &first.home_id, 4, 89);
    match skip_later {
        Err(ProjectAggregateError::Rejected { detail }) => {
            assert!(detail.contains("recovery step skipped"), "{detail}");
        }
        other => panic!("expected later skip reject, got {other:?}"),
    }

    let recovered = host
        .run_ordered_recovery(ConfirmCaller::OwnerManagement, &first.home_id, true, 90)
        .expect("recovery");
    assert_eq!(recovered.current_step, 7);
    assert_eq!(recovered.current_step_name, "resume-eligible-only");
    assert!(recovered.catch_up_asked);
    assert!(recovered.resume_eligible);
    assert!(recovered.epoch >= 2);

    let restore = host
        .record_restore_point(
            ConfirmCaller::OwnerManagement,
            &first.home_id,
            false,
            "local-restore-point",
            91,
        )
        .expect("restore point");
    assert!(restore.starts_with("restore-"));
    let status = host
        .observe_status(&first.home_id, None)
        .expect("final status");
    assert_eq!(status.restore_kind.as_deref(), Some("local-restore-point"));
    assert!(status.missed_segments >= 1);
    assert_eq!(status.recovery_step, 7);
    assert!(status.resume_eligible);
    assert!(!status.tray_proves_work);
}
