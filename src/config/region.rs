//! 区域配置模块

use tracing::warn;

/// 区域标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    /// 中国区
    Cn,
    /// 国际化区域（新加坡）
    I18n,
    /// 美区
    Us,
}

impl Region {
    /// 从字符串解析区域
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(region: &str) -> Option<Self> {
        match region.to_lowercase().as_str() {
            "cn" => Some(Self::Cn),
            "i18n" => Some(Self::I18n),
            "us" => Some(Self::Us),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cn => "cn",
            Self::I18n => "i18n",
            Self::Us => "us",
        }
    }

    /// 获取区域特定的 CAS_SESSION 环境变量名
    pub fn cas_session_env_var(&self) -> &'static str {
        match self {
            Self::Cn => "CAS_SESSION_CN",
            Self::I18n => "CAS_SESSION_I18n",
            Self::Us => "CAS_SESSION_US",
        }
    }

    /// 获取区域显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Cn => "中国区",
            Self::I18n => "国际化区域（新加坡）",
            Self::Us => "美区",
        }
    }
}

/// 区域配置信息
#[derive(Debug, Clone)]
pub struct RegionConfig {
    /// 区域标识符
    #[allow(dead_code)]
    pub region: Region,
    /// 日志服务 URL
    pub log_service_url: String,
    /// 虚拟区域
    pub vregion: String,
    /// 可用区域列表
    #[allow(dead_code)]
    pub zones: Vec<String>,
    /// 是否已配置（cn 区域可能未配置）
    pub configured: bool,
}

impl RegionConfig {
    /// 创建区域配置
    pub fn new(
        region: Region,
        log_service_url: String,
        vregion: String,
        zones: Vec<String>,
    ) -> Self {
        Self {
            region,
            log_service_url,
            vregion,
            zones,
            configured: true,
        }
    }

    /// 创建未配置的区域（主要用于 cn 区域）
    pub fn unconfigured(region: Region) -> Self {
        Self {
            region,
            log_service_url: String::new(),
            vregion: String::new(),
            zones: Vec::new(),
            configured: false,
        }
    }

    /// 检查是否已配置
    pub fn is_configured(&self) -> bool {
        self.configured
    }
}

/// 获取区域配置
pub fn get_region_config(region_str: &str) -> Option<RegionConfig> {
    let region = Region::from_str(region_str)?;

    match region {
        Region::Cn => {
            // CN 区域暂未提供配置，返回未配置状态
            warn!("CN 区域日志服务配置待补充");
            Some(RegionConfig::unconfigured(Region::Cn))
        }
        Region::I18n => {
            Some(RegionConfig::new(
                Region::I18n,
                "https://logservice-sg.tiktok-row.org/streamlog/platform/microservice/v1/query/trace".to_string(),
                "Singapore-Common,US-East,Singapore-Central".to_string(),
                vec![
                    "Singapore-Common".to_string(),
                    "US-East".to_string(),
                    "Singapore-Central".to_string(),
                ],
            ))
        }
        Region::Us => {
            Some(RegionConfig::new(
                Region::Us,
                "https://logservice-tx.tiktok-us.org/streamlog/platform/microservice/v1/query/trace".to_string(),
                "US-TTP,US-TTP2".to_string(),
                vec!["US-TTP".to_string(), "US-TTP2".to_string()],
            ))
        }
    }
}
