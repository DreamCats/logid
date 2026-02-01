//! 多区域日志查询模块

use crate::conditional_info;
use crate::auth::MultiRegionAuthManager;
use crate::error::LogidError;
use crate::log_query::client::LogQueryClient;
use crate::log_query::types::*;
use std::collections::HashMap;

/// 多区域日志查询器
///
/// 管理多个区域的日志查询客户端，提供统一的查询接口。
#[derive(Debug)]
#[allow(dead_code)]
pub struct MultiRegionLogQuery {
    /// 多区域认证管理器
    #[allow(dead_code)]
    auth_manager: MultiRegionAuthManager,
    /// 查询客户端映射
    clients: HashMap<String, LogQueryClient>,
}

#[allow(dead_code)]
impl MultiRegionLogQuery {
    /// 创建新的多区域日志查询器
    pub async fn new(regions: &[&str]) -> Result<Self, LogidError> {
        let auth_manager = MultiRegionAuthManager::new(regions)?;
        let mut clients = HashMap::new();

        for region in regions {
            let region_config = crate::config::get_region_config(region)
                .ok_or_else(|| LogidError::UnsupportedRegion(region.to_string()))?;

            let auth = auth_manager.get_manager(region).ok_or_else(|| {
                LogidError::AuthenticationFailed(format!("未找到 {} 区域的认证管理器", region))
            })?;

            let client = LogQueryClient::new(auth.as_ref().clone(), region_config).await?;
            clients.insert(region.to_string(), client);
        }

        conditional_info!("多区域日志查询器初始化完成，共 {} 个区域", clients.len());
        Ok(Self {
            auth_manager,
            clients,
        })
    }

    /// 获取指定区域的查询客户端
    pub fn get_client(&self, region: &str) -> Option<&LogQueryClient> {
        self.clients.get(region)
    }

    /// 在指定区域查询日志
    pub async fn query_logs_region(
        &self,
        region: &str,
        logid: &str,
        psm_list: &[String],
    ) -> Result<LogQueryResponse, LogidError> {
        let client = self.clients.get(region).ok_or_else(|| {
            LogidError::UnsupportedRegion(format!("未找到 {} 区域的查询客户端", region))
        })?;

        client.query_logs(logid, psm_list).await
    }

    /// 获取指定区域的详细日志信息
    pub async fn get_log_details_region(
        &self,
        region: &str,
        logid: &str,
        psm_list: &[String],
    ) -> Result<DetailedLogResult, LogidError> {
        let client = self.clients.get(region).ok_or_else(|| {
            LogidError::UnsupportedRegion(format!("未找到 {} 区域的查询客户端", region))
        })?;

        client.get_log_details(logid, psm_list).await
    }

    /// 获取所有已管理的区域列表
    pub fn managed_regions(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }
}
