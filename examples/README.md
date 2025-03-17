# Subconverter-rs Examples

This directory contains example applications that demonstrate how to use the subconverter-rs library.

## Running Examples

You can run any example using cargo:

```bash
cargo run --example example_name
```

## Available Examples

### 1. Add Nodes Example (`add_nodes_example.rs`)

Demonstrates the basic functionality of the `add_nodes` function with hardcoded examples.

```bash
cargo run --example add_nodes_example
```

### 2. URL Parser Example (`url_parser_example.rs`)

Demonstrates how to parse proxy nodes from a URL or subscription link passed as a command-line argument.

```bash
# Basic usage with a VMess URL
cargo run --example url_parser_example "vmess://eyJ2IjogIjIiLCAicHMiOiAiSEstQSIsICJhZGQiOiAiZXhhbXBsZS5jb20iLCAicG9ydCI6IDQ0MywgImlkIjogIjEyMzQ1Njc4LTEyMzQtMTIzNC0xMjM0LTEyMzQ1Njc4OTAxMiIsICJhaWQiOiAwLCAibmV0IjogIndzIiwgInR5cGUiOiAibm9uZSIsICJob3N0IjogImV4YW1wbGUuY29tIiwgInBhdGgiOiAiL3BhdGgiLCAidGxzIjogInRscyJ9"

# Parse a subscription link
cargo run --example url_parser_example "https://example.com/your_subscription_link"

# With filtering (include pattern)
cargo run --example url_parser_example "https://example.com/your_subscription_link" "HK|香港" 

# With filtering (include and exclude patterns)
cargo run --example url_parser_example "https://example.com/your_subscription_link" "HK|香港" "Test|测试"
```

### 3. Proxy Model Example (`proxy_model_example.rs`)

Demonstrates how to work with the `Proxy` model.

```bash
cargo run --example proxy_model_example
```

### 4. Node Manipulation Example (`node_manip_example.rs`)

Demonstrates how to manipulate proxy nodes after they've been parsed.

```bash
cargo run --example node_manip_example
``` 