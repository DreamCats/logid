//! 输出格式化器模块

use crate::conditional_info;
use crate::error::LogidError;
use crate::log_query::DetailedLogResult;
use crate::output::format::OutputConfig;
use serde_json::json;
use std::io::{self, Write};
use tracing::error;

/// 输出格式化器
pub struct OutputFormatter {
    config: OutputConfig,
}

impl OutputFormatter {
    /// 创建新的输出格式化器
    pub fn new(config: OutputConfig) -> Self {
        Self { config }
    }

    /// 格式化日志详情为 JSON 格式
    pub fn format_log_result(&self, log_details: &DetailedLogResult) -> Result<String, LogidError> {
        conditional_info!("格式化日志结果为 JSON 格式: logid={}", log_details.logid);

        let mut json_result = json!({
            "logid": log_details.logid,
            "region": log_details.region,
            "region_display_name": log_details.region_display_name,
            "total_items": log_details.total_items,
            "messages": log_details.messages,
            "timestamp": log_details.timestamp,
        });

        if self.config.show_metadata {
            if let Some(meta) = &log_details.meta {
                json_result["meta"] = serde_json::to_value(meta).map_err(LogidError::JsonParseError)?;
            }
        }

        if self.config.show_scan_time_range {
            if let Some(scan_time_ranges) = &log_details.scan_time_range {
                json_result["scan_time_range"] = serde_json::to_value(scan_time_ranges)
                    .map_err(LogidError::JsonParseError)?;
            }
        }

        if self.config.show_tag_infos {
            if let Some(tag_infos) = &log_details.tag_infos {
                json_result["tag_infos"] = serde_json::to_value(tag_infos)
                    .map_err(LogidError::JsonParseError)?;
            }
        }

        serde_json::to_string_pretty(&json_result).map_err(LogidError::JsonParseError)
    }

    /// 打印格式化结果到标准输出
    pub fn print_result(&self, log_details: &DetailedLogResult) -> Result<(), LogidError> {
        let formatted_output = self.format_log_result(log_details)?;
        print!("{}", formatted_output);
        io::stdout().flush().map_err(|e| {
            error!("刷新标准输出失败: {}", e);
            LogidError::IoError(e)
        })?;
        Ok(())
    }

    /// 写入格式化结果到指定的写入器
    #[allow(dead_code)]
    pub fn write_result<W: Write>(&self, writer: &mut W, log_details: &DetailedLogResult) -> Result<(), LogidError> {
        let formatted_output = self.format_log_result(log_details)?;
        writer.write_all(formatted_output.as_bytes()).map_err(|e| {
            error!("写入输出失败: {}", e);
            LogidError::IoError(e)
        })?;
        writer.flush().map_err(|e| {
            error!("刷新写入器失败: {}", e);
            LogidError::IoError(e)
        })?;
        Ok(())
    }
}
