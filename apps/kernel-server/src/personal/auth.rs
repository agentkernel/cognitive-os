//! Local channel-scoped session authority (P1-T04 / ADR-0019).

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::bounds::PersonalResourceBounds;

/// Task vs management channel class. Tokens never cross channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelClass {
    Task,
    Management,
}

impl ChannelClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Management => "management",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "task" => Some(Self::Task),
            "management" => Some(Self::Management),
            _ => None,
        }
    }
}

/// Fail-closed local auth outcomes. Messages never embed token material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalAuthError {
    BootstrapMissing,
    BootstrapMismatch,
    Unauthorized,
    ChannelBindingMismatch,
    SessionExpired,
    CookieAuthForbidden,
    Io { detail: &'static str },
    InvalidRequest { detail: &'static str },
}

impl LocalAuthError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BootstrapMissing => "LOCAL_BOOTSTRAP_MISSING",
            Self::BootstrapMismatch => "LOCAL_BOOTSTRAP_MISMATCH",
            Self::Unauthorized => "LOCAL_SESSION_UNAUTHORIZED",
            Self::ChannelBindingMismatch => "SHELL_CHANNEL_BINDING_MISMATCH",
            Self::SessionExpired => "LOCAL_SESSION_EXPIRED",
            Self::CookieAuthForbidden => "LOCAL_COOKIE_AUTH_FORBIDDEN",
            Self::Io { .. } => "LOCAL_AUTH_IO_FAILURE",
            Self::InvalidRequest { .. } => "LOCAL_AUTH_INVALID_REQUEST",
        }
    }
}

impl fmt::Display for LocalAuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BootstrapMissing => write!(formatter, "local bootstrap secret missing"),
            Self::BootstrapMismatch => write!(formatter, "local bootstrap secret mismatch"),
            Self::Unauthorized => write!(formatter, "local session unauthorized"),
            Self::ChannelBindingMismatch => write!(formatter, "channel binding mismatch"),
            Self::SessionExpired => write!(formatter, "local session expired"),
            Self::CookieAuthForbidden => write!(formatter, "cookie auth forbidden"),
            Self::Io { detail } => write!(formatter, "local auth I/O failure: {detail}"),
            Self::InvalidRequest { detail } => {
                write!(formatter, "invalid local auth request: {detail}")
            }
        }
    }
}

impl std::error::Error for LocalAuthError {}
/// Non-secret view of a minted session token for response JSON.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionTokenView {
    pub token: String,
    pub channel: ChannelClass,
    pub session_id: String,
    pub absolute_expiry_secs_from_now: u64,
    pub idle_expiry_secs_from_now: u64,
}

impl fmt::Debug for SessionTokenView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionTokenView")
            .field("token", &"[REDACTED]")
            .field("channel", &self.channel)
            .field("session_id", &self.session_id)
            .field(
                "absolute_expiry_secs_from_now",
                &self.absolute_expiry_secs_from_now,
            )
            .field("idle_expiry_secs_from_now", &self.idle_expiry_secs_from_now)
            .finish()
    }
}

/// Request to mint a channel-scoped session after bootstrap proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIssueRequest {
    pub channel: ChannelClass,
    pub principal_id: String,
    pub bootstrap_secret: String,
}

struct SessionRecord {
    channel: ChannelClass,
    principal_id: String,
    issued_at: Instant,
    last_seen_at: Instant,
    absolute_lifetime: Duration,
    idle_lifetime: Duration,
    epoch: u64,
}

/// In-process local session authority with private bootstrap secret file.
pub struct LocalSessionAuthority {
    bootstrap_secret_path: PathBuf,
    bootstrap_secret: String,
    bounds: PersonalResourceBounds,
    sessions: HashMap<String, SessionRecord>,
    epoch: u64,
    next_session_serial: u64,
}

impl fmt::Debug for LocalSessionAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LocalSessionAuthority")
            .field("bootstrap_secret_path", &self.bootstrap_secret_path)
            .field("bootstrap_secret", &"[REDACTED]")
            .field("session_count", &self.sessions.len())
            .field("epoch", &self.epoch)
            .finish()
    }
}
impl LocalSessionAuthority {
    /// Create authority, minting a bootstrap secret under the runtime tree.
    pub fn initialize(
        bootstrap_secret_path: impl Into<PathBuf>,
        bounds: PersonalResourceBounds,
    ) -> Result<Self, LocalAuthError> {
        let bootstrap_secret_path = bootstrap_secret_path.into();
        if let Some(parent) = bootstrap_secret_path.parent() {
            fs::create_dir_all(parent).map_err(|_| LocalAuthError::Io {
                detail: "failed to create bootstrap parent directory",
            })?;
            #[cfg(unix)]
            restrict_private_directory(parent)?;
        }
        let bootstrap_secret = generate_opaque_token("boot");
        write_private_file(&bootstrap_secret_path, bootstrap_secret.as_bytes())?;
        Ok(Self {
            bootstrap_secret_path,
            bootstrap_secret,
            bounds,
            sessions: HashMap::new(),
            epoch: 1,
            next_session_serial: 1,
        })
    }

