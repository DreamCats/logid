//! 日志查询模块
//!
//! 处理多区域的日志查询功能，通过 logid 进行日志搜索。
//! 支持并发区域查询和智能区域检测，提供统一的日志查询接口。

mod client;
mod multi_region;
mod types;

pub use client::LogQueryClient;
pub use multi_region::MultiRegionLogQuery;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_log_query_request() {
        let request = LogQueryRequest::new(
            "test_logid".to_string(),
            vec!["test_psm".to_string()],
            10,
            "test_vregion".to_string(),
        );

        assert_eq!(request.logid, "test_logid");
        assert_eq!(request.psm_list, vec!["test_psm"]);
        assert_eq!(request.scan_span_in_min, 10);
        assert_eq!(request.vregion, "test_vregion");
    }

    #[test]
    fn test_message_filtering() {
        let _filters = vec![Regex::new("test_filter").unwrap()];

        // 这里需要创建 LogQueryClient 实例来测试过滤功能
        // 由于构造函数需要异步，在单元测试中比较复杂
        // 可以考虑重构为同步测试或者使用异步测试框架
    }
}
