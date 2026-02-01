//! 字节跳动 LogID 查询工具库
//!
//! 提供基于 Rust 的 logid 查询功能，支持多区域日志查询。
//! 这个库包含了所有核心功能模块，可以被其他项目引用。
//!
//! ## 功能特性
//! - 🌍 多区域支持：美区 (us)、国际化区域 (i18n)、中国区 (cn)
//! - 🔐 JWT 认证：自动获取和刷新认证令牌，支持令牌缓存
//! - 🔍 消息过滤：支持自定义过滤规则，移除冗余信息
//! - 📋 JSON 输出：结构化 JSON 格式输出
//! - ❌ 错误处理：友好的错误信息和上下文提示
//! - 🔧 环境变量支持：从 `.env` 文件读取配置

// ============================================================================
// 公共宏定义 - 必须在所有模块声明之前
// ============================================================================

/// 检查日志是否启用
#[doc(hidden)]
pub fn __is_logging_enabled() -> bool {
    std::env::var("ENABLE_LOGGING")
        .map(|v| {
            let v = v.to_lowercase();
            v == "true" || v == "on" || v == "1" || v == "yes"
        })
        .unwrap_or(false)
}

/// 条件日志宏，只在 ENABLE_LOGGING 环境变量启用时输出
#[macro_export]
macro_rules! conditional_info {
    ($($arg:tt)*) => {
        if $crate::__is_logging_enabled() {
            tracing::info!($($arg)*);
        }
    };
}

// ============================================================================
// 模块声明
// ============================================================================

pub mod auth;
pub mod config;
pub mod error;
pub mod log_query;
pub mod output;

// 重新导出主要的公共类型和函数
pub use auth::{AuthManager, MultiRegionAuthManager};
pub use config::{
    create_message_filters, get_default_filters, get_region_config, EnvManager, FilterConfig,
    JwtInfo, Region, RegionConfig,
};
pub use error::LogidError;
pub use log_query::{
    DetailedLogResult, ExtractedLogMessage, ExtractedValue, LogGroup, LogMeta, LogQueryClient,
    LogQueryRequest, LogQueryResponse, MultiRegionLogQuery,
};
pub use output::{
    print_json_output, write_to_file, OutputConfig, OutputFormatter,
};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 库的简单描述
pub const DESCRIPTION: &str = "基于 Rust 的 logid 查询工具，支持多区域日志查询";
