//! JWT 认证模块
//!
//! 处理字节跳动内部 API 的 JWT 令牌获取和管理，支持多区域认证配置。
//! 提供基于 Cookie 的 JWT 认证功能，支持自动令牌刷新和过期检测。

mod manager;
mod multi_region;

pub use manager::AuthManager;
pub use multi_region::MultiRegionAuthManager;

#[cfg(test)]
mod tests {
    use crate::config::{JwtInfo, Region};

    #[test]
    fn test_region_from_str() {
        assert_eq!(Region::from_str("cn"), Some(Region::Cn));
        assert_eq!(Region::from_str("I18N"), Some(Region::I18n));
        assert_eq!(Region::from_str("us"), Some(Region::Us));
        assert_eq!(Region::from_str("invalid"), None);
    }

    #[test]
    fn test_region_as_str() {
        assert_eq!(Region::Cn.as_str(), "cn");
        assert_eq!(Region::I18n.as_str(), "i18n");
        assert_eq!(Region::Us.as_str(), "us");
    }

    #[test]
    fn test_jwt_info_validity() {
        // 测试有效的 JWT 信息
        let jwt_info = JwtInfo::new("test_token".to_string(), 3600);
        assert!(jwt_info.is_valid());

        // 测试即将过期的 JWT 信息
        let jwt_info = JwtInfo::new("test_token".to_string(), 200); // 不到 5 分钟
        assert!(!jwt_info.is_valid());
    }
}
