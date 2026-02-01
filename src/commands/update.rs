use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    published_at: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct UpdateResult {
    current_version: String,
    latest_version: String,
    updated: bool,
    message: String,
}

pub async fn update_command(check_only: bool, force: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("🔍 当前版本: {}", current_version);

    // 获取最新版本信息
    let release = get_latest_release().await?;
    let latest_version = release.tag_name.trim_start_matches('v');

    println!("🌟 最新版本: {}", latest_version);

    // 版本比较
    if !force && current_version >= latest_version {
        println!("✅ 当前已是最新版本！");
        return Ok(());
    }

    if check_only {
        if current_version < latest_version {
            println!("💡 有新版本可用，运行 'logid update' 进行更新");
        }
        return Ok(());
    }

    // 确认更新
    println!("📥 准备更新到版本: {}", latest_version);
    if !confirm_update()? {
        println!("❌ 更新已取消");
        return Ok(());
    }

    // 执行更新
    perform_update(&release).await?;

    Ok(())
}

async fn get_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::Client::builder()
        .user_agent("logid-update")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow::anyhow!("创建 HTTP 客户端失败: {}", e))?;

    let url = "https://api.github.com/repos/DreamCats/logid/releases/latest";

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("获取最新版本失败: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "获取最新版本失败，状态码: {}",
            response.status()
        ));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("解析版本信息失败: {}", e))?;

    Ok(release)
}

fn get_platform_asset(release: &GitHubRelease) -> Result<&GitHubAsset> {
    let platform = detect_platform();

    release
        .assets
        .iter()
        .find(|asset| asset.name.contains(&platform) && asset.name.contains("logid"))
        .ok_or_else(|| anyhow::anyhow!("找不到适用于 {} 平台的 logid 发布文件", platform))
}

fn detect_platform() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    // 匹配 workflow 生成的文件名格式
    match (os, arch) {
        ("linux", "x86_64") => "x86_64-unknown-linux".to_string(),
        ("linux", "aarch64") => "aarch64-unknown-linux".to_string(),
        ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
        ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
        ("windows", "x86_64") => "x86_64-pc-windows".to_string(),
        _ => format!("{}-{}", os, arch),
    }
}

fn confirm_update() -> Result<bool> {
    println!("是否继续更新？(y/N)");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("读取输入失败: {}", e))?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

async fn perform_update(release: &GitHubRelease) -> Result<()> {
    let asset = get_platform_asset(release)?;
    println!("📦 下载文件: {}", asset.name);

    // 获取当前可执行文件路径
    let current_exe = env::current_exe().map_err(|e| anyhow::anyhow!("获取当前路径失败: {}", e))?;
    let backup_path = current_exe.with_extension("backup");

    // 下载新文件
    let download_path = download_file(asset).await?;

    // 验证校验和（如果有的话）
    if let Err(e) = verify_checksum(&download_path, release).await {
        println!("⚠️  校验和验证失败: {}，但仍将继续更新", e);
    }

    // 备份当前文件
    println!("💾 备份当前文件...");
    fs::copy(&current_exe, &backup_path)
        .map_err(|e| anyhow::anyhow!("备份失败: {}", e))?;

    // 替换文件
    println!("🔄 替换文件...");
    replace_binary(&download_path, &current_exe)?;

    // 清理临时文件
    let _ = fs::remove_file(&download_path);

    println!("✅ 更新完成！");
    println!("💡 运行 'logid --version' 验证新版本");

    Ok(())
}

