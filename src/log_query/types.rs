//! 日志查询数据类型模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 日志查询请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryRequest {
    /// 日志 ID
    pub logid: String,
    /// PSM 服务列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub psm_list: Vec<String>,
    /// 扫描时间范围（分钟）
    #[serde(rename = "scan_span_in_min")]
    pub scan_span_in_min: i32,
    /// 虚拟区域
    pub vregion: String,
}

impl LogQueryRequest {
    /// 创建新的日志查询请求
    pub fn new(
        logid: String,
        psm_list: Vec<String>,
        scan_span_in_min: i32,
        vregion: String,
    ) -> Self {
        Self {
            logid,
            psm_list,
            scan_span_in_min,
            vregion,
        }
    }
}

/// 日志查询响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogQueryResponse {
    /// 数据内容
    pub data: Option<LogData>,
    /// 响应元数据
    pub meta: Option<serde_json::Value>,
    /// 标签信息
    #[serde(rename = "tag_infos")]
    pub tag_infos: Option<Vec<serde_json::Value>>,
    /// 响应时间戳
    pub timestamp: String,
    /// 区域信息
    pub region: String,
    /// 区域显示名称
    #[serde(rename = "region_display_name")]
    pub region_display_name: String,
}

/// 日志数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogData {
    /// 日志项目列表
    pub items: Vec<LogItem>,
    /// 元数据
    pub meta: Option<LogMeta>,
    /// 标签信息
    #[serde(rename = "tag_infos")]
    pub tag_infos: Option<Vec<serde_json::Value>>,
}

/// 日志项目
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogItem {
    /// 项目 ID
    pub id: String,
    /// 分组信息
    pub group: LogGroup,
    /// 值列表
    pub value: Vec<LogValue>,
}

/// 日志分组信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogGroup {
    /// PSM 服务名
    pub psm: Option<String>,
    /// Pod 名称
    #[serde(rename = "pod_name")]
    pub pod_name: Option<String>,
    /// IP 地址
    #[serde(rename = "ipv4")]
    pub ipv4: Option<String>,
    /// 环境
    pub env: Option<String>,
    /// 虚拟区域
    pub vregion: Option<String>,
    /// IDC
    pub idc: Option<String>,
}

/// 日志值
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogValue {
    /// 值 ID
    pub id: String,
    /// 键值对列表
    #[serde(rename = "kv_list")]
    pub kv_list: Vec<LogKv>,
    /// 日志级别
    pub level: Option<String>,
}

/// 日志键值对
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogKv {
    /// 键名
    pub key: String,
    /// 值
    pub value: String,
    /// 类型
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    /// 是否高亮显示
    pub highlight: Option<bool>,
}

/// 日志元数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogMeta {
    /// 扫描时间范围
    #[serde(rename = "scan_time_range")]
    pub scan_time_range: Option<Vec<TimeRange>>,
    /// 日志级别列表
    #[serde(rename = "level_list")]
    pub level_list: Option<Vec<String>>,
    /// 其他元数据字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 时间范围
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeRange {
    /// 开始时间戳
    pub start: Option<i64>,
    /// 结束时间戳
    pub end: Option<i64>,
}

/// 提取的日志消息
#[derive(Debug, Clone, Serialize)]
pub struct ExtractedLogMessage {
    /// 项目 ID
    pub id: String,
    /// 分组信息
    pub group: LogGroup,
    /// 提取的值列表（过滤后的）
    pub values: Vec<ExtractedValue>,
    /// 日志代码位置
    #[serde(rename = "location")]
    pub location: Option<String>,
    /// 日志级别
    pub level: Option<String>,
}

/// 提取的值
#[derive(Debug, Clone, Serialize)]
pub struct ExtractedValue {
    /// 键名
    pub key: String,
    /// 过滤后的值
    pub value: String,
    /// 原始值
    pub original_value: String,
    /// 类型
    pub type_field: Option<String>,
    /// 是否高亮显示
    pub highlight: bool,
}

/// 详细的日志查询结果
#[derive(Debug, Clone, Serialize)]
pub struct DetailedLogResult {
    /// 日志 ID
    pub logid: String,
    /// 提取的日志消息
    pub messages: Vec<ExtractedLogMessage>,
    /// 元数据
    pub meta: Option<LogMeta>,
    /// 标签信息
    #[serde(rename = "tag_infos")]
    pub tag_infos: Option<Vec<serde_json::Value>>,
    /// 消息总数
    #[serde(rename = "total_items")]
    pub total_items: usize,
    /// 扫描时间范围
    #[serde(rename = "scan_time_range")]
    pub scan_time_range: Option<Vec<TimeRange>>,
    /// 日志级别列表
    #[serde(rename = "level_list")]
    pub level_list: Option<Vec<String>>,
    /// 查询时间戳
    pub timestamp: String,
    /// 区域信息
    pub region: String,
    /// 区域显示名称
    #[serde(rename = "region_display_name")]
    pub region_display_name: String,
}
