//! 输出便捷函数模块

use crate::conditional_info;
use crate::error::LogidError;
use crate::log_query::DetailedLogResult;
use crate::output::format::OutputConfig;
use crate::output::formatter::OutputFormatter;

/// 便捷函数：打印 JSON 格式输出
#[allow(dead_code)]
pub fn print_json_output(log_details: &DetailedLogResult) -> Result<(), LogidError> {
    let config = OutputConfig::new();
    let formatter = OutputFormatter::new(config);
    formatter.print_result(log_details)
}

/// 便捷函数：输出到文件
#[allow(dead_code)]
pub fn write_to_file(
    log_details: &DetailedLogResult,
    file_path: &str,
    config: OutputConfig,
) -> Result<(), LogidError> {
    let mut file = std::fs::File::create(file_path).map_err(LogidError::IoError)?;

    let formatter = OutputFormatter::new(config);
    formatter.write_result(&mut file, log_details)?;

    conditional_info!("日志结果已写入文件: {}", file_path);
    Ok(())
}
