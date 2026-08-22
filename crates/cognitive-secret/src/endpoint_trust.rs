//! Endpoint trust policy for the LLM Provider Control Plane (P8-T13).
//!
//! Std-only: parse and classify a caller-supplied endpoint before any Secret
//! Store read or network I/O. Official OpenAI/Anthropic URLs are immutable.
//! Custom endpoints are OpenAI-compatible only. This module is not an
//! authority writer and never sees API key material.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Official OpenAI API root. Callers cannot substitute another host.
pub const OPENAI_OFFICIAL_ENDPOINT: &str = "https://api.openai.com/v1";
/// Official Anthropic API root. Callers cannot substitute another host.
pub const ANTHROPIC_OFFICIAL_ENDPOINT: &str = "https://api.anthropic.com";

/// Supported account kinds. There is no third-party Anthropic-compatible kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// `https://api.openai.com/v1` only.
    OpenaiOfficial,
    /// `https://api.anthropic.com` only.
    AnthropicOfficial,
    /// Caller-supplied OpenAI-compatible HTTPS or explicitly granted HTTP.
    OpenaiCompatible,
}

impl ProviderKind {
    /// Parse a wire/CLI kind token. Unknown and Anthropic-compatible custom
    /// tokens fail closed.
    pub fn parse(token: &str) -> Result<Self, EndpointTrustError> {
        match token {
            "openai_official" => Ok(Self::OpenaiOfficial),
            "anthropic_official" => Ok(Self::AnthropicOfficial),
            "openai_compatible" => Ok(Self::OpenaiCompatible),
            "anthropic_compatible" | "anthropic-compatible" | "anthropic_custom" => {
                Err(EndpointTrustError::UnsupportedAnthropicCompatible)
            }
            _ => Err(EndpointTrustError::UnsupportedKind),
        }
    }

    /// Stable wire token.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenaiOfficial => "openai_official",
            Self::AnthropicOfficial => "anthropic_official",
            Self::OpenaiCompatible => "openai_compatible",
        }
    }
}

/// How private the endpoint (or a resolved address) is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetworkScope {
    /// `127.0.0.0/8`, `::1`, `localhost`.
    Loopback = 0,
    /// RFC1918, ULA, link-local, CGNAT, unspecified.
    Private = 1,
    /// Everything else, including public DNS names before resolution.
    Public = 2,
}

impl NetworkScope {
    /// Stable wire token.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Loopback => "loopback",
            Self::Private => "private",
            Self::Public => "public",
        }
    }

    fn parse(token: &str) -> Result<Self, EndpointTrustError> {
        match token {
            "loopback" => Ok(Self::Loopback),
            "private" | "lan" => Ok(Self::Private),
            "public" => Ok(Self::Public),
            _ => Err(EndpointTrustError::Invalid),
        }
    }
}

/// Durable grants recorded on a custom account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointTrustGrant {
    /// Permit loopback / LAN / private / link-local targets.
    pub allow_private_network: bool,
    /// Permit `http://` (never implied by the private-network grant).
    pub allow_insecure_http: bool,
}

/// A normalized, policy-accepted endpoint. Safe to persist (no userinfo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedEndpoint {
    kind: ProviderKind,
    normalized: String,
    scheme: EndpointScheme,
    host: String,
    port: u16,
    path: String,
    scope: NetworkScope,
    grant: EndpointTrustGrant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EndpointScheme {
    Https,
    Http,
}

impl EndpointScheme {
    fn as_str(self) -> &'static str {
        match self {
            Self::Https => "https",
            Self::Http => "http",
        }
    }
}

