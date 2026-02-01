//! 多区域认证管理模块

use crate::auth::AuthManager;
use crate::error::LogidError;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// 多区域认证管理器
///
/// 管理多个区域的 JWT 认证，提供统一的认证接口。
#[derive(Debug)]
#[allow(dead_code)]
pub struct MultiRegionAuthManager {
    /// 区域认证管理器映射
    managers: HashMap<String, Arc<AuthManager>>,
}

#[allow(dead_code)]
impl MultiRegionAuthManager {
    /// 创建多区域认证管理器
    ///
    /// # 参数
    /// - `regions`: 要初始化的区域列表
    ///
    /// # 返回
    /// - `Result<Self, LogidError>`: 创建的管理器或错误
    pub fn new(regions: &[&str]) -> Result<Self, LogidError> {
        let mut managers = HashMap::new();

        for &region in regions {
            let manager = AuthManager::new(region)
                .map_err(|e| LogidError::AuthenticationFailed(format!("初始化 {} 区域认证失败: {}", region, e)))?;
            managers.insert(region.to_string(), Arc::new(manager));
            info!("已初始化 {} 区域认证管理器", region);
        }

        info!("多区域认证管理器初始化完成，共 {} 个区域", managers.len());
        Ok(Self { managers })
    }

    /// 获取指定区域的认证管理器
    pub fn get_manager(&self, region: &str) -> Option<Arc<AuthManager>> {
        self.managers.get(region).cloned()
    }

    /// 获取指定区域的 JWT 令牌
    pub async fn get_jwt_token(&self, region: &str, force_refresh: bool) -> Result<String, LogidError> {
        let manager = self.get_manager(region).ok_or_else(|| {
            LogidError::AuthenticationFailed(format!("未找到 {} 区域的认证管理器", region))
        })?;

        manager.get_jwt_token(force_refresh).await
    }

    /// 检查指定区域的令牌是否有效
    pub async fn is_token_valid(&self, region: &str) -> Result<bool, LogidError> {
        let manager = self.get_manager(region).ok_or_else(|| {
            LogidError::AuthenticationFailed(format!("未找到 {} 区域的认证管理器", region))
        })?;

        Ok(manager.is_token_valid().await)
    }

    /// 刷新指定区域的令牌
    pub async fn refresh_token(&self, region: &str) -> Result<String, LogidError> {
        let manager = self.get_manager(region).ok_or_else(|| {
            LogidError::AuthenticationFailed(format!("未找到 {} 区域的认证管理器", region))
        })?;

        manager.refresh_token().await
    }

    /// 刷新所有区域的令牌
    pub async fn refresh_all_tokens(&self) -> HashMap<String, Result<String, LogidError>> {
        let mut results = HashMap::new();

        for (region, manager) in &self.managers {
            let result = manager.refresh_token().await;
            results.insert(region.clone(), result);
        }

        results
    }

    /// 获取所有已管理的区域列表
    pub fn managed_regions(&self) -> Vec<String> {
        self.managers.keys().cloned().collect()
    }
}
