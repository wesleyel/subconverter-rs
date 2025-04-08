# WebAssembly Support for Subconverter-rs

This document explains how to build and use the WebAssembly version of subconverter-rs.

## Prerequisites

- [Rust](https://www.rust-lang.org/) with wasm32-unknown-unknown target
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (for testing with React)

## Setup rust-analyzer for WASM Development

1. This repository includes configurations for VS Code in `.vscode/settings.json` that sets up rust-analyzer to work with the wasm32 target.

2. The `.cargo/config.toml` file contains necessary configuration for WASM builds.

## Building the WASM Package

Run the provided build script:

```bash
./build-wasm.sh
```

This will:
1. Check if wasm-pack is installed and install it if necessary
2. Build the WASM package targeting web browsers 
3. Optimize the WASM binary if wasm-opt is available

The output will be in the `pkg` directory.

## Available WASM Functions

Currently, the following functions are exported to WASM:

- `initialize_settings_from_content(content: string): Result<void, Error>` - Initialize settings from YAML, TOML, or INI content
- `sub_process_wasm(query_json: string): Promise<string>` - Process subscription requests and return results as JSON

### Using sub_process_wasm

The `sub_process_wasm` function takes a single parameter:

- `query_json`: A JSON string containing the `SubconverterQuery` parameters

Example usage in JavaScript:

```javascript
import init, { initialize_settings_from_content, sub_process_wasm } from 'subconverter';

async function setupAndConvert() {
  // Initialize WASM module
  await init();
  
  // Initialize settings first
  const settingsContent = `common:
  api_mode: false
  max_pending_connections: 1024
  max_concurrent_threads: 4`;
  
  try {
    // Load settings
    initialize_settings_from_content(settingsContent);
    
    // Create query parameters
    const query = {
      target: "clash",
      url: "https://example.com/subscription",
      udp: true,
      emoji: true,
      append_type: true
    };
    
    // Process subscription
    const response = await sub_process_wasm(JSON.stringify(query));
    const result = JSON.parse(response);
    console.log("Conversion result:", result);
    
    // Access the conversion output
    const convertedConfig = result.content;
    const contentType = result.content_type;
  } catch (error) {
    console.error("Error:", error);
  }
}
```

#### SubconverterQuery Parameters

Here are the main parameters you can include in the query JSON:

| Parameter | Type | Description |
|-----------|------|-------------|
| `target` | string | Target format (e.g., "clash", "surge", "v2ray") |
| `url` | string | Subscription URL(s), multiple URLs can be separated by pipe (`\|`) |
| `config` | string | External config URL (optional) |
| `include` | string | Include remarks regex |
| `exclude` | string | Exclude remarks regex |
| `emoji` | boolean | Whether to add emoji to node names |
| `udp` | boolean | Enable UDP support |
| `tfo` | boolean | Enable TCP Fast Open |
| `scv` | boolean | Skip TLS certificate verification |
| `sort` | boolean | Enable sorting of nodes |
| `append_type` | boolean | Append proxy type to remarks |

## Development Notes

- The WASM version does not include the web-api feature, as web servers cannot run in the browser environment
- localStorage is used as a simple file system simulation for WASM builds
- WASM bindings are conditionally compiled with `#[cfg(target_arch = "wasm32")]`

## Testing with React + Rspack

A simple React testing project is provided in the `wasm-test` directory. This project uses Rspack for bundling and demonstrates how to use the WASM module.

## Troubleshooting

### Common Issues

1. **"Module not found" error**: Make sure you've built the WASM package with `./build-wasm.sh` before trying to use it.

2. **Permissions issues with fetch API**: When testing locally, use a web server instead of opening the HTML file directly.

3. **CORS issues**: If you're experiencing CORS issues when making fetch requests from your WASM code, make sure your server is configured to allow cross-origin requests. 