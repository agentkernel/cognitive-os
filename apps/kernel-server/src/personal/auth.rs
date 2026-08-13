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
#[derive(Clone, PartialEq, Eq)]
pub struct SessionIssueRequest {
    pub channel: ChannelClass,
    pub principal_id: String,
    pub bootstrap_secret: String,
}

impl fmt::Debug for SessionIssueRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionIssueRequest")
            .field("channel", &self.channel)
            .field("principal_id", &self.principal_id)
            .field("bootstrap_secret", &"[REDACTED]")
            .finish()
    }
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

const TOKEN_ENTROPY_BYTES: usize = 32;
const ENTROPY_PROBE_BYTES: usize = TOKEN_ENTROPY_BYTES * 2;

trait TokenEntropy {
    fn fill(&mut self, destination: &mut [u8]) -> Result<usize, LocalAuthError>;
}

struct OsTokenEntropy;

impl TokenEntropy for OsTokenEntropy {
    fn fill(&mut self, destination: &mut [u8]) -> Result<usize, LocalAuthError> {
        getrandom::fill(destination).map_err(|_| LocalAuthError::Io {
            detail: "operating system entropy unavailable",
        })?;
        Ok(destination.len())
    }
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
        let mut entropy = OsTokenEntropy;
        Self::initialize_with_entropy(bootstrap_secret_path, bounds, &mut entropy)
    }

    fn initialize_with_entropy(
        bootstrap_secret_path: impl Into<PathBuf>,
        bounds: PersonalResourceBounds,
        entropy: &mut dyn TokenEntropy,
    ) -> Result<Self, LocalAuthError> {
        let bootstrap_secret_path = bootstrap_secret_path.into();
        let bootstrap_secret = generate_opaque_token("boot", entropy)?;
        if let Some(parent) = bootstrap_secret_path.parent() {
            fs::create_dir_all(parent).map_err(|_| LocalAuthError::Io {
                detail: "failed to create bootstrap parent directory",
            })?;
            #[cfg(unix)]
            restrict_private_directory(parent)?;
        }
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
    #[cfg(test)]
    pub fn bootstrap_secret_for_tests(&self) -> &str {
        &self.bootstrap_secret
    }

    /// Issue a channel-scoped bearer after bootstrap proof.
    pub fn issue_session(
        &mut self,
        request: SessionIssueRequest,
        now: Instant,
    ) -> Result<SessionTokenView, LocalAuthError> {
        let mut entropy = OsTokenEntropy;
        self.issue_session_with_entropy(request, now, &mut entropy)
    }

    fn issue_session_with_entropy(
        &mut self,
        request: SessionIssueRequest,
        now: Instant,
        entropy: &mut dyn TokenEntropy,
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
        let token = generate_opaque_token("sess", entropy)?;
        if self.sessions.contains_key(&token) {
            return Err(LocalAuthError::Io {
                detail: "entropy source repeated an existing session token",
            });
        }
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
        self.authorize_principal(bearer_token, required_channel, now)
            .map(|_| ())
    }

    /// Authorize a bearer and return its authenticated principal identity.
    ///
    /// Task governance derives its actor and acceptance authority from this
    /// value; clients must never be trusted to repeat or select it.
    pub fn authorize_principal(
        &mut self,
        bearer_token: &str,
        required_channel: ChannelClass,
        now: Instant,
    ) -> Result<String, LocalAuthError> {
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
        Ok(record.principal_id.clone())
    }

    /// Authorize the owner-only local daemon-administration boundary.
    ///
    /// A session reaches this boundary only after the runtime bootstrap secret
    /// minted a management-channel bearer. Task, Pi, CLI, and worker bearers
    /// remain unable to use it even when they carry the same principal text.
    /// Callers use this before admitting immutable daemon-owned authorization
    /// or revocation facts; it grants no read/write capability by itself.
    pub fn authorize_daemon_administrator(
        &mut self,
        bearer_token: &str,
        now: Instant,
    ) -> Result<String, LocalAuthError> {
        self.authorize_principal(bearer_token, ChannelClass::Management, now)
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
fn generate_opaque_token(
    prefix: &str,
    entropy: &mut dyn TokenEntropy,
) -> Result<String, LocalAuthError> {
    let mut sample = [0u8; ENTROPY_PROBE_BYTES];
    let written = match entropy.fill(&mut sample) {
        Ok(written) => written,
        Err(error) => {
            sample.fill(0);
            return Err(error);
        }
    };
    if written != sample.len() {
        sample.fill(0);
        return Err(LocalAuthError::Io {
            detail: "entropy source returned a short sample",
        });
    }

    let (token_bytes, independent_probe) = sample.split_at(TOKEN_ENTROPY_BYTES);
    let has_zero_block = token_bytes.iter().all(|byte| *byte == 0)
        || independent_probe.iter().all(|byte| *byte == 0);
    if has_zero_block || constant_time_eq(token_bytes, independent_probe) {
        sample.fill(0);
        return Err(LocalAuthError::Io {
            detail: "entropy source returned a zero or repeated block",
        });
    }

    let encoded = encode_lower_hex(token_bytes);
    sample.fill(0);
    let midpoint = encoded.len() / 2;
    Ok(format!(
        "{prefix}-{}-{}",
        &encoded[..midpoint],
        &encoded[midpoint..]
    ))
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
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
    use std::{collections::HashSet, time::Duration};

    struct FailingEntropy;

    impl TokenEntropy for FailingEntropy {
        fn fill(&mut self, _destination: &mut [u8]) -> Result<usize, LocalAuthError> {
            Err(LocalAuthError::Io {
                detail: "injected entropy failure",
            })
        }
    }

    struct ShortEntropy {
        written: usize,
    }

    impl TokenEntropy for ShortEntropy {
        fn fill(&mut self, destination: &mut [u8]) -> Result<usize, LocalAuthError> {
            destination.fill(0xa5);
            Ok(self.written)
        }
    }

    struct RepeatedBlockEntropy;

    impl TokenEntropy for RepeatedBlockEntropy {
        fn fill(&mut self, destination: &mut [u8]) -> Result<usize, LocalAuthError> {
            for (index, byte) in destination.iter_mut().enumerate() {
                *byte = (index % TOKEN_ENTROPY_BYTES) as u8;
            }
            Ok(destination.len())
        }
    }

    struct DistinctBlockEntropy;

    impl TokenEntropy for DistinctBlockEntropy {
        fn fill(&mut self, destination: &mut [u8]) -> Result<usize, LocalAuthError> {
            for (index, byte) in destination.iter_mut().enumerate() {
                *byte = index as u8;
            }
            Ok(destination.len())
        }
    }

    /// 全长样本，但指定半区填零，用于证明零块在写文件前被拒绝。
    struct ZeroBlockEntropy {
        zero_token_half: bool,
        zero_probe_half: bool,
    }

    impl TokenEntropy for ZeroBlockEntropy {
        fn fill(&mut self, destination: &mut [u8]) -> Result<usize, LocalAuthError> {
            for (index, byte) in destination.iter_mut().enumerate() {
                *byte = (index as u8).saturating_add(1);
            }
            if self.zero_token_half {
                destination[..TOKEN_ENTROPY_BYTES].fill(0);
            }
            if self.zero_probe_half {
                destination[TOKEN_ENTROPY_BYTES..].fill(0);
            }
            Ok(destination.len())
        }
    }

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
        assert!(
            authority
                .authorize(&view.token, ChannelClass::Management, now)
                .is_ok()
        );
        assert!(matches!(
            authority.authorize(&view.token, ChannelClass::Task, now),
            Err(LocalAuthError::ChannelBindingMismatch)
        ));
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn only_management_session_can_enter_daemon_administration_boundary() {
        let temp = std::env::temp_dir().join(format!("cos-auth-admin-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let secret_path = temp.join("local-bootstrap.secret");
        let bounds = PersonalResourceBounds::personal_v1_baseline();
        let mut authority = LocalSessionAuthority::initialize(&secret_path, bounds).unwrap();
        let now = Instant::now();
        let secret = authority.bootstrap_secret_for_tests().to_owned();
        let management_session = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Management,
                    principal_id: "principal://local/owner".to_owned(),
                    bootstrap_secret: secret.clone(),
                },
                now,
            )
            .unwrap();
        let task_session = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Task,
                    principal_id: "principal://local/owner".to_owned(),
                    bootstrap_secret: secret,
                },
                now,
            )
            .unwrap();

        assert_eq!(
            authority
                .authorize_daemon_administrator(&management_session.token, now)
                .unwrap(),
            "principal://local/owner"
        );
        assert!(matches!(
            authority.authorize_daemon_administrator(&task_session.token, now),
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

    #[test]
    fn entropy_failure_creates_no_bootstrap_file() {
        let temp =
            std::env::temp_dir().join(format!("cos-auth-entropy-fail-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let secret_path = temp.join("local-bootstrap.secret");
        let mut entropy = FailingEntropy;

        let result = LocalSessionAuthority::initialize_with_entropy(
            &secret_path,
            PersonalResourceBounds::personal_v1_baseline(),
            &mut entropy,
        );

        assert!(matches!(result, Err(LocalAuthError::Io { .. })));
        assert!(!secret_path.exists(), "熵失败不得创建 bootstrap 文件");
        assert!(!temp.exists(), "熵失败不得创建 runtime 目录");
    }

    #[test]
    fn zero_and_short_entropy_create_no_bootstrap_file() {
        for written in [
            0,
            TOKEN_ENTROPY_BYTES - 1,
            TOKEN_ENTROPY_BYTES,
            ENTROPY_PROBE_BYTES - 1,
        ] {
            let temp = std::env::temp_dir().join(format!(
                "cos-auth-entropy-short-{}-{written}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&temp);
            let secret_path = temp.join("local-bootstrap.secret");
            let mut entropy = ShortEntropy { written };

            let result = LocalSessionAuthority::initialize_with_entropy(
                &secret_path,
                PersonalResourceBounds::personal_v1_baseline(),
                &mut entropy,
            );

            assert!(matches!(result, Err(LocalAuthError::Io { .. })));
            assert!(!secret_path.exists(), "短熵不得创建 bootstrap 文件");
            assert!(!temp.exists(), "短熵不得创建 runtime 目录");
        }
    }

    #[test]
    fn zero_entropy_block_creates_no_bootstrap_file() {
        for (zero_token_half, zero_probe_half) in [(true, true), (true, false), (false, true)] {
            let temp = std::env::temp_dir().join(format!(
                "cos-auth-entropy-zero-{}-{}-{}",
                std::process::id(),
                u8::from(zero_token_half),
                u8::from(zero_probe_half)
            ));
            let _ = fs::remove_dir_all(&temp);
            let secret_path = temp.join("local-bootstrap.secret");
            let mut entropy = ZeroBlockEntropy {
                zero_token_half,
                zero_probe_half,
            };

            let result = LocalSessionAuthority::initialize_with_entropy(
                &secret_path,
                PersonalResourceBounds::personal_v1_baseline(),
                &mut entropy,
            );

            assert!(matches!(result, Err(LocalAuthError::Io { .. })));
            assert!(!secret_path.exists(), "零熵块不得创建 bootstrap 文件");
            assert!(!temp.exists(), "零熵块不得创建 runtime 目录");
        }
    }

    #[test]
    fn repeated_entropy_block_is_rejected_before_bootstrap_creation() {
        let temp =
            std::env::temp_dir().join(format!("cos-auth-entropy-repeat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let secret_path = temp.join("local-bootstrap.secret");
        let mut entropy = RepeatedBlockEntropy;

        let result = LocalSessionAuthority::initialize_with_entropy(
            &secret_path,
            PersonalResourceBounds::personal_v1_baseline(),
            &mut entropy,
        );

        assert!(matches!(result, Err(LocalAuthError::Io { .. })));
        assert!(!secret_path.exists(), "重复熵不得创建 bootstrap 文件");
        assert!(!temp.exists(), "重复熵不得创建 runtime 目录");
    }

    #[test]
    fn session_entropy_failure_creates_no_session_or_token_view() {
        let temp =
            std::env::temp_dir().join(format!("cos-auth-session-fail-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let secret_path = temp.join("local-bootstrap.secret");
        let mut bootstrap_entropy = DistinctBlockEntropy;
        let mut authority = LocalSessionAuthority::initialize_with_entropy(
            &secret_path,
            PersonalResourceBounds::personal_v1_baseline(),
            &mut bootstrap_entropy,
        )
        .unwrap();
        let bootstrap_secret = authority.bootstrap_secret_for_tests().to_owned();
        let mut session_entropy = FailingEntropy;

        let result = authority.issue_session_with_entropy(
            SessionIssueRequest {
                channel: ChannelClass::Task,
                principal_id: "principal://local/owner".to_owned(),
                bootstrap_secret,
            },
            Instant::now(),
            &mut session_entropy,
        );

        assert!(matches!(result, Err(LocalAuthError::Io { .. })));
        assert_eq!(authority.session_count(), 0);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn deterministic_session_entropy_cannot_repeat_a_token() {
        let temp =
            std::env::temp_dir().join(format!("cos-auth-session-repeat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let secret_path = temp.join("local-bootstrap.secret");
        let mut bootstrap_entropy = DistinctBlockEntropy;
        let mut authority = LocalSessionAuthority::initialize_with_entropy(
            &secret_path,
            PersonalResourceBounds::personal_v1_baseline(),
            &mut bootstrap_entropy,
        )
        .unwrap();
        let bootstrap_secret = authority.bootstrap_secret_for_tests().to_owned();
        let mut session_entropy = DistinctBlockEntropy;
        let request = || SessionIssueRequest {
            channel: ChannelClass::Task,
            principal_id: "principal://local/owner".to_owned(),
            bootstrap_secret: bootstrap_secret.clone(),
        };

        let first =
            authority.issue_session_with_entropy(request(), Instant::now(), &mut session_entropy);
        let second =
            authority.issue_session_with_entropy(request(), Instant::now(), &mut session_entropy);

        assert!(first.is_ok());
        assert!(matches!(second, Err(LocalAuthError::Io { .. })));
        assert_eq!(authority.session_count(), 1);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn bounded_os_rng_sample_produces_unique_opaque_token_shapes() {
        let temp = std::env::temp_dir().join(format!("cos-auth-unique-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let secret_path = temp.join("local-bootstrap.secret");
        let mut authority = LocalSessionAuthority::initialize(
            &secret_path,
            PersonalResourceBounds::personal_v1_baseline(),
        )
        .unwrap();
        let bootstrap_secret = authority.bootstrap_secret_for_tests().to_owned();
        let mut observed = HashSet::new();

        for _ in 0..128 {
            let view = authority
                .issue_session(
                    SessionIssueRequest {
                        channel: ChannelClass::Task,
                        principal_id: "principal://local/owner".to_owned(),
                        bootstrap_secret: bootstrap_secret.clone(),
                    },
                    Instant::now(),
                )
                .unwrap();
            let parts = view.token.split('-').collect::<Vec<_>>();
            assert!(
                parts.len() == 3
                    && parts[0] == "sess"
                    && parts[1].len() == TOKEN_ENTROPY_BYTES
                    && parts[2].len() == TOKEN_ENTROPY_BYTES
                    && parts[1..]
                        .iter()
                        .all(|part| part.bytes().all(|byte| byte.is_ascii_hexdigit())),
                "令牌必须保留 prefix-group-group opaque 形状"
            );
            assert!(
                observed.insert(view.token),
                "有界样本不得产生重复令牌；此断言不构成统计随机性声明"
            );
        }

        assert_eq!(observed.len(), 128);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn debug_and_log_serialization_redact_token_material() {
        let temp = std::env::temp_dir().join(format!("cos-auth-redaction-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let secret_path = temp.join("local-bootstrap.secret");
        let mut authority = LocalSessionAuthority::initialize(
            &secret_path,
            PersonalResourceBounds::personal_v1_baseline(),
        )
        .unwrap();
        let bootstrap_secret = authority.bootstrap_secret_for_tests().to_owned();
        let view = authority
            .issue_session(
                SessionIssueRequest {
                    channel: ChannelClass::Management,
                    principal_id: "principal://local/owner".to_owned(),
                    bootstrap_secret,
                },
                Instant::now(),
            )
            .unwrap();
        let token = view.token.clone();
        let debug = format!("{view:?}");
        let serialized_log = serde_json::json!({ "session": debug }).to_string();

        assert!(!serialized_log.contains(&token));
        assert!(serialized_log.contains("REDACTED"));
        let authority_debug = format!("{authority:?}");
        assert!(!authority_debug.contains(authority.bootstrap_secret_for_tests()));
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn session_issue_request_debug_redacts_bootstrap_secret() {
        let bootstrap_secret = ["fixture-bootstrap-", "material-never-log"].concat();
        let request = SessionIssueRequest {
            channel: ChannelClass::Management,
            principal_id: "principal://local/owner".to_owned(),
            bootstrap_secret: bootstrap_secret.clone(),
        };

        let debug = format!("{request:?}");

        assert!(!debug.contains(&bootstrap_secret));
        assert!(debug.contains("[REDACTED]"));
    }

    #[cfg(unix)]
    #[test]
    fn bootstrap_file_keeps_private_mode() {
        use std::os::unix::fs::PermissionsExt;

        let temp = std::env::temp_dir().join(format!("cos-auth-mode-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let secret_path = temp.join("local-bootstrap.secret");
        let _authority = LocalSessionAuthority::initialize(
            &secret_path,
            PersonalResourceBounds::personal_v1_baseline(),
        )
        .unwrap();

        let mode = fs::metadata(&secret_path).unwrap().permissions().mode() & 0o777;
        assert!(mode == 0o600, "bootstrap 文件必须保持 0600");
        let _ = fs::remove_dir_all(&temp);
    }
}
