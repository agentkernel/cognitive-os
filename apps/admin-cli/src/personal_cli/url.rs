//! Provider base URL normalization for `cognitive init`.

/// Normalize a user-supplied Provider base URL.
///
/// Corrections:
/// - trim surrounding whitespace
/// - strip a single trailing `/`
///
/// Failures (actionable):
/// - empty
/// - `http://` (must use `https://`)
/// - embedded credentials (`user:pass@`)
/// - whitespace inside the URL
pub fn normalize_provider_base_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(
            "base_url is empty; provide an absolute https:// Provider API root \
             (example: https://api.deepseek.com/v1)"
                .to_owned(),
        );
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(
            "base_url must not contain whitespace; remove spaces and retry".to_owned(),
        );
    }
    let lowercase = trimmed.to_ascii_lowercase();
    if lowercase.starts_with("http://") {
        return Err(
            "base_url uses http:// which is rejected; use https:// and retry \
             (example: https://api.deepseek.com/v1)"
                .to_owned(),
        );
    }
    if !lowercase.starts_with("https://") {
        return Err(
            "base_url must start with https://; corrected form example: \
             https://api.deepseek.com/v1"
                .to_owned(),
        );
    }
    if trimmed["https://".len()..].contains('@') {
        return Err(
            "base_url must not embed credentials; store the API key via SecretStore \
             (--api-key-file) instead of the URL"
                .to_owned(),
        );
    }
    let without_trailing_slash = trimmed.trim_end_matches('/');
    if without_trailing_slash == "https://" {
        return Err("base_url host is required after https://".to_owned());
    }
    Ok(without_trailing_slash.to_owned())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::normalize_provider_base_url;

    #[test]
    fn strips_trailing_slash_and_rejects_http() {
        let normalized =
            normalize_provider_base_url("  https://api.deepseek.com/v1/  ").unwrap();
        assert_eq!(normalized, "https://api.deepseek.com/v1");
        let http_error = normalize_provider_base_url("http://api.deepseek.com/v1").unwrap_err();
        assert!(http_error.contains("https://"), "{http_error}");
    }
}
