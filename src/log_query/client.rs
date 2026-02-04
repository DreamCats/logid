//! 日志查询客户端模块

use crate::conditional_info;
use crate::auth::AuthManager;
use crate::config::{create_message_filters, RegionConfig};
use crate::error::LogidError;
use crate::log_query::types::*;
use regex::Regex;
use std::time::Instant;
use tracing::{error, warn};

/// 从环境变量获取代理地址
fn get_proxy_from_env() -> Option<reqwest::Proxy> {
    // 优先使用 HTTPS_PROXY
    if let Ok(proxy) = std::env::var("HTTPS_PROXY") {
        if !proxy.is_empty() {
            if let Ok(p) = reqwest::Proxy::https(&proxy) {
                return Some(p);
            }
        }
    }
    // 其次使用 HTTP_PROXY
    if let Ok(proxy) = std::env::var("HTTP_PROXY") {
        if !proxy.is_empty() {
            if let Ok(p) = reqwest::Proxy::http(&proxy) {
                return Some(p);
            }
        }
    }
    None
}

/// 日志查询客户端
///
/// 提供基于 JWT 认证的多区域日志查询功能，支持美区和国际化区域的并发查询。
/// 该结构体封装了日志服务的 API 调用，提供统一的日志查询接口。
#[derive(Debug)]
pub struct LogQueryClient {
    /// 认证管理器
    auth_manager: AuthManager,
    /// 区域配置
    region_config: RegionConfig,
    /// 消息过滤器列表
    message_filters: Vec<Regex>,
    /// HTTP 客户端
    client: reqwest::Client,
}