/// Fail-closed endpoint policy outcomes. Messages never contain secrets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointTrustError {
    /// Empty, oversized, or structurally unparseable URL.
    Invalid,
    /// URL userinfo is present (`https://user:pass@host`).
    EmbeddedCredentials,
    /// Fragment is present.
    FragmentForbidden,
    /// Query string is present on the stored endpoint.
    QueryForbidden,
    /// Scheme is not `https` / granted `http`.
    UnsupportedScheme,
    /// Custom path is not the OpenAI-compatible API root.
    ArbitraryPath,
    /// HTTP used without `--allow-insecure-http`.
    InsecureHttpRequiresGrant,
    /// Loopback/private/LAN used without `--allow-private-network`.
    PrivateNetworkRequiresGrant,
    /// Official kind with a non-canonical endpoint.
    OfficialEndpointImmutable,
    /// Custom Anthropic-compatible endpoint.
    UnsupportedAnthropicCompatible,
    /// Unknown provider kind token.
    UnsupportedKind,
    /// Header injection / override attempt.
    ArbitraryHeaderForbidden,
    /// Redirects are never followed.
    RedirectForbidden,
    /// Resolved address is more private than the recorded grant.
    DnsRebinding,
    /// Endpoint authority, scheme, or network scope changed and must be reconfirmed.
    ReconfirmRequired,
}

impl EndpointTrustError {
    /// Stable error code for management JSON.
    pub fn code(self) -> &'static str {
        match self {
            Self::Invalid => "PROVIDER_ENDPOINT_INVALID",
            Self::EmbeddedCredentials => "PROVIDER_ENDPOINT_EMBEDDED_CREDENTIALS",
            Self::FragmentForbidden => "PROVIDER_ENDPOINT_FRAGMENT_FORBIDDEN",
            Self::QueryForbidden => "PROVIDER_ENDPOINT_QUERY_FORBIDDEN",
            Self::UnsupportedScheme => "PROVIDER_ENDPOINT_SCHEME_UNSUPPORTED",
            Self::ArbitraryPath => "PROVIDER_ENDPOINT_PATH_FORBIDDEN",
            Self::InsecureHttpRequiresGrant => "PROVIDER_ENDPOINT_HTTP_REQUIRES_GRANT",
            Self::PrivateNetworkRequiresGrant => "PROVIDER_ENDPOINT_PRIVATE_REQUIRES_GRANT",
            Self::OfficialEndpointImmutable => "PROVIDER_ENDPOINT_OFFICIAL_IMMUTABLE",
            Self::UnsupportedAnthropicCompatible => {
                "PROVIDER_ENDPOINT_ANTHROPIC_COMPATIBLE_FORBIDDEN"
            }
            Self::UnsupportedKind => "PROVIDER_KIND_UNSUPPORTED",
            Self::ArbitraryHeaderForbidden => "PROVIDER_ENDPOINT_HEADER_INJECTION_FORBIDDEN",
            Self::RedirectForbidden => "PROVIDER_ENDPOINT_REDIRECT_FORBIDDEN",
            Self::DnsRebinding => "PROVIDER_ENDPOINT_DNS_REBINDING",
            Self::ReconfirmRequired => "PROVIDER_ENDPOINT_RECONFIRM_REQUIRED",
        }
    }
}

impl std::fmt::Display for EndpointTrustError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for EndpointTrustError {}

impl TrustedEndpoint {
    /// Validate and normalize an account endpoint.
    pub fn evaluate(
        kind: ProviderKind,
        raw_endpoint: Option<&str>,
        grant: EndpointTrustGrant,
    ) -> Result<Self, EndpointTrustError> {
        match kind {
            ProviderKind::OpenaiOfficial => {
                reject_custom_override(raw_endpoint, OPENAI_OFFICIAL_ENDPOINT)?;
                parse_https_official(kind, OPENAI_OFFICIAL_ENDPOINT)
            }
            ProviderKind::AnthropicOfficial => {
                reject_custom_override(raw_endpoint, ANTHROPIC_OFFICIAL_ENDPOINT)?;
                parse_https_official(kind, ANTHROPIC_OFFICIAL_ENDPOINT)
            }
            ProviderKind::OpenaiCompatible => {
                let raw = raw_endpoint.unwrap_or("").trim();
                if raw.is_empty() {
                    return Err(EndpointTrustError::Invalid);
                }
                let parsed = parse_custom(raw, grant)?;
                if parsed.host.eq_ignore_ascii_case("api.anthropic.com") {
                    return Err(EndpointTrustError::UnsupportedAnthropicCompatible);
                }
                Ok(parsed)
            }
        }
    }

