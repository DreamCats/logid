# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

Rust-based CLI tool for querying internal log services by logid. Supports multi-region (US, I18N, CN) queries with JWT authentication, PSM filtering, and JSON output.

## Build Commands

```bash
# Build
cargo build --release

# Test
cargo test

# Run
./target/release/logid query <logid> --region us
./target/release/logid query <logid> --region us --psm psm1 --psm psm2

# Debug
ENABLE_LOGGING=true cargo run -- query <logid> --region us
```

## Configuration

`.env` file locations (by priority):
1. `<exe_dir>/.env`
2. `~/.config/logid/.env`

```bash
mkdir -p ~/.config/logid
cat > ~/.config/logid/.env << EOF
CAS_SESSION_US=your_session
CAS_SESSION_I18n=your_session
EOF
```

## Architecture

```
src/
├── lib.rs              # Library entry, conditional_info! macro
├── main.rs             # CLI entry
├── error.rs            # Error types
├── auth/               # JWT authentication
│   ├── manager.rs      # AuthManager
│   └── multi_region.rs # MultiRegionAuthManager
├── config/             # Configuration
│   ├── region.rs       # Region enum, RegionConfig
│   ├── env.rs          # EnvManager, .env loading
│   ├── filter.rs       # Message filters
│   └── jwt.rs          # JwtInfo
├── log_query/          # Log query
│   ├── types.rs        # Request/Response types
│   ├── client.rs       # LogQueryClient
│   └── multi_region.rs # MultiRegionLogQuery
├── output/             # Output formatting
│   ├── format.rs       # OutputConfig
│   └── formatter.rs    # JSON formatter
└── commands/           # Subcommands
    └── update.rs       # Self-update
```

## Key Patterns

**Authentication**: CAS_SESSION cookie → JWT token (1h validity, cached)

**Data Flow**: HTTP request → JSON parse → extract `_msg` → filter → JSON output

**Conditional Logging**: `conditional_info!` macro, controlled by `ENABLE_LOGGING` env var

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CAS_SESSION_US` | US region auth |
| `CAS_SESSION_I18n` | I18N region auth |
| `CAS_SESSION_CN` | CN region auth |
| `CAS_SESSION` | Fallback auth |
| `ENABLE_LOGGING` | Debug logging (true/false) |

## Common Tasks

- **Add region**: Update `REGION_AUTH_URLS` in `auth/manager.rs`, `get_region_config()` in `config/region.rs`
- **Add filter**: Update `get_default_filters()` in `config/filter.rs`
- **Debug**: Set `ENABLE_LOGGING=true`

## Dependencies

- `clap` 4.4: CLI parsing
- `reqwest` 0.11: HTTP client (rustls)
- `tokio` 1.0: Async runtime
- `serde` 1.0: JSON
- `regex` 1.10: Filtering
- `tracing` 0.1: Logging
