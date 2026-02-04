//! 环境变量管理模块

use crate::conditional_info;
use crate::config::Region;
use crate::error::LogidError;
use std::collections::HashMap;

/// 用户配置目录名称
const USER_CONFIG_DIR: &str = ".config/logid";
/// 环境变量文件名
const ENV_FILE_NAME: &str = ".env";

/// 环境变量管理器
#[derive(Debug)]
pub struct EnvManager {
    env_vars: HashMap<String, String>,
}

impl EnvManager {
    /// 创建环境变量管理器，自动加载 .env 文件
    pub fn new() -> Result<Self, LogidError> {
        // 获取可执行文件所在目录
        let exe_path = std::env::current_exe()
            .map_err(|e| LogidError::InternalError(format!("获取可执行文件路径失败: {}", e)))?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| LogidError::InternalError("无法确定可执行文件目录".to_string()))?;

        // 构建可执行文件同级目录的 .env 文件路径
        let exe_env_path = exe_dir.join(ENV_FILE_NAME);

        // 构建用户级别目录的 .env 文件路径 (~/.config/logid/.env)
        let user_env_path = dirs::home_dir()
            .map(|home| home.join(USER_CONFIG_DIR).join(ENV_FILE_NAME))
            .ok_or_else(|| LogidError::InternalError("无法确定用户主目录".to_string()))?;

        let mut env_loaded = false;

        // 优先尝试加载可执行文件同级目录的 .env 文件
        if exe_env_path.exists() {
            match dotenvy::from_path(&exe_env_path) {
                Ok(_) => {
                    conditional_info!("成功加载 .env 文件: {}", exe_env_path.display());
                    env_loaded = true;
                }
                Err(e) => {
                    conditional_info!("加载可执行文件同级目录的 .env 文件失败: {} - {}", exe_env_path.display(), e);
                }
            }
        }

        // 如果可执行文件目录没有 .env 文件，尝试用户级别目录
        if !env_loaded && user_env_path.exists() {
            match dotenvy::from_path(&user_env_path) {
                Ok(_) => {
                    conditional_info!("成功加载用户级别 .env 文件: {}", user_env_path.display());
                    env_loaded = true;
                }
                Err(e) => {
                    conditional_info!("加载用户级别 .env 文件失败: {} - {}", user_env_path.display(), e);
                }
            }
        }

        // 如果两个位置都没有找到 .env 文件，显示友好的警告和设置指导
        if !env_loaded {
            eprintln!("⚠️  未找到 .env 配置文件");
            eprintln!("   搜索位置:");
            eprintln!("   1. {}", exe_env_path.display());
            eprintln!("   2. {}", user_env_path.display());
            eprintln!("   请在以上任一位置创建 .env 文件并配置以下内容：");
            eprintln!("   CAS_SESSION_US=your_us_session_cookie_here");
            eprintln!("   CAS_SESSION_I18n=your_i18n_session_cookie_here");
            eprintln!("   ENABLE_LOGGING=false");
            eprintln!("   详细配置请参考项目文档");
        }

        let mut env_vars = HashMap::new();

        // 收集所有环境变量
        for (key, value) in std::env::vars() {
            env_vars.insert(key, value);
        }

        Ok(Self { env_vars })
    }

    /// 获取区域的 CAS_SESSION 值
    /// 优先使用区域特定的环境变量，然后回退到通用的 CAS_SESSION
    pub fn get_cas_session(&self, region: Region) -> Result<String, LogidError> {
        let region_var = region.cas_session_env_var();

        // 优先使用区域特定的环境变量
        if let Some(session) = self.env_vars.get(region_var) {
            if !session.is_empty() {
                conditional_info!("使用区域特定的 CAS_SESSION: {}", region_var);
                return Ok(session.clone());
            }
        }

        // 回退到通用的 CAS_SESSION
        if let Some(session) = self.env_vars.get("CAS_SESSION") {
            if !session.is_empty() {
                conditional_info!("使用通用的 CAS_SESSION (回退)");
                return Ok(session.clone());
            }
        }

        Err(LogidError::MissingCredentials(format!(
            "未找到 {} 或 CAS_SESSION 环境变量",
            region_var
        )))
    }

    /// 获取任意环境变量
    #[allow(dead_code)]
    pub fn get_env(&self, key: &str) -> Option<String> {
        self.env_vars.get(key).cloned()
    }
}
