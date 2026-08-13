//! Daemon-side nested stage observation for the Pi Provider route (P9-T05).
//!
//! The Pi client measures the loopback wait it observes; only the daemon can
//! separate the preflight/SecretStore work inside that wait from the outbound
//! Provider exchange. This module produces the two response headers that carry
//! that split, plus the correlation echo that joins them to the Pi-side record.
//!
//! It is deliberately inert:
//!
//!   - **Denied by default.** Without an explicit campaign authorization in the
//!     daemon's own environment, nothing is echoed and no stage is reported.
//!     The request is served exactly as it was before this module existed.
//!   - **Not an authority writer.** It holds no store, no secret backend and no
//!     transport. It returns a header string; it cannot persist, mutate or
//!     schedule anything, and it never changes a response body or status.
//!   - **Content-free.** The only value it reflects is an opaque correlation
//!     id whose shape excludes a bearer, a `SecretRef` or a prompt fragment. A
//!     request header that is malformed, oversized or duplicated is refused
//!     rather than echoed.

const CORRELATION_ID_HEADER: &str = "x-cognitiveos-correlation-id";
const CORRELATION_ID_PREFIX: &str = "campaign-";
const CORRELATION_ID_HEX_LENGTH: usize = 32;

/// Environment variable that requests daemon-side stage reporting.
pub const ROUTE_OBSERVATION_ENABLE_VARIABLE: &str = "COGNITIVEOS_PI_ROUTE_OBSERVATION";

/// The only value that enables it.
pub const ROUTE_OBSERVATION_ENABLED_VALUE: &str = "enabled";

/// Why a request carries no joinable correlation id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrelationRefusal {
    /// The request carried no correlation header at all.
    Absent,
    /// The header value is not the fixed opaque campaign shape.
    Malformed,
    /// The header appeared more than once, so no single request is identified.
    Duplicated,
}

/// Extract the one opaque correlation id a request may carry.
///
/// A duplicate header is refused rather than resolved by precedence: two ids
/// mean the observation cannot identify a single request, and silently picking
/// one would let a caller split or merge samples.
pub fn extract_correlation_id(headers: &str) -> Result<String, CorrelationRefusal> {
    let mut found: Option<&str> = None;
    for line in headers.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if !name.trim().eq_ignore_ascii_case(CORRELATION_ID_HEADER) {
            continue;
        }
        if found.is_some() {
            return Err(CorrelationRefusal::Duplicated);
        }
        found = Some(value.trim());
    }
    let value = found.ok_or(CorrelationRefusal::Absent)?;
    if is_opaque_correlation_id(value) {
        Ok(value.to_owned())
    } else {
        Err(CorrelationRefusal::Malformed)
    }
}

