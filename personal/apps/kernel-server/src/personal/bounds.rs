//! Resource bounds for the Personal daemon (ADR-0019 section 3).

/// Personal v1 baseline ceilings. Values may tighten later but must not be
/// removed without a new product decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonalResourceBounds {
    pub max_request_body_bytes: usize,
    pub hard_body_ceiling_bytes: usize,
    pub max_header_block_bytes: usize,
    pub max_header_count: usize,
    pub read_header_timeout_secs: u64,
    pub request_body_read_timeout_secs: u64,
    pub max_concurrent_connections: usize,
    pub max_concurrent_connections_per_channel: usize,
    pub max_in_flight_requests: usize,
    pub session_absolute_lifetime_secs: u64,
    pub session_idle_lifetime_secs: u64,
}

impl PersonalResourceBounds {
    /// ADR-0019 baseline table.
    pub const fn personal_v1_baseline() -> Self {
        Self {
            max_request_body_bytes: 1024 * 1024,
            hard_body_ceiling_bytes: 8 * 1024 * 1024,
            max_header_block_bytes: 16 * 1024,
            max_header_count: 64,
            read_header_timeout_secs: 10,
            request_body_read_timeout_secs: 30,
            max_concurrent_connections: 32,
            max_concurrent_connections_per_channel: 16,
            max_in_flight_requests: 16,
            session_absolute_lifetime_secs: 12 * 60 * 60,
            session_idle_lifetime_secs: 30 * 60,
        }
    }

    /// Effective body limit: min(default, hard ceiling).
    pub fn effective_max_body_bytes(&self) -> usize {
        self.max_request_body_bytes
            .min(self.hard_body_ceiling_bytes)
    }
}

/// Fail-closed outcomes when a request violates resource bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBoundError {
    BodyTooLarge { observed: usize, limit: usize },
    HeaderBlockTooLarge { observed: usize, limit: usize },
    TooManyHeaders { observed: usize, limit: usize },
    ConnectionLimitExceeded,
    InFlightLimitExceeded,
}

impl RequestBoundError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BodyTooLarge { .. } => "REQUEST_BODY_TOO_LARGE",
            Self::HeaderBlockTooLarge { .. } => "REQUEST_HEADER_BLOCK_TOO_LARGE",
            Self::TooManyHeaders { .. } => "REQUEST_HEADER_COUNT_EXCEEDED",
            Self::ConnectionLimitExceeded => "CONNECTION_LIMIT_EXCEEDED",
            Self::InFlightLimitExceeded => "IN_FLIGHT_LIMIT_EXCEEDED",
        }
    }
}

/// Validate raw header-block length and header line count (excluding the
/// request line). `header_block` is the bytes between the request line CRLF
/// and the blank line that ends headers.
pub fn validate_header_block(
    header_block: &[u8],
    bounds: &PersonalResourceBounds,
) -> Result<(), RequestBoundError> {
    if header_block.len() > bounds.max_header_block_bytes {
        return Err(RequestBoundError::HeaderBlockTooLarge {
            observed: header_block.len(),
            limit: bounds.max_header_block_bytes,
        });
    }
    let header_count = header_block
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty() && *line != b"\r")
        .count();
    if header_count > bounds.max_header_count {
        return Err(RequestBoundError::TooManyHeaders {
            observed: header_count,
            limit: bounds.max_header_count,
        });
    }
    Ok(())
}

/// Validate declared or observed body length against the effective body limit.
pub fn validate_body_length(
    body_length: usize,
    bounds: &PersonalResourceBounds,
) -> Result<(), RequestBoundError> {
    let limit = bounds.effective_max_body_bytes();
    if body_length > limit {
        return Err(RequestBoundError::BodyTooLarge {
            observed: body_length,
            limit,
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn baseline_body_limit_is_one_mebibyte() {
        let bounds = PersonalResourceBounds::personal_v1_baseline();
        assert_eq!(bounds.effective_max_body_bytes(), 1024 * 1024);
        assert!(validate_body_length(1024, &bounds).is_ok());
        assert!(matches!(
            validate_body_length(bounds.effective_max_body_bytes() + 1, &bounds),
            Err(RequestBoundError::BodyTooLarge { .. })
        ));
    }

    #[test]
    fn header_block_over_limit_fails_closed() {
        let bounds = PersonalResourceBounds::personal_v1_baseline();
        let oversized = vec![b'a'; bounds.max_header_block_bytes + 1];
        assert!(matches!(
            validate_header_block(&oversized, &bounds),
            Err(RequestBoundError::HeaderBlockTooLarge { .. })
        ));
    }
}