    /// Reconstruct a previously persisted trusted endpoint.
    pub fn from_persisted(
        kind: ProviderKind,
        normalized: &str,
        network_scope: &str,
        grant: EndpointTrustGrant,
    ) -> Result<Self, EndpointTrustError> {
        let parsed = Self::evaluate(kind, Some(normalized), grant)?;
        if parsed.scope != NetworkScope::parse(network_scope)? {
            return Err(EndpointTrustError::ReconfirmRequired);
        }
        Ok(parsed)
    }

    /// Durable URL with no userinfo, query, or fragment.
    pub fn normalized(&self) -> &str {
        &self.normalized
    }

    pub fn kind(&self) -> ProviderKind {
        self.kind
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn scope(&self) -> NetworkScope {
        self.scope
    }

    pub fn grant(&self) -> EndpointTrustGrant {
        self.grant
    }

    pub fn uses_http(&self) -> bool {
        self.scheme == EndpointScheme::Http
    }

    /// Join a provider-relative path (`/models`, `/chat/completions`).
    pub fn join_api_path(&self, relative: &str) -> Result<String, EndpointTrustError> {
        if !relative.starts_with('/') || relative.contains("://") || relative.contains('?') {
            return Err(EndpointTrustError::ArbitraryPath);
        }
        if relative.contains(['@', '#', '\\']) {
            return Err(EndpointTrustError::ArbitraryPath);
        }
        let base = self.normalized.trim_end_matches('/');
        Ok(format!("{base}{relative}"))
    }

    /// True when a later URL changes scheme, host, port, or broadens scope.
    pub fn requires_reconfirm(&self, next: &TrustedEndpoint) -> bool {
        self.scheme != next.scheme
            || self.host != next.host
            || self.port != next.port
            || next.scope < self.scope
            || (self.scheme == EndpointScheme::Https && next.scheme == EndpointScheme::Http)
    }
}

/// Header names callers may not supply. Official adapters emit a fixed set.
pub fn reject_caller_headers(
    headers: &[(String, String)],
    kind: ProviderKind,
) -> Result<(), EndpointTrustError> {
    for (name, value) in headers {
        if name.contains(['\r', '\n']) || value.contains(['\r', '\n']) {
            return Err(EndpointTrustError::ArbitraryHeaderForbidden);
        }
        let lowered = name.to_ascii_lowercase();
        match kind {
            ProviderKind::OpenaiOfficial | ProviderKind::OpenaiCompatible => {
                if lowered != "authorization" && lowered != "content-type" && lowered != "accept" {
                    return Err(EndpointTrustError::ArbitraryHeaderForbidden);
                }
                if lowered == "authorization" && !value.starts_with("Bearer ") {
                    return Err(EndpointTrustError::ArbitraryHeaderForbidden);
                }
            }
            ProviderKind::AnthropicOfficial => {
                if lowered != "x-api-key"
                    && lowered != "anthropic-version"
                    && lowered != "content-type"
                    && lowered != "accept"
                {
                    return Err(EndpointTrustError::ArbitraryHeaderForbidden);
                }
                if lowered == "authorization" {
                    return Err(EndpointTrustError::ArbitraryHeaderForbidden);
                }
            }
        }
    }
    Ok(())
}

/// Evaluate DNS answers against the recorded grant. Any more-private address
/// than the grant allows is treated as DNS rebinding.
pub fn evaluate_resolved_targets(
    endpoint: &TrustedEndpoint,
    resolved: &[IpAddr],
) -> Result<(), EndpointTrustError> {
    if resolved.is_empty() {
        return Err(EndpointTrustError::Invalid);
    }
    let allowed = if endpoint.grant.allow_private_network {
        NetworkScope::Loopback
    } else {
        NetworkScope::Public
    };
    for address in resolved {
        let scope = classify_ip(*address);
        if scope < allowed {
            return Err(EndpointTrustError::DnsRebinding);
        }
        if scope < endpoint.scope && !endpoint.grant.allow_private_network {
            return Err(EndpointTrustError::DnsRebinding);
        }
        if endpoint.scope == NetworkScope::Public
            && scope < NetworkScope::Public
            && !endpoint.grant.allow_private_network
        {
            return Err(EndpointTrustError::DnsRebinding);
        }
    }
    Ok(())
}

fn reject_custom_override(
    raw_endpoint: Option<&str>,
    official: &str,
) -> Result<(), EndpointTrustError> {
    match raw_endpoint
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        None => Ok(()),
        Some(value) if value == official => Ok(()),
        Some(_) => Err(EndpointTrustError::OfficialEndpointImmutable),
    }
}

