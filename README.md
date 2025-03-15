# subconverter-rs
A more powerful utility to convert between proxy subscription format, the original codes are transformed from the cpp version subconverter by Cursor!

> Transform. Optimize. Simplify. A blazingly fast proxy subscription converter rewritten in Rust.

**⚠️ WORK IN PROGRESS ⚠️** - This project is currently under active development. Features may be incomplete or subject to change.

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-active%20development-brightgreen.svg)](https://github.com/lonelam/subconverter-rs)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

subconverter-rs takes the power of the original [subconverter](https://github.com/tindy2013/subconverter) project and reimplements it in Rust, bringing memory safety, concurrency without data races, and significantly improved performance.

## Why Rust?

- **Performance**: Experience comparable or better performance than C++ with Rust's zero-cost abstractions
- **Memory Safety**: Eliminate segmentation faults and buffer overflows without sacrificing performance
- **Concurrency**: Safe concurrent processing for handling multiple subscriptions simultaneously
- **Modern Tooling**: Benefit from Cargo's dependency management, testing framework, and documentation generation
- **Cross-Platform**: Easily compile for various platforms with minimal configuration
- **Maintainability**: More readable, modular code that's easier to extend and contribute to


## Why ?
The subconverter is not easy to use and can be really hard to contribute, more than half of PRs are aborted.

However, the subconverter is almost the only one tool that could provide compatibility about a bunch of proxy tools.

## Supported Features
- Converting between various proxy subscription formats
- Filtering nodes based on remarks and rules
- Adding emojis to node remarks
- Renaming nodes based on custom rules
- Preprocessing nodes with custom rules
- Parsing local configuration files
- Command line interface

## Supported Proxy Types
- VMess
- Shadowsocks
- ShadowsocksR
- Trojan
- HTTP/HTTPS
- SOCKS
- Hysteria/Hysteria2
- WireGuard
- Snell

## Supported Output Formats
- Clash
- Surge
- Quantumult
- Quantumult X
- Loon
- ShadowsocksD (SSD)
- Mellow
- SingBox

## Installation

### From Source
```bash
git clone https://github.com/yourusername/subconverter-rs.git
cd subconverter-rs
cargo build --release
```

The binary will be available at `target/release/subconverter-rs`.

### From Cargo
```bash
cargo install subconverter-rs
```

## Usage

### Command Line
```bash
subconverter-rs [options]
```

### Library
You can use subconverter-rs as a library in your Rust projects:

```rust
use subconverter_rs::generator::config::proxy::Proxy;
use subconverter_rs::generator::{
    add_nodes, filter_nodes, node_rename, preprocess_nodes, ExtraSettings, ParseSettings,
    RegexMatchConfig,
};

fn main() {
    // Create sample nodes
    let mut nodes = Vec::new();
    
    // Parse configuration files or links
    let parse_settings = ParseSettings::default();
    add_nodes("config.txt", &mut nodes, 1, &parse_settings);
    
    // Preprocess nodes with custom rules
    let mut ext = ExtraSettings::default();
    preprocess_nodes(&mut nodes, &ext);
    
    // Convert to different formats
    let clash_config = subconverter_rs::generator::config::formats::clash::proxy_to_clash(
        &mut nodes, "", &[], &[], false, &ext
    );
}
```

## Examples
Check out the `examples` directory for more usage examples:

```bash
cargo run --example node_manip_example
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
  
### Roadmap

- [x] Basic proxy parsing and conversion
- [x] Node filtering and manipulation
- [ ] Complete VMess protocol support
- [ ] Web interface for online conversion
- [ ] HTTP server for subscription conversion
- [ ] RESTful API
- [ ] Plugin system for easy extension
- [ ] Complete feature parity with original subconverter
- [ ] Performance benchmarks vs. original implementation
- [ ] Docker container and CI/CD pipelines

## License
This project is licensed under the MIT License - see the LICENSE file for details.
