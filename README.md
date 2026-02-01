# logid

基于 Rust 开发的命令行工具，用于通过 logid 查询内部日志服务。

## 功能特性

- 多区域支持：美区 (us)、国际化区域 (i18n)、中国区 (cn)
- JWT 认证：自动获取和刷新认证令牌
- 消息过滤：自动移除冗余信息
- JSON 输出：结构化输出便于解析

## 安装

```bash
cargo build --release
```

编译后的二进制文件位于 `target/release/logid`。

## 配置

### 环境变量

工具从以下位置加载 `.env` 文件（按优先级）：

1. 可执行文件同级目录：`<exe_dir>/.env`
2. 用户配置目录：`~/.logid/.env`

创建配置：

```bash
mkdir -p ~/.logid
cat > ~/.logid/.env << EOF
CAS_SESSION_US=your_us_session_cookie
CAS_SESSION_I18n=your_i18n_session_cookie
EOF
```

### 获取 CAS_SESSION

1. 登录对应的云平台
2. 浏览器开发者工具 → Cookie
3. 找到 `CAS_SESSION` 的值

## 使用

```bash
# 查询美区日志
logid query <logid> --region us

# 查询国际化区域
logid query <logid> --region i18n

# 过滤特定 PSM
logid query <logid> --region us --psm service.psm

# 多个 PSM
logid query <logid> --region us --psm psm1 --psm psm2
```

### 参数

| 参数 | 必需 | 说明 |
|------|------|------|
| `logid` | 是 | 日志 ID |
| `--region`, `-r` | 是 | 区域 (cn/i18n/us) |
| `--psm`, `-p` | 否 | PSM 过滤（可多次指定）|

## 输出示例

```json
{
  "logid": "20240101-abc123",
  "region": "us",
  "region_display_name": "美区",
  "total_items": 5,
  "messages": [
    {
      "id": "msg_1",
      "group": {
        "psm": "payment.service",
        "pod_name": "payment-pod-123",
        "ipv4": "192.168.1.100"
      },
      "values": [
        {
          "key": "_msg",
          "value": "Payment processed successfully"
        }
      ]
    }
  ],
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## 区域

| 区域 | 状态 |
|------|------|
| `us` | 可用 |
| `i18n` | 可用 |
| `cn` | 待配置 |

## 消息过滤

自动过滤以下冗余信息：
- `_compliance_nlp_log`
- `_compliance_whitelist_log`
- `_compliance_source=footprint`
- `user_extra` 字段
- `LogID`、`Addr`、`Client` 字段

## 代码结构

```
src/
├── lib.rs              # 库入口
├── main.rs             # CLI 入口
├── error.rs            # 错误定义
├── auth/               # 认证模块
│   ├── manager.rs      # JWT 认证管理
│   └── multi_region.rs # 多区域认证
├── config/             # 配置模块
│   ├── region.rs       # 区域配置
│   ├── env.rs          # 环境变量
│   └── filter.rs       # 消息过滤
├── log_query/          # 日志查询模块
│   ├── types.rs        # 数据类型
│   ├── client.rs       # 查询客户端
│   └── multi_region.rs # 多区域查询
├── output/             # 输出模块
│   ├── format.rs       # 输出配置
│   └── formatter.rs    # JSON 格式化
└── commands/           # 子命令
    └── update.rs       # 自更新
```

## 开发

```bash
# 测试
cargo test

# 调试运行
ENABLE_LOGGING=true cargo run -- query <logid> --region us
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `CAS_SESSION_US` | 美区认证 |
| `CAS_SESSION_I18n` | 国际化区域认证 |
| `CAS_SESSION_CN` | 中国区认证 |
| `CAS_SESSION` | 通用回退 |
| `ENABLE_LOGGING` | 启用调试日志 |

## 更新日志

### v0.1.0
- 初始版本
- 多区域日志查询
- PSM 过滤
- JSON 输出
- 消息过滤
