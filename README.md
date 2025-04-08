# subconverter-rs

<div align="center">

> Transform. Optimize. Simplify. A blazingly fast proxy subscription converter rewritten in Rust.

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-beta-blue.svg)](https://github.com/lonelam/subconverter-rs)
[![GPL-3.0+ License](https://img.shields.io/badge/license-GPL--3.0%2B-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/subconverter.svg)](https://crates.io/crates/subconverter)
[![GitHub stars](https://img.shields.io/github/stars/lonelam/subconverter-rs?style=social)](https://github.com/lonelam/subconverter-rs/stargazers)

</div>

A more powerful utility to convert between proxy subscription formats, transformed from the C++ version subconverter! This Rust implementation offers improved performance and reliability while maintaining compatibility with the original.

**⚠️ BETA VERSION AVAILABLE ⚠️** - This project is now in beta. Core features are implemented but may still have some rough edges.

**Coming Soon** - wasm version for one-click deployment.

## 📋 Table of Contents

- [Features](#features)
- [Implementation Status](#implementation-status)
- [Protocol Support Matrix](#protocol-support-matrix)
- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Advanced Usage](#advanced-usage)
- [Configuration](#configuration)
- [Development](#development)
- [License](#license)

## ✨ Features

- High-performance subscription conversion with Rust's speed and safety
- Support for various proxy formats and protocols
- Flexible node filtering, renaming, and emoji addition
- Customizable templates and rule sets
- HTTP server with RESTful API endpoints
- Compatible with original subconverter configuration

## 📊 Protocol Support Matrix

The following table shows the support status of different proxy protocols in various rule types:

| Protocol \ Rule Type | Clash | SingBox | Surge(2,3,4) | V2Ray | Quantumult | Quantumult X | Loon | Surfboard | Mellow | SIP002/8 | Mixed | TG-like |
|--------------|:-----:|:-------:|:-----:|:-----:|:----------:|:------------:|:----:|:---------:|:------:|:--------:|:----------:|:-----:|
| VLESS        | ✅ | ✅ | ⚠️ | ✅ | ❌ | ⚠️ | ⚠️ | ❌ | ❌ | ❌ | ⬇️ | ⬇️ |
| Hysteria/2   | ✅ | ✅ | ⚠️ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ❌ | ⬇️ | ⬇️ |
| VMess        | ✅ | ✅ | ⚠️ | ✅ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ❌ | ✅ | ⬇️ |
| Trojan       | ✅ | ✅ | ⚠️ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ❌ | ✅ | ⬇️ |
| SS           | ✅ | ✅ | ⚠️ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ✅ | ✅ | ⬇️ |
| SSR          | ✅ | ✅ | ⚠️ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ❌ | ✅ | ⬇️ |
| HTTP/SOCKS   | ✅ | ✅ | ⚠️ | ❌ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | ❌ | ⬇️ | ⬇️ |
| WireGuard    | ✅ | ✅ | ⚠️ | ⬇️ | ❌ | ⚠️ | ⚠️ | ⚠️ | ❌ | ❌ | ⬇️ | ⬇️ |
| Snell        | ✅ | ❌ | ⚠️ | ❌ | ❌ | ⚠️ | ⚠️ | ⚠️ | ❌ | ❌ | ⬇️ | ⬇️ |
| SSD          | ⬇️ | ⬇️ | ⬇️ | ⬇️ | ⬇️ | ⬇️ | ⬇️ | ⬇️ | ⬇️ | ⬇️ | ❌ | ⬇️ |

**Legend:**
- ✅ Fully supported (both input and output)
- ⚠️ Partially supported (untested)
- ⬇️ Supported as input source only
- ⬆️ Supported as output target only
- ❌ Not supported

**Notes:**
1. Shadowrocket users can use the `ss`, `ssr`, `v2ray`, and `mixed` parameters
2. For HTTP/Socks links without naming (TG-like), you can append `&remarks=` for naming and `&group=` for group naming. These parameters need to be [URLEncoded](https://www.urlencoder.org/)
3. When the target type is `mixed`, all supported nodes will be output as a normal subscription (Base64 encoded)
4. When the target type is `auto`, the output target type will be automatically determined based on the request's `User-Agent`


## 📥 Installation

### From GitHub Releases
```bash
# Download the latest release
DOWNLOAD_URL=$(curl -s https://api.github.com/repos/lonelam/subconverter-rs/releases/latest | \
               jq -r '.assets[] | select(.name == "$subconverter_zip_name").browser_download_url')
curl -L -O "$DOWNLOAD_URL"
tar -zxf $subconverter_zip_name

# Navigate to the extracted directory
cd subconverter
```

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

### Docker (Coming Soon)
```bash
docker pull lonelam/subconverter-rs
docker run -d -p 25500:25500 lonelam/subconverter-rs
```

## 🔰 Basic Usage

### API Endpoint

```
http://127.0.0.1:25500/sub?target=%TARGET%&url=%URL%&config=%CONFIG%
```

### Parameters

| Parameter | Required | Example | Description | Status |
| --------- | :------: | ------- | ----------- | :----: |
| target | Yes | surge&ver=4 | Target configuration type | ✅ |
| url | Yes | https%3A%2F%2Fwww.xxx.com | Subscription link (URLEncoded) | ✅ |
| config | No | https%3A%2F%2Fwww.xxx.com | External configuration (URLEncoded) | ✅ |

### Simple Conversion Examples

<details>
<summary><b>Converting a Single Subscription</b></summary>

```
Original subscription: https://example.com/subscribe/ABCDE?surge=ss

URLEncoded: https%3A%2F%2Fexample.com%2Fsubscribe%2FABCDE%3Fsurge%3Dss

API call: http://127.0.0.1:25500/sub?target=clash&url=https%3A%2F%2Fexample.com%2Fsubscribe%2FABCDE%3Fsurge%3Dss
```
</details>

<details>
<summary><b>Combining Multiple Subscriptions</b></summary>

```
Original subscriptions:
1. https://example1.com/subscribe/ABCDE?clash=vmess
2. https://example2.com/subscribe/ABCDE?clash=vmess

Combined with pipe: https://example1.com/subscribe/ABCDE?clash=vmess|https://example2.com/subscribe/ABCDE?clash=vmess

URLEncoded: https%3A%2F%2Fexample1.com%2Fsubscribe%2FABCDE%3Fclash%3Dvmess%7Chttps%3A%2F%2Fexample2.com%2Fsubscribe%2FABCDE%3Fclash%3Dvmess

API call: http://127.0.0.1:25500/sub?target=clash&url=https%3A%2F%2Fexample1.com%2Fsubscribe%2FABCDE%3Fclash%3Dvmess%7Chttps%3A%2F%2Fexample2.com%2Fsubscribe%2FABCDE%3Fclash%3Dvmess
```
</details>

<details>
<summary><b>Converting a Single Node</b></summary>

```
Original node: ss://YWVzLTEyOC1nY206dGVzdA==@192.168.100.1:8888#Example1

URLEncoded: ss%3A%2F%2FYWVzLTEyOC1nY206dGVzdA%3D%3D%40192%2E168%2E100%2E1%3A8888%23Example1

API call: http://127.0.0.1:25500/sub?target=clash&url=ss%3A%2F%2FYWVzLTEyOC1nY206dGVzdA%3D%3D%40192%2E168%2E100%2E1%3A8888%23Example1
```
</details>

### Quick Surge to Clash Conversion

For quick conversion from Surge to Clash without additional configuration:
```
http://127.0.0.1:25500/surge2clash?link=SurgeSubscriptionLink
```
Note: The Surge subscription link does NOT need to be URLEncoded.

## 🔧 Advanced Usage

### Advanced API Parameters

<details>
<summary><b>Click to show all available parameters</b></summary>

| Parameter | Required | Example | Description | Status |
| --------- | :------: | ------- | ----------- | :----: |
| emoji | No | true/false | Enable emoji in node names | ✅ |
| add_emoji | No | true/false | Add emoji before node names | ✅ |
| remove_emoji | No | true/false | Remove existing emoji from node names | ✅ |
| append_type | No | true/false | Add proxy type ([SS], [SSR], etc.) to node names | ✅ |
| tfo | No | true/false | Enable TCP Fast Open | ✅ |
| udp | No | true/false | Enable UDP support | ✅ |
| scv | No | true/false | Skip certificate verification for TLS nodes | ✅ |
| tls13 | No | true/false | Enable TLS 1.3 for nodes | ✅ |
| sort | No | true/false | Sort nodes by name | ✅ |
| include | No | (regex) | Only include nodes matching the pattern | ✅ |
| exclude | No | (regex) | Exclude nodes matching the pattern | ✅ |
| filename | No | MyConfig | Set the file name for the generated config | ✅ |
| list | No | true/false | Output as node list or provider format | ✅ |
| insert | No | true/false | Insert nodes from insert_url in config | ✅ |
| prepend | No | true/false | Insert nodes at the beginning | ✅ |
</details>

## ⚙️ Configuration

subconverter-rs supports multiple configuration file formats. It will load configuration in the following priority order: `pref.toml`, `pref.yml`, `pref.ini`.

### Key Configuration Sections

<details>
<summary><b>[common]</b> - Global node filtering and base configuration settings</summary>

- api_mode: API mode settings
- api_access_token: Token for accessing private interfaces
- default_url: Default subscription links to load
- enable_insert: Whether to add insertion nodes
- insert_url: URL for insertion nodes
- exclude_remarks: Exclude nodes matching the pattern
- include_remarks: Only include nodes matching the pattern
- default_external_config: Default external configuration file
- clash_rule_base: Clash configuration template
- surge_rule_base: Surge configuration template
</details>

<details>
<summary><b>[userinfo]</b> - Rules for extracting user information from node names</summary>

- stream_rule: Rules for extracting traffic information
- time_rule: Rules for extracting time information
</details>

<details>
<summary><b>[node_pref]</b> - Node preferences (UDP, TFO, renaming, sorting)</summary>

- udp_flag: Open UDP mode for nodes
- tcp_fast_open_flag: Open TFO mode for nodes
- skip_cert_verify_flag: Turn off certificate checks for TLS nodes
- tls13_flag: Add TLS 1.3 parameters for nodes
- sort_flag: Sort nodes by name
- append_sub_userinfo: Whether to append traffic information
- clash_use_new_field_name: Whether to use Clash's new field names
- clash_proxies_style: Clash configuration file format style
- rename_node: Node renaming rules
</details>

<details>
<summary><b>Additional Sections</b> - [managed_config], [emojis], [ruleset], [proxy_group], [template]</summary>

There are several other configuration sections for managed config settings, emoji handling, custom rule sets, proxy groups, and template system settings. See the documentation for detailed information.
</details>

### External Configuration

You can host configuration files on GitHub Gist or other accessible network locations. URL-encode the configuration URL and add it to the `&config=` parameter.

### Local Generation

For generating configurations locally, create a `generate.ini` file:

```ini
[test]
path=output.conf
target=surge
ver=4
url=ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpwYXNzd29yZA@www.example.com:1080#Example
```

Then run: `subconverter -g`

## 👩‍💻 Development

Contributions are welcome! Please feel free to submit a Pull Request.

### How to Contribute

1. **Pick an issue**: Check our [issue tracker](https://github.com/lonelam/subconverter-rs/issues) for tasks labeled `good first issue` or `help wanted`
2. **Implement new proxy types**: Help expand support for additional proxy protocols
3. **Improve parsing**: Enhance the robustness of the various format parsers
4. **Add tests**: Increase test coverage to ensure stability
5. **Documentation**: Improve docs or add examples to help others use the project
6. **Performance optimizations**: Help make the converter even faster

## 📄 License

This project is licensed under the GPL-3.0+ License - see the [LICENSE](LICENSE) file for details.
