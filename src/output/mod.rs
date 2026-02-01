//! 输出格式化模块
//!
//! 提供 JSON 格式输出支持。

mod format;
mod formatter;
mod utils;

pub use format::OutputConfig;
pub use formatter::OutputFormatter;
pub use utils::{print_json_output, write_to_file};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_query::{ExtractedLogMessage, ExtractedValue, LogGroup, TimeRange, DetailedLogResult};
    use serde_json::Value;

    fn create_test_log_result() -> DetailedLogResult {
        DetailedLogResult {
            logid: "test_logid_123".to_string(),
            messages: vec![
                ExtractedLogMessage {
                    id: "msg_1".to_string(),
                    group: LogGroup {
                        psm: Some("test.psm".to_string()),
                        pod_name: Some("test-pod-123".to_string()),
                        ipv4: Some("192.168.1.100".to_string()),
                        env: Some("production".to_string()),
                        vregion: Some("US-TTP".to_string()),
                        idc: Some("us-east-1".to_string()),
                    },
                    values: vec![
                        ExtractedValue {
                            key: "_msg".to_string(),
                            value: "这是一条测试消息".to_string(),
                            original_value: "这是一条测试消息".to_string(),
                            type_field: Some("string".to_string()),
                            highlight: false,
                        },
                    ],
                    level: Some("INFO".to_string()),
                    location: Some("src/main.rs:42".to_string()),
                },
            ],
            meta: None,
            tag_infos: None,
            total_items: 1,
            scan_time_range: Some(vec![TimeRange {
                start: Some(1609459200),
                end: Some(1609459260),
            }]),
            level_list: Some(vec!["INFO".to_string(), "ERROR".to_string()]),
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            region: "us".to_string(),
            region_display_name: "美区".to_string(),
        }
    }

    #[test]
    fn test_output_config_default() {
        let config = OutputConfig::default();
        assert!(config.show_metadata);
        assert!(config.show_scan_time_range);
        assert!(!config.show_tag_infos);
    }

    #[test]
    fn test_formatter_json_output() {
        let config = OutputConfig::new();
        let formatter = OutputFormatter::new(config);
        let log_result = create_test_log_result();

        let output = formatter.format_log_result(&log_result).unwrap();

        let json_value: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(json_value["logid"], "test_logid_123");
        assert_eq!(json_value["region"], "us");
        assert_eq!(json_value["region_display_name"], "美区");
        assert_eq!(json_value["total_items"], 1);
        assert!(json_value["messages"].is_array());
    }

    #[test]
    fn test_print_json_output() {
        let log_result = create_test_log_result();
        assert!(print_json_output(&log_result).is_ok());
    }
}
