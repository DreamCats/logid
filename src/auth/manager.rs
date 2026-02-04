//! JWT 认证管理器模块

use crate::conditional_info;
use crate::config::{EnvManager, JwtInfo, Region};
use crate::error::LogidError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

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

/// 区域 JWT 认证端点配置
const REGION_AUTH_URLS: &[(Region, &str)] = &[
    (Region::Cn, "https://cloud.bytedance.net/auth/api/v1/jwt"),
    (Region::I18n, "https://cloud-i18n.bytedance.net/auth/api/v1/jwt"),
    (Region::Us, "https://cloud-ttp-us.bytedance.net/auth/api/v1/jwt"),
    (Region::Eu, "https://cloud-i18n.tiktok-eu.org/auth/api/v1/jwt"),
];

/// JWT 认证管理器
///
/// 提供字节跳动内部 API 的 JWT 令牌管理功能，支持多区域认证配置。
/// 该结构体负责获取、缓存和刷新 JWT 令牌，支持基于 Cookie 的认证方式。
#[derive(Debug, Clone)]
pub struct AuthManager {
    /// 区域标识符
    region: Region,
    /// HTTP 客户端
    client: reqwest::Client,
    /// 缓存的 JWT 信息
    jwt_cache: Arc<RwLock<Option<JwtInfo>>>,
    /// CAS_SESSION Cookie 值
    cas_session: String,
    /// 认证 URL
    auth_url: String,
}

impl AuthManager {
    /// 创建新的认证管理器
    ///
    /// # 参数
    /// - `region`: 区域标识符 ("cn"、"i18n"、"us")
    ///
    /// # 返回
    /// - `Result<Self, LogidError>`: 创建的认证管理器或错误
    ///
    /// # 错误
    /// - 如果无法获取到有效的 Cookie 值
    /// - 如果 HTTP 客户端创建失败
    pub fn new(region: &str) -> Result<Self, LogidError> {
        let region = Region::from_str(region)
            .ok_or_else(|| LogidError::UnsupportedRegion(region.to_string()))?;

        // 加载环境变量
        let env_manager = EnvManager::new()?;

        // 获取 CAS_SESSION 值
        let cas_session = env_manager.get_cas_session(region)?;

        // 获取认证 URL
        let auth_url = REGION_AUTH_URLS
            .iter()
            .find(|(r, _)| *r == region)
            .map(|(_, url)| url.to_string())
            .unwrap_or_else(|| {
                // 默认使用中国区的 URL
                warn!("使用默认的中国区认证 URL，可能不是预期的");
                "https://cloud.bytedance.net/auth/api/v1/jwt".to_string()
            });

        // 配置 HTTP 客户端，模拟浏览器行为
        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
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
                    reqwest::header::ACCEPT_ENCODING,
                    "gzip, deflate, br, zstd".parse().unwrap(),
                );
                headers
            });

        // 添加代理配置
        if let Some(proxy) = get_proxy_from_env() {
            client_builder = client_builder.proxy(proxy);
            let proxy_url = std::env::var("HTTPS_PROXY")
                .or_else(|_| std::env::var("HTTP_PROXY"))
                .unwrap_or_default();
            conditional_info!("使用代理: {}", proxy_url);
        }

        let client = client_builder
            .build()
            .map_err(|e| LogidError::InternalError(format!("创建 HTTP 客户端失败: {}", e)))?;

        conditional_info!(
            "初始化 JWT 认证管理器: region={}, auth_url={}",
            region.as_str(),
            auth_url
        );

        Ok(Self {
            region,
            client,
            jwt_cache: Arc::new(RwLock::new(None)),
            cas_session,
            auth_url,
        })
    }

    /// 获取 JWT 令牌，必要时进行刷新
    ///
    /// 如果当前令牌有效且未强制刷新，则返回缓存的令牌。
    /// 否则，向认证服务请求新的 JWT 令牌。
    ///
    /// # 参数
    /// - `force_refresh`: 即使当前令牌有效也强制刷新
    ///
    /// # 返回
    /// - `Result<String, LogidError>`: JWT 令牌字符串或错误
    ///
    /// # 错误
    /// - 如果令牌获取失败
    /// - 如果网络请求失败
    pub async fn get_jwt_token(&self, force_refresh: bool) -> Result<String, LogidError> {
        // 检查缓存令牌是否有效
        if !force_refresh {
            {
                let cache = self.jwt_cache.read().await;
                if let Some(ref jwt_info) = *cache {
                    if jwt_info.is_valid() {
                        debug!("使用缓存的 JWT 令牌");
                        return Ok(jwt_info.token.clone());
                    }
                }
            }
        }

        // 获取新令牌
        conditional_info!("正在获取新的 JWT 令牌");
        let jwt_info = self.fetch_jwt_token().await?;

        // 更新缓存
        {
            let mut cache = self.jwt_cache.write().await;
            *cache = Some(jwt_info.clone());
        }

        conditional_info!("JWT 令牌获取成功");
        Ok(jwt_info.token)
    }

    /// 向认证服务获取新的 JWT 令牌
    async fn fetch_jwt_token(&self) -> Result<JwtInfo, LogidError> {
        // 准备认证请求头，包含 Cookie 信息
        let cookie_header = format!("CAS_SESSION={}", self.cas_session);

        let response = self
            .client
            .get(&self.auth_url)
            .header("Cookie", cookie_header)
            .send()
            .await?;

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "JWT 认证请求失败: status={}, body={}",
                status,
                error_text
            );
            return Err(LogidError::AuthenticationFailed(format!(
                "HTTP {}: {}",
                status,
                error_text
            )));
        }

        // 从响应头获取 JWT 令牌
        let jwt_token = response
            .headers()
            .get("x-jwt-token")
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| {
                LogidError::AuthenticationFailed("响应头中没有 JWT 令牌".to_string())
            })?;

        conditional_info!("JWT 令牌获取成功");
        Ok(JwtInfo::new(jwt_token.to_string(), 3600)) // 假设有效期为 1 小时
    }

    /// 检查当前令牌是否有效
    #[allow(dead_code)]
    pub async fn is_token_valid(&self) -> bool {
        let cache = self.jwt_cache.read().await;
        if let Some(ref jwt_info) = *cache {
            jwt_info.is_valid()
        } else {
            false
        }
    }

    /// 强制刷新令牌
    #[allow(dead_code)]
    pub async fn refresh_token(&self) -> Result<String, LogidError> {
        self.get_jwt_token(true).await
    }

    /// 获取区域信息
    pub fn region(&self) -> Region {
        self.region
    }

    /// 获取区域字符串表示
    pub fn region_str(&self) -> &'static str {
        self.region.as_str()
    }
}

impl Drop for AuthManager {
    fn drop(&mut self) {
        conditional_info!("销毁 JWT 认证管理器: region={}", self.region.as_str());
    }
}
