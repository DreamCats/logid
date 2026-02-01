//! JWT 认证信息模块

use std::time::{Duration, Instant};

/// JWT 认证信息
#[derive(Debug, Clone)]
pub struct JwtInfo {
    /// JWT 令牌
    pub token: String,
    /// 过期时间
    pub expires_at: Instant,
}

impl JwtInfo {
    /// 创建新的 JWT 信息
    pub fn new(token: String, expires_in_seconds: u64) -> Self {
        Self {
            token,
            expires_at: Instant::now() + Duration::from_secs(expires_in_seconds),
        }
    }

    /// 检查令牌是否有效（5 分钟缓冲时间）
    pub fn is_valid(&self) -> bool {
        Instant::now() < (self.expires_at - Duration::from_secs(300))
    }
}
