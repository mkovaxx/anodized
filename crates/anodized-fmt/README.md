# Anodized-fmt

A formatter for `#[spec]` annotations in Rust code using [Anodized](https://github.com/mkovaxx/anodized).

## Overview

`anodized-fmt` reformats `#[spec]` attributes while leaving all other code unchanged. It is built on top of `anodized-core` which provides anodized specification.

## Installation

```bash
cargo install anodized-fmt
```

Or build from source:

```bash
cd crates/anodized-fmt
cargo build --release
```

## Usage

### Format files

```bash
# Format a single file
anodized-fmt src/main.rs

# Format all Rust files in a directory
anodized-fmt src/

# Format current directory
anodized-fmt
```

### Check mode (CI)

```bash
# Check if files are formatted without modifying them
anodized-fmt --check
```

### Configuration Options

```bash
# Use a custom config file
anodized-fmt --config anodized-fmt.toml

# Print default configuration
anodized-fmt --print-config default

# Print current configuration (with file overrides)
anodized-fmt --print-config current
```

### Options

- `--check` - Check if files are formatted without modifying
- `--config <FILE>` - Path to configuration file
- `--print-config <OPTION>` - Print configuration (default, current)
- `--verbose` / `-v` - Verbose output
- `--quiet` / `-q` - Suppress non-error output

## Configuration

Create an `anodized-fmt.toml` file in your project root:

```toml
# Maximum line width for spec attributes
max_width = 100

# Number of spaces for indentation
indent = 4

# Trailing comma: "always", "never", "auto"
trailing_comma = "always"
```

## Example

**Before:**

```rust
#[spec(requires: x > 0, ensures: *output > 0)]
fn add_one(x: i32) -> i32 {
    x + 1
}
```

**After:**

```rust
#[spec(
    requires: x > 0,
    ensures: *output > 0,
)]
fn add_one(x: i32) -> i32 {
    x + 1
}
```

## Library Usage

```rust
use anodized_fmt::{format_file, Config};

let source = r#"
#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 { x + 1 }
"#;

let config = Config::default();
let formatted = format_file(source, &config)?;
```

## How It Works

1. **Find** - Locate `#[spec(` patterns in source text using simple string search
2. **Parse** - Parse each spec using `anodized-core` into a `Spec` struct
3. **Format** - Walk the `Spec` struct and emit formatted text
4. **Replace** - Substitute the original attribute with the formatted version

Individual expressions are formatted using `quote` for consistency with Rust formatting.