impl LogQueryClient {
    /// 创建新的日志查询客户端
    pub async fn new(
        auth_manager: AuthManager,
        region_config: RegionConfig,
    ) -> Result<Self, LogidError> {
        // 创建消息过滤器
        let message_filters = create_message_filters(None)?;

        // 配置 HTTP 客户端
        let mut client_builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Edg/140.0.0.0")
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT,
                    "application/json, text/plain, */*".parse().unwrap(),
                );
                headers.insert(
                    reqwest::header::ACCEPT_LANGUAGE,
                    "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap(),
                );
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                headers
            });

        // 添加代理配置
        if let Some(proxy) = get_proxy_from_env() {
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder
            .build()
            .map_err(|e| LogidError::InternalError(format!("创建 HTTP 客户端失败: {}", e)))?;

        conditional_info!(
            "创建日志查询客户端: region={}, url={}",
            auth_manager.region_str(),
            region_config.log_service_url
        );

        Ok(Self {
            auth_manager,
            region_config,
            message_filters,
            client,
        })
    }

    /// 根据 logid 查询日志
    pub async fn query_logs(
        &self,
        logid: &str,
        psm_list: &[String],
    ) -> Result<LogQueryResponse, LogidError> {
        // 检查区域是否配置
        if !self.region_config.is_configured() {
            return Err(LogidError::RegionNotConfigured(
                self.auth_manager.region_str().to_string(),
            ));
        }

        let start_time = Instant::now();
        conditional_info!(
            "开始查询日志: logid={}, region={}, psm_list={:?}",
            logid,
            self.auth_manager.region_str(),
            psm_list
        );

        // 获取 JWT 令牌
        let jwt_token = self.auth_manager.get_jwt_token(false).await.map_err(|e| {
            LogidError::AuthenticationFailed(format!(
                "获取 {} 区域 JWT 令牌失败: {}",
                self.auth_manager.region_str(),
                e
            ))
        })?;

        // 准备请求体
        let request_body = LogQueryRequest::new(
            logid.to_string(),
            psm_list.to_vec(),
            10, // 固定 10 分钟扫描范围
            self.region_config.vregion.clone(),
        );

        // 发送 HTTP POST 请求到日志服务 API
        let response = self
            .client
            .post(&self.region_config.log_service_url)
            .header("X-Jwt-Token", jwt_token.as_str())
            .header("accept", "application/json, text/plain, */*")
            .header("Content-Type", "application/json")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Edg/140.0.0.0")
            .json(&request_body)
            .send()
            .await?;

        let elapsed = start_time.elapsed();
        conditional_info!(
            "日志查询请求完成: status={}, elapsed={:?}",
            response.status(),
            elapsed
        );

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "日志查询失败: status={}, body={}",
                status,
                error_text
            );
            return Err(LogidError::QueryFailed(
                self.auth_manager.region_str().to_string(),
                anyhow::anyhow!("HTTP {}: {}", status, error_text),
            ));
        }

        // 解析响应数据
        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            LogidError::NetworkError(e)
        })?;

        // 尝试解析不同的响应格式
        let data = if let Some(outer_data) = response_data.get("data") {
            if let Some(_items) = outer_data.get("items") {
                outer_data.clone()
            } else if outer_data.get("items").is_none() && response_data.get("items").is_some() {
                response_data.clone()
            } else {
                outer_data.clone()
            }
        } else if response_data.get("items").is_some() {
            response_data.clone()
        } else {
            warn!("响应中未找到预期的 data 或 items 字段，返回空数据");
            serde_json::json!({"items": []})
        };

        let meta = response_data.get("meta").cloned();
        let tag_infos = response_data.get("tag_infos").cloned();

        let result = LogQueryResponse {
            data: Some(serde_json::from_value(data.clone()).map_err(|e| {
                error!("解析日志数据失败: {}, 原始数据: {}", e, serde_json::to_string(&data).unwrap_or_default());
                LogidError::JsonParseError(e)
            })?),
            meta,
            tag_infos: tag_infos.and_then(|v| serde_json::from_value(v).ok()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            region: self.auth_manager.region_str().to_string(),
            region_display_name: self.auth_manager.region().display_name().to_string(),
        };

        let items_count = result.data.as_ref().map(|data| data.items.len()).unwrap_or(0);
        conditional_info!(
            "日志查询完成: region={}, logid={}, items_found={}, elapsed={:?}",
            self.auth_manager.region_str(),
            logid,
            items_count,
            elapsed
        );

        Ok(result)
    }

    /// 获取详细的日志信息
    pub async fn get_log_details(
        &self,
        logid: &str,
        psm_list: &[String],
    ) -> Result<DetailedLogResult, LogidError> {
        let response = self.query_logs(logid, psm_list).await?;

        let data = response.data.as_ref().ok_or_else(|| {
            LogidError::QueryFailed(
                self.auth_manager.region_str().to_string(),
                anyhow::anyhow!("响应中没有数据内容"),
            )
        })?;

        let messages = self.extract_log_messages(data);
        let meta = data.meta.clone();
        let tag_infos = response.tag_infos.clone();

        Ok(DetailedLogResult {
            logid: logid.to_string(),
            messages,
            meta: meta.clone(),
            tag_infos,
            total_items: data.items.len(),
            scan_time_range: meta.as_ref().and_then(|m| m.scan_time_range.clone()),
            level_list: meta.as_ref().and_then(|m| m.level_list.clone()),
            timestamp: response.timestamp,
            region: response.region,
            region_display_name: response.region_display_name,
        })
    }

    /// 从 API 响应中提取日志消息
    pub fn extract_log_messages(&self, data: &LogData) -> Vec<ExtractedLogMessage> {
        let mut messages = Vec::new();

        for item in &data.items {
            for value in &item.value {
                let mut extracted_values = Vec::new();
                let mut location = None;
                let level = value.level.clone();

                for kv in &value.kv_list {
                    if kv.key == "_msg" {
                        let filtered_value = self.filter_message_content(&kv.value);
                        extracted_values.push(ExtractedValue {
                            key: kv.key.clone(),
                            value: filtered_value,
                            original_value: kv.value.clone(),
                            type_field: kv.type_field.clone(),
                            highlight: kv.highlight.unwrap_or(false),
                        });
                    } else if kv.key == "_location" {
                        location = Some(kv.value.clone());
                    }
                }

                if !extracted_values.is_empty() {
                    messages.push(ExtractedLogMessage {
                        id: format!("{}-{}", item.id, value.id),
                        group: item.group.clone(),
                        values: extracted_values,
                        location,
                        level,
                    });
                }
            }
        }

        conditional_info!("提取了 {} 条日志消息", messages.len());
        messages
    }

    /// 过滤消息内容中的冗余字段
    fn filter_message_content(&self, message: &str) -> String {
        let mut filtered = message.to_string();

        for regex in &self.message_filters {
            filtered = regex.replace_all(&filtered, "").to_string();
        }

        // 清理多余空格和换行符
        filtered = regex::Regex::new(r"[ \t]{2,}")
            .map(|re| re.replace_all(&filtered, " ").to_string())
            .unwrap_or(filtered.clone());

        filtered = regex::Regex::new(r"\n\s*\n\s*\n")
            .map(|re| re.replace_all(&filtered, "\n\n").to_string())
            .unwrap_or(filtered);

        filtered.trim().to_string()
    }

    /// 获取区域信息
    #[allow(dead_code)]
    pub fn region(&self) -> &str {
        self.auth_manager.region_str()
    }

    /// 获取区域配置
    #[allow(dead_code)]
    pub fn region_config(&self) -> &RegionConfig {
        &self.region_config
    }
}
