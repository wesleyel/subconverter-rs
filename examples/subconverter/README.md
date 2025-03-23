# Subconverter Example

This example demonstrates how to use the subconverter functionality in the subconverter-rs crate to convert proxy configurations between different formats.

## Files

- `main.rs` - The main example code that demonstrates how to use the subconverter API
- `config.ini` - A sample configuration file similar to the original subconverter's pref.ini
- `clash_base.yml` - A base template for Clash output format

## Usage

Run the example with:

```
cargo run --example subconverter [CONFIG_PATH] [SUBSCRIPTION_URL] [TARGET_FORMAT]
```

Arguments:
- `CONFIG_PATH` - Path to the configuration INI file (defaults to examples/subconverter/config.ini)
- `SUBSCRIPTION_URL` - URL to the subscription to convert (defaults to a placeholder)
- `TARGET_FORMAT` - Target format to convert to (clash, surge, quanx, etc., defaults to clash)

Example:

```
cargo run --example subconverter examples/subconverter/config.ini https://example.com/subscription clash
```

## Features Demonstrated

1. Loading and parsing subconverter configuration files
2. Setting up the SubconverterConfig with the builder pattern
3. Converting proxy subscriptions between different formats
4. Handling ruleset content and proxy groups
5. Processing emoji patterns and other settings

The output will be written to `examples/subconverter/output.yml` (or other extension based on target format). 