fn is_opaque_correlation_id(value: &str) -> bool {
    // 与 Pi 侧 `campaign-[0-9a-f]{32}` 对齐：只接受小写 hex，避免大小写变体被当成
    // 另一次请求，也避免把任意 hex 形状的材料误当成 join key。
    let Some(hexadecimal_suffix) = value.strip_prefix(CORRELATION_ID_PREFIX) else {
        return false;
    };
    hexadecimal_suffix.len() == CORRELATION_ID_HEX_LENGTH
        && hexadecimal_suffix
            .bytes()
            .all(|character| matches!(character, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether the daemon's own environment authorizes stage reporting.
pub fn route_observation_authorized() -> bool {
    std::env::var(ROUTE_OBSERVATION_ENABLE_VARIABLE)
        .is_ok_and(|value| value == ROUTE_OBSERVATION_ENABLED_VALUE)
}

/// The nested durations one authorized Provider request produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NestedProviderStages {
    pub preflight_elapsed_nanos: u128,
    pub provider_network_elapsed_nanos: u128,
}

/// Build the observation response headers for one Provider request.
///
/// Returns an empty string — and therefore an unchanged response — whenever the
/// daemon is unauthorized, the correlation id is unusable, or either duration
/// is not positive. A zero-duration stage is dropped rather than emitted,
/// because a campaign that reads zero cannot tell "instant" from "unmeasured".
pub fn observation_response_headers(
    authorized: bool,
    correlation_id: Result<&str, CorrelationRefusal>,
    stages: NestedProviderStages,
) -> String {
    if !authorized {
        return String::new();
    }
    let Ok(correlation_id) = correlation_id else {
        return String::new();
    };
    if !is_opaque_correlation_id(correlation_id) {
        return String::new();
    }
    if stages.preflight_elapsed_nanos == 0 || stages.provider_network_elapsed_nanos == 0 {
        return String::new();
    }
    // Provider 网络时长仍由既有的 `X-CognitiveOS-Provider-Network-Nanos` 始终下发
    //（P9-T04 合同，不因本插桩而改成授权门控）。这里只追加 join 所需的回显与
    // preflight；两个时长都为正才回显，避免半组 nested stage 被当成完整观测。
    format!(
        "X-CognitiveOS-Correlation-Id: {correlation_id}\r\nX-CognitiveOS-Daemon-Preflight-Nanos: {}\r\n",
        stages.preflight_elapsed_nanos
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    const VALID_ID: &str = "campaign-0123456789abcdef0123456789abcdef";

    fn stages() -> NestedProviderStages {
        NestedProviderStages {
            preflight_elapsed_nanos: 1_000,
            provider_network_elapsed_nanos: 4_000,
        }
    }

    #[test]
    fn one_well_formed_correlation_header_is_extracted() {
        let headers = format!("Host: 127.0.0.1\r\nX-CognitiveOS-Correlation-Id: {VALID_ID}");
        assert_eq!(extract_correlation_id(&headers), Ok(VALID_ID.to_owned()));
    }

    #[test]
    fn a_malformed_correlation_header_is_refused_rather_than_echoed() {
        for value in [
            "",
            "campaign-",
            "campaign-0123456789abcdef",
            "campaign-0123456789abcdef0123456789abcdeg",
            "campaign-0123456789abcdef0123456789abcdefff",
            "campaign-0123456789ABCDEF0123456789ABCDEF",
            "Bearer local-session-token",
            "sk-0123456789abcdef0123456789abcdef",
        ] {
            let headers = format!("x-cognitiveos-correlation-id: {value}");
            assert_eq!(
                extract_correlation_id(&headers),
                Err(CorrelationRefusal::Malformed),
                "{value} must not be accepted as a correlation id"
            );
            assert!(
                observation_response_headers(true, Err(CorrelationRefusal::Malformed), stages())
                    .is_empty()
            );
        }
    }

    #[test]
    fn a_duplicated_correlation_header_identifies_no_single_request() {
        let headers = format!(
            "X-CognitiveOS-Correlation-Id: {VALID_ID}\r\nx-cognitiveos-correlation-id: {VALID_ID}"
        );
        assert_eq!(
            extract_correlation_id(&headers),
            Err(CorrelationRefusal::Duplicated)
        );
        assert!(
            observation_response_headers(true, Err(CorrelationRefusal::Duplicated), stages())
                .is_empty()
        );
    }

    #[test]
    fn an_absent_correlation_header_reports_no_stage() {
        assert_eq!(
            extract_correlation_id("Host: 127.0.0.1"),
            Err(CorrelationRefusal::Absent)
        );
        assert!(
            observation_response_headers(true, Err(CorrelationRefusal::Absent), stages())
                .is_empty()
        );
    }

    #[test]
    fn an_unauthorized_daemon_reports_nothing_even_for_a_valid_request() {
        assert!(observation_response_headers(false, Ok(VALID_ID), stages()).is_empty());
    }

    #[test]
    fn a_zero_duration_stage_is_dropped_instead_of_reported_as_instant() {
        for stages in [
            NestedProviderStages {
                preflight_elapsed_nanos: 0,
                provider_network_elapsed_nanos: 4_000,
            },
            NestedProviderStages {
                preflight_elapsed_nanos: 1_000,
                provider_network_elapsed_nanos: 0,
            },
        ] {
            assert!(observation_response_headers(true, Ok(VALID_ID), stages).is_empty());
        }
    }

    #[test]
    fn an_authorized_joined_request_reports_the_preflight_stage_and_the_echo() {
        let headers = observation_response_headers(true, Ok(VALID_ID), stages());
        assert_eq!(
            headers,
            format!(
                "X-CognitiveOS-Correlation-Id: {VALID_ID}\r\nX-CognitiveOS-Daemon-Preflight-Nanos: 1000\r\n"
            )
        );
        // 只追加响应头：不得提前结束头部、不得夹带 JSON body 或凭据形状材料。
        assert!(headers.ends_with("\r\n"));
        assert_eq!(headers.matches("\r\n").count(), 2);
        assert!(!headers.contains("\r\n\r\n"));
        assert!(!headers.contains('{'));
        assert!(!headers.to_ascii_lowercase().contains("authorization"));
        assert!(!headers.to_ascii_lowercase().contains("bearer"));
        assert!(!headers.contains("sk-"));
    }

    #[test]
    fn observation_headers_never_echo_a_refused_correlation_value() {
        let secret_shaped = "sk-0123456789abcdef0123456789abcdef";
        let headers = format!("x-cognitiveos-correlation-id: {secret_shaped}");
        assert_eq!(
            extract_correlation_id(&headers),
            Err(CorrelationRefusal::Malformed)
        );
        let emitted =
            observation_response_headers(true, Err(CorrelationRefusal::Malformed), stages());
        assert!(emitted.is_empty());
        assert!(!emitted.contains(secret_shaped));
    }

    #[test]
    fn authorized_and_unauthorized_header_blocks_leave_the_same_body_bytes() {
        // 产品响应用 body 参数单独写出；观测头只改变头部字符串。下面用同一段
        // body 证明授权开关不能改写或嵌入完成内容。
        let body = br#"{"id":"synthetic-completion","usage":{"prompt_tokens":1}}"#;
        let unauthorized = observation_response_headers(false, Ok(VALID_ID), stages());
        let authorized = observation_response_headers(true, Ok(VALID_ID), stages());
        assert!(unauthorized.is_empty());
        assert!(!authorized.is_empty());
        assert!(
            !authorized
                .as_bytes()
                .windows(body.len())
                .any(|window| window == body)
        );
        let unauthorized_frame = format!(
            "HTTP/1.1 200 OK\r\n{unauthorized}Content-Length: {}\r\n\r\n{}",
            body.len(),
            std::str::from_utf8(body).expect("fixture body")
        );
        let authorized_frame = format!(
            "HTTP/1.1 200 OK\r\n{authorized}Content-Length: {}\r\n\r\n{}",
            body.len(),
            std::str::from_utf8(body).expect("fixture body")
        );
        let unauthorized_body = unauthorized_frame
            .split("\r\n\r\n")
            .nth(1)
            .expect("unauthorized body");
        let authorized_body = authorized_frame
            .split("\r\n\r\n")
            .nth(1)
            .expect("authorized body");
        assert_eq!(unauthorized_body.as_bytes(), body);
        assert_eq!(authorized_body.as_bytes(), body);
        assert_eq!(unauthorized_body, authorized_body);
    }
}
