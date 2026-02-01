//! 输出格式配置模块

/// 输出配置
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// 是否显示元数据
    pub show_metadata: bool,
    /// 是否显示扫描时间范围
    pub show_scan_time_range: bool,
    /// 是否显示标签信息
    pub show_tag_infos: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            show_metadata: true,
            show_scan_time_range: true,
            show_tag_infos: false,
        }
    }
}

impl OutputConfig {
    /// 创建新的输出配置
    pub fn new() -> Self {
        Self::default()
    }
}
