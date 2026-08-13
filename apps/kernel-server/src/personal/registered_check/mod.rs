//! 由 daemon 固定登记的确定性检查执行边界。

#[cfg(test)]
mod tests {
    use super::{RegisteredCheckRegistry, RegisteredCheckRunRequest};

    #[test]
    fn caller_can_request_registered_check_by_check_id_only() {
        let request = RegisteredCheckRunRequest::new("c2a.repair.typescript");
        let descriptor = RegisteredCheckRegistry::production()
            .resolve(&request)
            .expect("固定登记的 C2a 检查应可仅凭 check_id 解析");

        assert_eq!(descriptor.check_id(), "c2a.repair.typescript");
    }
}
