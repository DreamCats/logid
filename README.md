# logid

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A fast, reliable command-line tool for querying distributed log services by trace ID.

## Features

- **Multi-Region Support** - Query logs across US, International, and CN regions
- **Smart Authentication** - Automatic JWT token management with caching
- **Message Filtering** - Built-in noise reduction for cleaner output
- **Structured Output** - JSON format for easy parsing and integration
- **Self-Update** - Built-in update mechanism for easy upgrades

## Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/DreamCats/logid/main/install.sh | bash
```

Or with custom install directory:

```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/DreamCats/logid/main/install.sh | bash
```

### From Source

```bash
git clone https://github.com/DreamCats/logid.git
cd logid
cargo build --release
```

Binary will be available at `target/release/logid`.

### Pre-built Binaries

Download from [Releases](https://github.com/DreamCats/logid/releases).

## Quick Start

```bash
# Query logs in US region
logid query <trace-id> --region us

# Query with PSM filter
logid query <trace-id> --region us --psm my.service

# Multiple PSM filters
logid query <trace-id> --region i18n --psm service.a --psm service.b
```

## Configuration

Create a configuration file at `~/.config/logid/.env`:

```bash
mkdir -p ~/.config/logid
cat > ~/.config/logid/.env << 'EOF'
CAS_SESSION_US=your_us_session
CAS_SESSION_I18n=your_i18n_session
CAS_SESSION_CN=your_cn_session
EOF
```

Configuration is loaded from (in order of priority):
1. `<executable-directory>/.env`
2. `~/.config/logid/.env`

## Usage

```
logid query <LOGID> --region <REGION> [OPTIONS]

Arguments:
  <LOGID>  Trace ID to query

Options:
  -r, --region <REGION>  Target region (us/i18n/cn)
  -p, --psm <PSM>        Filter by PSM (can be specified multiple times)
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

```bash
# Basic query
logid query "abc-123-def" --region us

# Filter specific service
logid query "abc-123-def" --region us --psm payment.service

# Query international region with multiple filters
logid query "abc-123-def" --region i18n \
  --psm user.service \
  --psm auth.service
```

## Output

```json
{
  "logid": "abc-123-def",
  "region": "us",
  "region_display_name": "US Region",
  "total_items": 3,
  "messages": [
    {
      "id": "msg_1",
      "group": {
        "psm": "payment.service",
        "pod_name": "payment-pod-abc",
        "ipv4": "10.0.0.1",
        "env": "production"
      },
      "values": [
        {
          "key": "_msg",
          "value": "Payment processed successfully"
        }
      ],
      "level": "INFO",
      "location": "src/handler.rs:42"
    }
  ],
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CAS_SESSION_US` | Authentication for US region |
| `CAS_SESSION_I18n` | Authentication for International region |
| `CAS_SESSION_CN` | Authentication for CN region |
| `CAS_SESSION` | Fallback authentication |
| `ENABLE_LOGGING` | Enable debug output (`true`/`false`) |

## Supported Regions

| Region | Status |
|--------|--------|
| `us` | Available |
| `i18n` | Available |
| `cn` | Coming soon |

## Self-Update

```bash
# Check for updates
logid update --check

# Update to latest version
logid update

# Force update
logid update --force
```

## Development

```bash
# Run tests
cargo test

# Build debug version
cargo build

# Run with debug logging
ENABLE_LOGGING=true cargo run -- query <logid> --region us
```

## Project Structure

```
src/
├── lib.rs              # Library entry point
├── main.rs             # CLI entry point
├── error.rs            # Error definitions
├── auth/               # Authentication module
├── config/             # Configuration management
├── log_query/          # Log query client
├── output/             # Output formatting
└── commands/           # CLI subcommands
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with these excellent crates:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [serde](https://github.com/serde-rs/serde) - Serialization framework
