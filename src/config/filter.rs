//! 消息过滤配置模块

use crate::conditional_info;
use crate::error::LogidError;
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::warn;

/// 过滤配置
#[derive(Debug, Clone, Deserialize)]
pub struct FilterConfig {
    /// 消息过滤规则列表
    #[serde(rename = "msg_filters", alias = "_msg_filters", alias = "patterns")]
    pub msg_filters: Option<Vec<String>>,
}

impl FilterConfig {
    /// 从文件加载过滤配置
    pub fn from_file(path: &PathBuf) -> Result<Option<Self>, LogidError> {
        if !path.exists() {
            conditional_info!("过滤配置文件不存在: {}", path.display());
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;

        // 尝试解析不同格式的配置
        if let Some(filters) = config.get("msg_filters") {
            Ok(Some(FilterConfig {
                msg_filters: Some(serde_json::from_value(filters.clone())?),
            }))
        } else if let Some(filters) = config.get("_msg_filters") {
            Ok(Some(FilterConfig {
                msg_filters: Some(serde_json::from_value(filters.clone())?),
            }))
        } else if let Some(filters) = config.get("patterns") {
            Ok(Some(FilterConfig {
                msg_filters: Some(serde_json::from_value(filters.clone())?),
            }))
        } else {
            warn!("过滤配置文件格式不正确，缺少有效的过滤规则字段");
            Ok(None)
        }
    }

    /// 获取过滤规则列表，如果配置为空则返回默认规则
    pub fn get_filters(&self) -> Vec<String> {
        self.msg_filters
            .clone()
            .unwrap_or_else(get_default_filters)
    }
}

/// 获取默认的过滤规则
pub fn get_default_filters() -> Vec<String> {
    vec![
        "_compliance_nlp_log".to_string(),
        "_compliance_whitelist_log".to_string(),
        "_compliance_source=footprint".to_string(),
        r#"(?s)"user_extra":\s*"\{.*?\}""#.to_string(),
        r#"(?m)"LogID":\s*"[^"]*""#.to_string(),
        r#"(?m)"Addr":\s*"[^"]*""#.to_string(),
        r#"(?m)"Client":\s*"[^"]*""#.to_string(),
    ]
}

/// 创建消息过滤器
pub fn create_message_filters(
    config_path: Option<&PathBuf>,
) -> Result<Vec<Regex>, LogidError> {
    let patterns = if let Some(path) = config_path {
        match FilterConfig::from_file(path)? {
            Some(config) => {
                conditional_info!("从配置文件加载过滤规则: {}", path.display());
                config.get_filters()
            }
            None => {
                conditional_info!("使用默认过滤规则");
                get_default_filters()
            }
        }
    } else {
        // 尝试从项目根目录加载配置文件
        let default_path = PathBuf::from("reference/message_filters.json");
        match FilterConfig::from_file(&default_path)? {
            Some(config) => {
                conditional_info!("从默认配置文件加载过滤规则: {}", default_path.display());
                config.get_filters()
            }
            None => {
                conditional_info!("使用默认过滤规则");
                get_default_filters()
            }
        }
    };

    // 预编译正则表达式
    let mut compiled_filters = Vec::new();
    for pattern in patterns {
        let regex = Regex::new(&pattern)
            .map_err(|e| LogidError::FilterConfigError(format!("无效的正则表达式 '{}': {}", pattern, e)))?;
        compiled_filters.push(regex);
    }

    conditional_info!("已加载 {} 条消息过滤规则", compiled_filters.len());
    Ok(compiled_filters)
}
