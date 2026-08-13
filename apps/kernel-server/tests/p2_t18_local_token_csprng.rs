const AUTH_SOURCE: &str = include_str!("../src/personal/auth.rs");

fn production_auth_source() -> &'static str {
    AUTH_SOURCE
        .split_once("mod tests {")
        .map_or(AUTH_SOURCE, |(production, _tests)| production)
}

#[test]
fn production_token_generation_uses_only_the_os_csprng() {
    let source = production_auth_source();
    let forbidden_markers = [
        ["Default", "Hasher"].concat(),
        ["std::process::", "id()"].concat(),
        ["Instant::now()", ".hash"].concat(),
        ["elapsed()", ".as_nanos()"].concat(),
        ["random_", "u64"].concat(),
    ];

    for marker in forbidden_markers {
        assert!(
            !source.contains(&marker),
            "生产令牌生成仍包含禁止的非密码学熵来源"
        );
    }

    // 只确认生产路径调用操作系统熵接口，不据此作统计随机性声明。
    let os_csprng_marker = ["get", "random::fill"].concat();
    assert!(
        source.contains(&os_csprng_marker),
        "生产令牌生成尚未调用操作系统 CSPRNG"
    );
}