fn parse_https_official(
    kind: ProviderKind,
    official: &str,
) -> Result<TrustedEndpoint, EndpointTrustError> {
    parse_custom(
        official,
        EndpointTrustGrant {
            allow_private_network: false,
            allow_insecure_http: false,
        },
    )
    .map(|mut endpoint| {
        endpoint.kind = kind;
        endpoint
    })
}

fn parse_custom(
    raw: &str,
    grant: EndpointTrustGrant,
) -> Result<TrustedEndpoint, EndpointTrustError> {
    if raw.is_empty() || raw.len() > 512 || raw.chars().any(char::is_whitespace) {
        return Err(EndpointTrustError::Invalid);
    }
    if raw.contains('#') {
        return Err(EndpointTrustError::FragmentForbidden);
    }
    if raw.contains('?') {
        return Err(EndpointTrustError::QueryForbidden);
    }
    if raw.contains('@') {
        return Err(EndpointTrustError::EmbeddedCredentials);
    }

    let (scheme, rest) = if let Some(rest) = raw.strip_prefix("https://") {
        (EndpointScheme::Https, rest)
    } else if let Some(rest) = raw.strip_prefix("http://") {
        (EndpointScheme::Http, rest)
    } else {
        return Err(EndpointTrustError::UnsupportedScheme);
    };
    if scheme == EndpointScheme::Http && !grant.allow_insecure_http {
        return Err(EndpointTrustError::InsecureHttpRequiresGrant);
    }

    let (authority, path) = match rest.split_once('/') {
        Some((authority, tail)) => (authority, format!("/{tail}")),
        None => (rest, String::new()),
    };
    if authority.is_empty() || authority.starts_with('[') && !authority.contains(']') {
        return Err(EndpointTrustError::Invalid);
    }
    if authority.contains(['\\', ' ', '\t']) {
        return Err(EndpointTrustError::Invalid);
    }
    let (host, port) = split_host_port(authority, scheme)?;
    validate_root_path(&path)?;
    let scope = classify_host(&host);
    if scope < NetworkScope::Public && !grant.allow_private_network {
        return Err(EndpointTrustError::PrivateNetworkRequiresGrant);
    }

    let normalized_path = match path.as_str() {
        "/" | "" => String::new(),
        "/v1/" => "/v1".to_owned(),
        other => other.to_owned(),
    };
    let host_rendered = render_host(&host);
    let default_port = default_port(scheme);
    let authority_rendered = if port == default_port {
        host_rendered.clone()
    } else {
        format!("{host_rendered}:{port}")
    };
    let normalized = if normalized_path.is_empty() {
        format!("{}://{authority_rendered}", scheme.as_str())
    } else {
        format!(
            "{}://{authority_rendered}{normalized_path}",
            scheme.as_str()
        )
    };

    Ok(TrustedEndpoint {
        kind: ProviderKind::OpenaiCompatible,
        normalized,
        scheme,
        host,
        port,
        path: normalized_path,
        scope,
        grant,
    })
}

fn split_host_port(
    authority: &str,
    scheme: EndpointScheme,
) -> Result<(String, u16), EndpointTrustError> {
    if let Some(rest) = authority.strip_prefix('[') {
        let Some((host, after)) = rest.split_once(']') else {
            return Err(EndpointTrustError::Invalid);
        };
        let port = if after.is_empty() {
            default_port(scheme)
        } else {
            let Some(port_text) = after.strip_prefix(':') else {
                return Err(EndpointTrustError::Invalid);
            };
            parse_port(port_text)?
        };
        let _: Ipv6Addr = host.parse().map_err(|_| EndpointTrustError::Invalid)?;
        return Ok((host.to_ascii_lowercase(), port));
    }
    if let Some((host, port_text)) = authority.rsplit_once(':') {
        if host.contains(':') {
            return Err(EndpointTrustError::Invalid);
        }
        if !port_text.is_empty() && port_text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Ok((host.to_ascii_lowercase(), parse_port(port_text)?));
        }
    }
    Ok((authority.to_ascii_lowercase(), default_port(scheme)))
}

