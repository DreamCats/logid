//! 错误处理模块
//!
//! 定义了应用中使用的各种错误类型，提供友好的错误信息和上下文。

use thiserror::Error;

/// 应用主错误类型
#[derive(Error, Debug)]
pub enum LogidError {
    #[error("不支持的区域: {0}")]
    UnsupportedRegion(String),

    #[error("区域 {0} 未配置，请提供相应的日志服务配置")]
    RegionNotConfigured(String),

    #[error("认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("缺少认证凭据: {0}")]
    MissingCredentials(String),

    #[error("日志查询失败 [区域: {0}]: {1}")]
    QueryFailed(String, #[source] anyhow::Error),

    #[error("网络请求失败: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON 解析失败: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("正则表达式错误: {0}")]
    RegexError(#[from] regex::Error),

    #[error("环境变量错误: {0}")]
    EnvError(#[from] dotenvy::Error),

    #[error("环境配置文件未找到: {0}")]
    #[allow(dead_code)]
    EnvFileNotFound(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("过滤配置文件格式错误: {0}")]
    FilterConfigError(String),

    #[error("内部错误: {0}")]
    InternalError(String),
}