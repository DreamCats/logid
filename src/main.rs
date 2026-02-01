//! # 字节跳动 LogID 查询工具
//!
//! 这是一个基于 Rust 开发的命令行工具，用于通过 logid 查询字节跳动内部日志服务。
//! 支持多区域（us/i18n/cn）查询、PSM 过滤，输出 JSON 格式。

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::error;

// 使用库中的模块
use logid::{
    auth, config, error::LogidError, log_query, output,
    conditional_info,
};

mod commands;

#[derive(Parser)]
#[command(name = "logid")]
#[command(about = "字节跳动 logid 查询工具", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(
        about = "查询日志",
        long_about = "通过 logid 查询字节跳动内部日志服务\n\n示例:\n  logid query '550e8400-e29b-41d4-a716-446655440000' --region us\n  logid query 'logid123' --region i18n --psm service.psm\n  logid query 'logid456' --region us --psm psm1 --psm psm2\n\n参数说明:\n  - logid: 要查询的日志 ID，通常是 UUID 格式\n  - region: 查询区域 (cn/i18n/us)\n  - psm: 过滤的 PSM 服务名称，可多次指定\n\n区域说明:\n  * us: 美区 (https://logservice-tx.tiktok-us.org)\n  * i18n: 国际化区域 (https://logservice-sg.tiktok-row.org)\n  * cn: 中国区 (需要特殊配置)\n\n认证说明:\n  需要在环境变量中配置对应区域的 CAS_SESSION:\n  - CAS_SESSION_US: 美区认证凭据\n  - CAS_SESSION_I18n: 国际化区域认证凭据\n  - CAS_SESSION_CN: 中国区认证凭据"
    )]
    Query {
        /// 要查询的日志 ID
        logid: String,
        /// 查询区域 (cn/i18n/us)
        #[arg(short, long)]
        region: String,
        /// 过滤的 PSM 服务名称
        #[arg(short, long)]
        psm: Vec<String>,
    },
    #[command(
        about = "更新 logid 到最新版本",
        long_about = "更新 logid 到最新版本\n\n示例:\n  logid update\n  logid update --check\n  logid update --force\n\n参数说明:\n  - check: 仅检查是否有新版本，不执行更新\n  - force: 强制更新，即使当前已是最新版本\n\n更新流程:\n  1. 从 GitHub 获取最新版本信息\n  2. 比较当前版本与最新版本\n  3. 下载对应平台的二进制文件\n  4. 验证文件完整性（SHA256）\n  5. 备份当前版本并替换文件\n\n注意事项:\n  - 需要网络连接\n  - 需要文件写入权限\n  - 更新前会自动备份当前版本\n  - 支持 Linux/macOS/Windows 平台"
    )]
    Update {
        /// 仅检查更新，不执行下载和安装
        #[arg(long)]
        check: bool,
        /// 强制更新，即使当前已是最新版本
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 检查是否启用日志，默认关闭
    let logging_enabled = std::env::var("ENABLE_LOGGING")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    let should_log = matches!(logging_enabled.as_str(), "true" | "on" | "1" | "yes");

    if should_log {
        tracing_subscriber::fmt::init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_ansi(false)
            .compact()
            .init();
    }

    let cli = Cli::parse();

    match run_command(cli.command).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("执行失败: {}", e);
            print_error(&e);
            Err(e)
        }
    }
}

async fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Query { logid, region, psm } => {
            conditional_info!("开始查询日志: logid={}, region={}, psm_list={:?}", logid, region, psm);
            run_query(&logid, &region, &psm).await
        }
        Commands::Update { check, force } => {
            commands::update::update_command(check, force).await
        }
    }
}

/// 执行日志查询的主要逻辑
async fn run_query(
    logid: &str,
    region: &str,
    psm_list: &[String],
) -> Result<()> {
    // 检查区域配置
    let region_config = config::get_region_config(region)
        .ok_or_else(|| LogidError::UnsupportedRegion(region.to_string()))?;

    // 如果是 cn 区域且未配置，显示友好错误
    if region == "cn" && !region_config.is_configured() {
        return Err(LogidError::RegionNotConfigured(region.to_string()).into());
    }

    // 创建认证管理器
    let auth_manager = auth::AuthManager::new(region)?;

    conditional_info!("创建日志查询客户端...");
    let log_client = log_query::LogQueryClient::new(auth_manager, region_config).await?;

    conditional_info!("开始查询日志...");
    let query_response = log_client.query_logs(logid, psm_list).await?;

    conditional_info!("提取日志消息...");
    let data = query_response.data.as_ref().ok_or_else(|| {
        anyhow::anyhow!("响应中没有数据内容")
    })?;

    // 使用 LogQueryClient 的 extract_log_messages 方法提取消息
    let extracted_messages = log_client.extract_log_messages(data);

    conditional_info!("格式化输出结果...");
    let output_config = output::OutputConfig::new();
    let formatter = output::OutputFormatter::new(output_config);

    // 创建 DetailedLogResult 结构
    let data_items = data.items.len();
    let log_details = log_query::DetailedLogResult {
        logid: logid.to_string(),
        region: region.to_string(),
        messages: extracted_messages,
        scan_time_range: None,
        meta: query_response.data.and_then(|d| d.meta),
        tag_infos: query_response.tag_infos,
        total_items: data_items,
        level_list: None,
        timestamp: query_response.timestamp,
        region_display_name: query_response.region_display_name,
    };

    let formatted = formatter.format_log_result(&log_details)?;
    println!("{}", formatted);

    Ok(())
}

/// 打印友好的错误信息
fn print_error(error: &anyhow::Error) {
    if let Some(logid_error) = error.downcast_ref::<LogidError>() {
        match logid_error {
            LogidError::UnsupportedRegion(region) => {
                eprintln!("不支持的区域: {}", region);
                eprintln!("支持的区域: cn, i18n, us");
            }
            LogidError::RegionNotConfigured(region) => {
                eprintln!("区域 {} 尚未配置日志服务", region);
                eprintln!("请联系相关团队获取配置信息");
            }
            LogidError::MissingCredentials(var) => {
                eprintln!("缺少认证凭据: {}", var);
                eprintln!("请在环境变量或 .env 文件中设置相应的 CAS_SESSION");
                eprintln!("例如: export CAS_SESSION_US=your_session_cookie");
            }
            LogidError::AuthenticationFailed(msg) => {
                eprintln!("认证失败: {}", msg);
                eprintln!("请检查 CAS_SESSION 是否有效或网络连接是否正常");
            }
            LogidError::NetworkError(e) => {
                eprintln!("网络请求失败: {}", e);
                eprintln!("请检查网络连接和防火墙设置");
            }
            LogidError::QueryFailed(region, source) => {
                eprintln!("区域 {} 查询失败: {}", region, source);
                eprintln!("请检查日志 ID 是否正确或稍后重试");
            }
            _ => {
                eprintln!("发生错误: {}", error);
            }
        }
    } else {
        eprintln!("未知错误: {}", error);
    }
}
