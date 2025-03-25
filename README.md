# subconverter-rs
A more powerful utility to convert between proxy subscription format, the original codes are transformed from the cpp version subconverter!

> Transform. Optimize. Simplify. A blazingly fast proxy subscription converter rewritten in Rust.

**⚠️ BETA VERSION AVAILABLE ⚠️** - This project is now in beta. Core features are implemented but may still have some rough edges.

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-beta-blue.svg)](https://github.com/lonelam/subconverter-rs)
[![GPL-3.0+ License](https://img.shields.io/badge/license-GPL--3.0%2B-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/subconverter.svg)](https://crates.io/crates/subconverter)

subconverter-rs takes the power of the original [subconverter](https://github.com/tindy2013/subconverter) project and reimplements it in Rust, bringing memory safety, concurrency without data races, and significantly improved performance.

## Why?
The original subconverter is not easy to use and can be really hard to contribute to, with more than half of PRs being abandoned.

However, subconverter is almost the only tool that provides compatibility across a bunch of proxy tools.

## Roadmap

| Feature | Status | Priority | Description |
|---------|:------:|:--------:|-------------|
| Core Conversion Engine | ✅ | High | Basic proxy parsing and conversion between formats |
| Node Manipulation | ✅ | High | Filtering, renaming, and preprocessing nodes |
| VMess Protocol Support | ✅ | High | Complete support for VMess protocol |
| Crates.io Publication | ✅ | Medium | Published as a Rust crate for easy installation |
| HTTP Server | 🔄 | High | Server for subscription conversion |
| Additional API Endpoints | 🔄 | Medium | Implement /surge2clash, /getprofile, etc. |
| Template System | 🔄 | Medium | Support for customizable templates |
| Web Interface | 🔄 | Medium | Online conversion interface |
| RESTful API | 🔄 | Medium | Comprehensive API for integration |
| Auto-upload to Gist | 🔄 | Low | Automatic upload of generated configurations |
| Plugin System | 🔄 | Low | Easy extension of functionality |
| Feature Parity | 🔄 | Ongoing | Complete feature parity with original subconverter |
| Performance Benchmarks | 🔄 | Low | Comparison with original implementation |
| Docker Container | 🔄 | Medium | Containerization for easy deployment |
| CI/CD Pipelines | 🔄 | Medium | Automated testing and deployment |

Legend:
- ✅ Completed
- 🔄 In Progress/Planned

## Implementation Status

subconverter-rs has implemented the core functionality of the original C++ version, including:

### Supported Features
- Converting between various proxy subscription formats
- Filtering nodes based on remarks and rules
- Adding emojis to node remarks
- Renaming nodes based on custom rules
- Preprocessing nodes with custom rules
- Parsing local configuration files
- Command line interface

### Supported Proxy Types
- VMess
- Shadowsocks
- ShadowsocksR
- Trojan
- HTTP/HTTPS
- SOCKS
- Hysteria/Hysteria2
- WireGuard
- Snell

### Supported Output Formats
- Clash
- Surge
- Quantumult
- Quantumult X
- Loon
- ShadowsocksD (SSD)
- Mellow
- SingBox

## Installation

### From Crates.io
```bash
cargo install subconverter
```

### From Source
```bash
git clone https://github.com/lonelam/subconverter-rs.git
cd subconverter-rs
cargo build --release
```

The binary will be available at `target/release/subconverter-rs`.

## Usage

### Command Line
```bash
subconverter [options]
```

## Configuration
subconverter-rs uses similar configuration to the original subconverter.

## Development
Contributions are welcome! Please feel free to submit a Pull Request.

### How to Contribute

1. **Pick an issue**: Check our [issue tracker](https://github.com/lonelam/subconverter-rs/issues) for tasks labeled `good first issue` or `help wanted`
2. **Implement new proxy types**: Help expand support for additional proxy protocols
3. **Improve parsing**: Enhance the robustness of the various format parsers
4. **Add tests**: Increase test coverage to ensure stability
5. **Documentation**: Improve docs or add examples to help others use the project
6. **Performance optimizations**: Help make the converter even faster

For questions or discussions, you can:
- Open an issue on GitHub
  
## License
This project is licensed under the GPL-3.0+ License - see the LICENSE file for details.