fn parse_port(text: &str) -> Result<u16, EndpointTrustError> {
    let port: u16 = text.parse().map_err(|_| EndpointTrustError::Invalid)?;
    if port == 0 {
        return Err(EndpointTrustError::Invalid);
    }
    Ok(port)
}

fn default_port(scheme: EndpointScheme) -> u16 {
    match scheme {
        EndpointScheme::Https => 443,
        EndpointScheme::Http => 80,
    }
}

fn validate_root_path(path: &str) -> Result<(), EndpointTrustError> {
    match path {
        "" | "/" | "/v1" | "/v1/" => Ok(()),
        _ => Err(EndpointTrustError::ArbitraryPath),
    }
}

fn classify_host(host: &str) -> NetworkScope {
    if host.eq_ignore_ascii_case("localhost") || host.eq_ignore_ascii_case("localhost.") {
        return NetworkScope::Loopback;
    }
    if let Ok(address) = host.parse::<IpAddr>() {
        return classify_ip(address);
    }
    NetworkScope::Public
}

fn classify_ip(address: IpAddr) -> NetworkScope {
    match address {
        IpAddr::V4(v4) => classify_ipv4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return classify_ipv4(v4);
            }
            if v6.is_loopback() {
                NetworkScope::Loopback
            } else if ipv6_is_private(v6) {
                NetworkScope::Private
            } else {
                NetworkScope::Public
            }
        }
    }
}

fn classify_ipv4(address: Ipv4Addr) -> NetworkScope {
    if address.is_loopback() {
        NetworkScope::Loopback
    } else if address.is_private()
        || address.is_link_local()
        || address.is_unspecified()
        || address.is_broadcast()
        || address.is_multicast()
        || is_carrier_grade_nat(address)
    {
        NetworkScope::Private
    } else {
        NetworkScope::Public
    }
}

fn is_carrier_grade_nat(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    octets[0] == 100 && (octets[1] & 0b1100_0000) == 64
}

fn ipv6_is_private(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    (segments[0] & 0xfe00) == 0xfc00 || (segments[0] & 0xffc0) == 0xfe80 || address.is_unspecified()
}