async fn download_file(asset: &GitHubAsset) -> Result<PathBuf> {
    let client = reqwest::Client::new();
    let response = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("下载失败: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "下载失败，状态码: {}",
            response.status()
        ));
    }

    let temp_dir = env::temp_dir();
    let file_name = asset.name.replace(".tar.gz", "").replace(".zip", "");
    let download_path = temp_dir.join(file_name);

    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("读取下载内容失败: {}", e))?;

    // 如果是压缩包，需要解压
    if asset.name.ends_with(".tar.gz") {
        extract_tar_gz(&bytes, &download_path)?;
    } else if asset.name.ends_with(".zip") {
        extract_zip(&bytes, &download_path)?;
    } else {
        // 直接写入文件
        let mut file = fs::File::create(&download_path)
            .map_err(|e| anyhow::anyhow!("创建文件失败: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| anyhow::anyhow!("写入文件失败: {}", e))?;
    }

    Ok(download_path)
}

fn extract_tar_gz(data: &[u8], output_path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let gz = GzDecoder::new(data);
    let mut archive = Archive::new(gz);

    for entry in archive.entries().map_err(|e| anyhow::anyhow!("解压失败: {}", e))? {
        let mut entry = entry.map_err(|e| anyhow::anyhow!("读取条目失败: {}", e))?;
        let path = entry.path().map_err(|e| anyhow::anyhow!("获取路径失败: {}", e))?;

        if path.file_name().unwrap_or_default().to_string_lossy().contains("logid") {
            entry.unpack(output_path).map_err(|e| anyhow::anyhow!("解压文件失败: {}", e))?;
            break;
        }
    }

    Ok(())
}

fn extract_zip(data: &[u8], output_path: &Path) -> Result<()> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let reader = Cursor::new(data);
    let mut archive = ZipArchive::new(reader).map_err(|e| anyhow::anyhow!("打开 zip 失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| anyhow::anyhow!("读取 zip 条目失败: {}", e))?;
        let name = file.name();

        if name.contains("logid") && !name.ends_with('/') {
            let mut output = fs::File::create(output_path)
                .map_err(|e| anyhow::anyhow!("创建输出文件失败: {}", e))?;
            std::io::copy(&mut file, &mut output)
                .map_err(|e| anyhow::anyhow!("复制文件失败: {}", e))?;
            break;
        }
    }

    Ok(())
}

async fn verify_checksum(download_path: &Path, release: &GitHubRelease) -> Result<()> {
    // 查找 SHA256SUMS 文件
    let checksum_asset = release
        .assets
        .iter()
        .find(|asset| asset.name == "SHA256SUMS")
        .ok_or_else(|| anyhow::anyhow!("找不到 SHA256SUMS 文件"))?;

    let client = reqwest::Client::new();
    let response = client
        .get(&checksum_asset.browser_download_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("下载校验和文件失败: {}", e))?;

    let checksums = response
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("读取校验和失败: {}", e))?;

    // 计算文件校验和
    let file_data = fs::read(download_path)
        .map_err(|e| anyhow::anyhow!("读取文件失败: {}", e))?;
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let file_checksum = format!("{:x}", hasher.finalize());

    // 验证校验和
    let file_name = download_path.file_name().unwrap().to_string_lossy();
    for line in checksums.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 && parts[1].contains(&*file_name) {
            if parts[0] == file_checksum {
                println!("✅ 校验和验证通过");
                return Ok(());
            } else {
                return Err(anyhow::anyhow!("校验和不匹配"));
            }
        }
    }

    Err(anyhow::anyhow!("找不到文件的校验和信息"))
}

fn replace_binary(source: &Path, target: &Path) -> Result<()> {
    // 设置新文件权限（仅在 Unix 系统上）
    #[cfg(unix)]
    set_permissions(source, target)?;

    // 替换文件
    fs::copy(source, target)
        .map_err(|e| anyhow::anyhow!("替换文件失败: {}", e))?;

    Ok(())
}

#[cfg(unix)]
#[allow(unused_imports)]
fn set_permissions(source: &Path, target: &Path) -> Result<()> {
    let metadata = fs::metadata(target)
        .map_err(|e| anyhow::anyhow!("获取元数据失败: {}", e))?;
    // 使用 trait 方法设置权限
    use std::os::unix::fs::PermissionsExt;
    let permissions = metadata.permissions();
    fs::set_permissions(source, permissions)
        .map_err(|e| anyhow::anyhow!("设置权限失败: {}", e))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_permissions(_source: &Path, _target: &Path) -> Result<()> {
    // Windows 平台不需要特殊权限设置
    Ok(())
}