    /// Load an existing bootstrap secret file (restart with preserved secret).
    pub fn load_existing(
        bootstrap_secret_path: impl Into<PathBuf>,
        bounds: PersonalResourceBounds,
    ) -> Result<Self, LocalAuthError> {
        let bootstrap_secret_path = bootstrap_secret_path.into();
        let bootstrap_secret = fs::read_to_string(&bootstrap_secret_path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                LocalAuthError::BootstrapMissing
            } else {
                LocalAuthError::Io {
                    detail: "failed to read bootstrap secret",
                }
            }
        })?;
        let bootstrap_secret = bootstrap_secret.trim().to_owned();
        if bootstrap_secret.is_empty() {
            return Err(LocalAuthError::BootstrapMissing);
        }
        Ok(Self {
            bootstrap_secret_path,
            bootstrap_secret,
            bounds,
            sessions: HashMap::new(),
            epoch: 1,
            next_session_serial: 1,
        })
    }

    pub fn bootstrap_secret_path(&self) -> &Path {
        &self.bootstrap_secret_path
    }

    /// Test/helper read of bootstrap material for hermetic fixtures only.
    pub fn bootstrap_secret_for_tests(&self) -> &str {
        &self.bootstrap_secret
    }

    /// Issue a channel-scoped bearer after bootstrap proof.
    pub fn issue_session(
        &mut self,
        request: SessionIssueRequest,
        now: Instant,
    ) -> Result<SessionTokenView, LocalAuthError> {
        if request.principal_id.is_empty() || request.principal_id.len() > 128 {
            return Err(LocalAuthError::InvalidRequest {
                detail: "principal_id length out of range",
            });
        }
        if !constant_time_eq(
            request.bootstrap_secret.as_bytes(),
            self.bootstrap_secret.as_bytes(),
        ) {
            return Err(LocalAuthError::BootstrapMismatch);
        }
        let token = generate_opaque_token("sess");
        let session_id = format!("sess-{}-{}", self.epoch, self.next_session_serial);
        self.next_session_serial = self.next_session_serial.saturating_add(1);
        let absolute_lifetime = Duration::from_secs(self.bounds.session_absolute_lifetime_secs);
        let idle_lifetime = Duration::from_secs(self.bounds.session_idle_lifetime_secs);
        self.sessions.insert(
            token.clone(),
            SessionRecord {
                channel: request.channel,
                principal_id: request.principal_id,
                issued_at: now,
                last_seen_at: now,
                absolute_lifetime,
                idle_lifetime,
                epoch: self.epoch,
            },
        );
        Ok(SessionTokenView {
            token,
            channel: request.channel,
            session_id,
            absolute_expiry_secs_from_now: self.bounds.session_absolute_lifetime_secs,
            idle_expiry_secs_from_now: self.bounds.session_idle_lifetime_secs,
        })
    }

    /// Authorize a bearer for the required channel. Touches idle expiry on success.
    pub fn authorize(
        &mut self,
        bearer_token: &str,
        required_channel: ChannelClass,
        now: Instant,
    ) -> Result<(), LocalAuthError> {
        if bearer_token.is_empty() {
            return Err(LocalAuthError::Unauthorized);
        }
        let Some(record) = self.sessions.get_mut(bearer_token) else {
            return Err(LocalAuthError::Unauthorized);
        };
        if record.channel != required_channel {
            return Err(LocalAuthError::ChannelBindingMismatch);
        }
        if now.duration_since(record.issued_at) > record.absolute_lifetime {
            return Err(LocalAuthError::SessionExpired);
        }
        if now.duration_since(record.last_seen_at) > record.idle_lifetime {
            return Err(LocalAuthError::SessionExpired);
        }
        if record.epoch != self.epoch {
            return Err(LocalAuthError::Unauthorized);
        }
        record.last_seen_at = now;
        let _ = &record.principal_id;
        Ok(())
    }

    /// Invalidate all outstanding tokens (restart / explicit revoke).
    pub fn revoke_all(&mut self) {
        self.sessions.clear();
        self.epoch = self.epoch.saturating_add(1);
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}
fn generate_opaque_token(prefix: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    prefix.hash(&mut hasher);
    Instant::now().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    format!("{prefix}-{:016x}-{:016x}", hasher.finish(), random_u64())
}