fn render_host(host: &str) -> String {
    if host.contains(':') {
        format!("[{host}]")
    } else {
        host.to_owned()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn public_grant() -> EndpointTrustGrant {
        EndpointTrustGrant {
            allow_private_network: false,
            allow_insecure_http: false,
        }
    }

    fn private_http_grant() -> EndpointTrustGrant {
        EndpointTrustGrant {
            allow_private_network: true,
            allow_insecure_http: true,
        }
    }

    #[test]
    fn official_endpoints_are_immutable_and_reject_overrides() {
        let openai =
            TrustedEndpoint::evaluate(ProviderKind::OpenaiOfficial, None, public_grant()).unwrap();
        assert_eq!(openai.normalized(), OPENAI_OFFICIAL_ENDPOINT);
        assert!(matches!(
            TrustedEndpoint::evaluate(
                ProviderKind::OpenaiOfficial,
                Some("https://example.test/v1"),
                public_grant()
            ),
            Err(EndpointTrustError::OfficialEndpointImmutable)
        ));
        assert!(matches!(
            TrustedEndpoint::evaluate(
                ProviderKind::AnthropicOfficial,
                Some("https://api.anthropic.com.evil.test"),
                public_grant()
            ),
            Err(EndpointTrustError::OfficialEndpointImmutable)
        ));
    }

    #[test]
    fn embedded_credentials_fragments_queries_and_arbitrary_paths_fail() {
        for url in [
            "https://user:pass@api.openai.com/v1",
            "https://api.example.test/v1#frag",
            "https://api.example.test/v1?foo=1",
            "https://api.example.test/v1/chat/completions",
            "https://api.example.test/v1/../secret",
        ] {
            assert!(
                TrustedEndpoint::evaluate(
                    ProviderKind::OpenaiCompatible,
                    Some(url),
                    public_grant()
                )
                .is_err(),
                "must refuse {url}"
            );
        }
        assert_eq!(
            TrustedEndpoint::evaluate(
                ProviderKind::OpenaiCompatible,
                Some("https://user:pass@api.example.test/v1"),
                public_grant()
            ),
            Err(EndpointTrustError::EmbeddedCredentials)
        );
    }

    #[test]
    fn http_and_private_targets_require_explicit_grants() {
        assert_eq!(
            TrustedEndpoint::evaluate(
                ProviderKind::OpenaiCompatible,
                Some("http://api.example.test/v1"),
                public_grant()
            ),
            Err(EndpointTrustError::InsecureHttpRequiresGrant)
        );
        assert_eq!(
            TrustedEndpoint::evaluate(
                ProviderKind::OpenaiCompatible,
                Some("https://127.0.0.1/v1"),
                public_grant()
            ),
            Err(EndpointTrustError::PrivateNetworkRequiresGrant)
        );
        assert_eq!(
            TrustedEndpoint::evaluate(
                ProviderKind::OpenaiCompatible,
                Some("https://192.168.1.10/v1"),
                public_grant()
            ),
            Err(EndpointTrustError::PrivateNetworkRequiresGrant)
        );
        let granted = TrustedEndpoint::evaluate(
            ProviderKind::OpenaiCompatible,
            Some("http://10.0.0.8/v1"),
            private_http_grant(),
        )
        .unwrap();
        assert!(granted.uses_http());
        assert_eq!(granted.scope(), NetworkScope::Private);
    }

    #[test]
    fn dns_rebinding_to_loopback_fails_without_private_grant() {
        let endpoint = TrustedEndpoint::evaluate(
            ProviderKind::OpenaiCompatible,
            Some("https://models.example.test/v1"),
            public_grant(),
        )
        .unwrap();
        assert_eq!(
            evaluate_resolved_targets(&endpoint, &[IpAddr::V4(Ipv4Addr::LOCALHOST)]),
            Err(EndpointTrustError::DnsRebinding)
        );
        assert!(
            evaluate_resolved_targets(&endpoint, &[IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))]).is_ok()
        );
    }

    #[test]
    fn anthropic_compatible_kind_and_host_are_rejected() {
        assert_eq!(
            ProviderKind::parse("anthropic_compatible"),
            Err(EndpointTrustError::UnsupportedAnthropicCompatible)
        );
        assert_eq!(
            TrustedEndpoint::evaluate(
                ProviderKind::OpenaiCompatible,
                Some("https://api.anthropic.com/v1"),
                public_grant()
            ),
            Err(EndpointTrustError::UnsupportedAnthropicCompatible)
        );
    }

    #[test]
    fn caller_cannot_inject_headers_or_override_bearer() {
        assert_eq!(
            reject_caller_headers(
                &[("X-Api-Key".to_owned(), "secret".to_owned())],
                ProviderKind::OpenaiCompatible
            ),
            Err(EndpointTrustError::ArbitraryHeaderForbidden)
        );
        assert_eq!(
            reject_caller_headers(
                &[("Authorization".to_owned(), "Basic abc".to_owned())],
                ProviderKind::OpenaiCompatible
            ),
            Err(EndpointTrustError::ArbitraryHeaderForbidden)
        );
        assert_eq!(
            reject_caller_headers(
                &[("Authorization".to_owned(), "Bearer x".to_owned())],
                ProviderKind::AnthropicOfficial
            ),
            Err(EndpointTrustError::ArbitraryHeaderForbidden)
        );
    }

    #[test]
    fn https_to_http_or_authority_change_requires_reconfirm() {
        let https = TrustedEndpoint::evaluate(
            ProviderKind::OpenaiCompatible,
            Some("https://api.example.test/v1"),
            public_grant(),
        )
        .unwrap();
        let http = TrustedEndpoint::evaluate(
            ProviderKind::OpenaiCompatible,
            Some("http://api.example.test/v1"),
            EndpointTrustGrant {
                allow_private_network: false,
                allow_insecure_http: true,
            },
        )
        .unwrap();
        assert!(https.requires_reconfirm(&http));
        let other_host = TrustedEndpoint::evaluate(
            ProviderKind::OpenaiCompatible,
            Some("https://other.example.test/v1"),
            public_grant(),
        )
        .unwrap();
        assert!(https.requires_reconfirm(&other_host));
    }
}