fn random_u64() -> u64 {
    let mut value = std::process::id() as u64;
    value ^= Instant::now().elapsed().as_nanos() as u64;
    value = value
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0xA24B_AED4_96E9_05C3);
    value
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut acc = 0u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        acc |= left_byte ^ right_byte;
    }
    acc == 0
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), LocalAuthError> {
    let mut file = fs::File::create(path).map_err(|_| LocalAuthError::Io {
        detail: "failed to create bootstrap secret file",
    })?;
    file.write_all(bytes).map_err(|_| LocalAuthError::Io {
        detail: "failed to write bootstrap secret file",
    })?;
    file.write_all(b"\n").map_err(|_| LocalAuthError::Io {
        detail: "failed to terminate bootstrap secret file",
    })?;
    #[cfg(unix)]
    restrict_private_file(path)?;
    Ok(())
}

#[cfg(unix)]
fn restrict_private_directory(path: &Path) -> Result<(), LocalAuthError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| LocalAuthError::Io {
        detail: "failed to set bootstrap directory mode",
    })
}

#[cfg(unix)]
fn restrict_private_file(path: &Path) -> Result<(), LocalAuthError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|_| LocalAuthError::Io {
        detail: "failed to set bootstrap secret mode",
    })
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn issue_and_authorize_same_channel() {
        let temp = std::env::temp_dir().join(format!("cos-auth-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let secret_path = temp.join("local-bootstrap.secret");
        let bounds = PersonalResourceBounds::personal_v1_baseline();
        let mut authority = LocalSessionAuthority::initialize(&secret_path, bounds).unwrap();
        let now = Instant::now();
        let secret = authority.bootstrap_secret_for_tests().to_owned();
        let view = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Management,
                    principal_id: "principal://local/owner".to_owned(),
                    bootstrap_secret: secret,
                },
                now,
            )
            .unwrap();
        assert!(authority
            .authorize(&view.token, ChannelClass::Management, now)
            .is_ok());
        assert!(matches!(
            authority.authorize(&view.token, ChannelClass::Task, now),
            Err(LocalAuthError::ChannelBindingMismatch)
        ));
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn wrong_bootstrap_fails_closed() {
        let temp = std::env::temp_dir().join(format!("cos-auth-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let secret_path = temp.join("local-bootstrap.secret");
        let bounds = PersonalResourceBounds::personal_v1_baseline();
        let mut authority = LocalSessionAuthority::initialize(&secret_path, bounds).unwrap();
        let err = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Task,
                    principal_id: "principal://local/owner".to_owned(),
                    bootstrap_secret: "wrong".to_owned(),
                },
                Instant::now(),
            )
            .unwrap_err();
        assert_eq!(err, LocalAuthError::BootstrapMismatch);
        let debug = format!("{:?}", authority);
        assert!(!debug.contains(authority.bootstrap_secret_for_tests()));
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn idle_expiry_and_revoke_all() {
        let temp = std::env::temp_dir().join(format!("cos-auth-idle-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let secret_path = temp.join("local-bootstrap.secret");
        let mut bounds = PersonalResourceBounds::personal_v1_baseline();
        bounds.session_idle_lifetime_secs = 1;
        bounds.session_absolute_lifetime_secs = 3600;
        let mut authority = LocalSessionAuthority::initialize(&secret_path, bounds).unwrap();
        let issued_at = Instant::now();
        let secret = authority.bootstrap_secret_for_tests().to_owned();
        let view = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Task,
                    principal_id: "principal://local/owner".to_owned(),
                    bootstrap_secret: secret.clone(),
                },
                issued_at,
            )
            .unwrap();
        let expired_at = issued_at + Duration::from_secs(2);
        assert!(matches!(
            authority.authorize(&view.token, ChannelClass::Task, expired_at),
            Err(LocalAuthError::SessionExpired)
        ));
        let view2 = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Task,
                    principal_id: "principal://local/owner".to_owned(),
                    bootstrap_secret: secret,
                },
                Instant::now(),
            )
            .unwrap();
        authority.revoke_all();
        assert!(matches!(
            authority.authorize(&view2.token, ChannelClass::Task, Instant::now()),
            Err(LocalAuthError::Unauthorized)
        ));
        let _ = fs::remove_dir_all(&temp);
    }
}