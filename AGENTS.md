# AGENTS.md

Guidelines for agentic coding agents working in this repository.

## Project Overview

This is a Rust implementation of DEFLATE compression (LZ77 + Huffman coding). The project is a library (`m_compressor`) with a binary entry point that reads a file path from stdin and compresses the file.

## Build/Lint/Test Commands

```bash
# Build (debug)
cargo build --manifest-path m_compressor/Cargo.toml

# Build (release)
cargo build --release --manifest-path m_compressor/Cargo.toml

# Run the binary
cargo run --manifest-path m_compressor/Cargo.toml

# Run all tests
cargo test --manifest-path m_compressor/Cargo.toml

# Run a single test by name
cargo test --manifest-path m_compressor/Cargo.toml <test_name>

# Run a single test file (integration tests)
cargo test --manifest-path m_compressor/Cargo.toml --test compressor_tests

# Run tests in a specific module
cargo test --manifest-path m_compressor/Cargo.toml huffman::tests

# Lint with clippy
cargo clippy --manifest-path m_compressor/Cargo.toml

# Format code
cargo fmt --manifest-path m_compressor/Cargo.toml

# Check formatting without applying
cargo fmt --manifest-path m_compressor/Cargo.toml -- --check

# Check for compilation errors without building
cargo check --manifest-path m_compressor/Cargo.toml
```

## Code Style Guidelines

### Imports

- Group imports by scope: standard library first, then external crates, then local modules
- Use `use` statements at the top of the file
- Import specific items rather than glob imports (`use std::fs::File` not `use std::fs::*`)
- For local modules, use `crate::` prefix for absolute paths

```rust
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::{
    compressor::{CompressError, lz77::LzSymbol},
    constants,
    utils::bit_writer::BitWriter,
};
```

### Module Structure

- Public modules: `pub mod module_name;`
- Internal modules: `pub(crate) mod module_name;`
- Re-export public API from `lib.rs` and module `mod.rs` files
- Keep module structure flat where possible

### Naming Conventions

- **Functions/Variables**: `snake_case`
- **Types/Structs/Enums/Traits**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Type Aliases**: `PascalCase` (e.g., `type LitLenCntArr = [usize; 286];`)
- **Static Variables**: `SCREAMING_SNAKE_CASE`

### Types

- Prefer strong typing with newtypes or enums over primitive types
- Use type aliases for complex types to improve readability
- Use `impl AsRef<Path>` for function parameters accepting file paths
- Use `PathBuf` for owned paths, `&Path` for borrowed paths

### Error Handling

- Use `Result<T, E>` for fallible operations
- Define custom error enums with `#[derive(Debug)]`
- Use `map_err` to transform errors with context
- Print errors with `println!` before returning error variants

```rust
#[derive(Debug)]
pub enum CompressError {
    FileOpen,
    StreamRead,
    FileWrite,
}

File::open(&self.in_file_path).map_err(|err| {
    println!("Error: {err}");
    CompressError::FileOpen
})?;
```

### Comments

- Use block comments `/* */` for module-level documentation
- Use inline comments `//` for implementation explanations
- Document public APIs with `///` doc comments
- Include rationale in comments for non-obvious decisions

### Formatting

- Use 4-space indentation (Rust standard)
- Place opening braces on the same line
- Use trailing commas in multi-line arrays/structs
- Break long lines at logical points (after operators, after commas)

### Testing

- Place unit tests in `#[cfg(test)] mod tests` within the source file
- Place integration tests in `m_compressor/tests/` directory
- Use `#[test]` attribute for test functions
- Use `assert!`, `assert_eq!`, and `assert_ne!` macros
- Clean up test files after tests complete

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // test implementation
    }
}
```

### Memory and Performance

- Use `VecDeque` for queue-like data structures
- Use `BufReader`/`BufWriter` for I/O operations with appropriate capacity
- Define buffer sizes as constants (e.g., `READER_CAPACITY`, `BUFFER_CAPACITY`)
- Use `static` for compile-time constants, `const` for inline constants

### Project-Specific Patterns

- LZ77 symbols are represented by the `LzSymbol` enum with `Literal` and `Pointer` variants
- Huffman codes use canonical form with bit-reversal for DEFLATE compatibility
- BitWriter writes LSB-first (DEFLATE standard)
- Constants for DEFLATE alphabet sizes and base codes are in `constants.rs`

## File Structure

```
m_compressor/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public module exports
│   ├── main.rs          # Binary entry point
│   ├── constants.rs     # DEFLATE constants
│   ├── compressor/
│   │   ├── mod.rs       # MCompressor struct, public API
│   │   ├── huffman.rs   # Huffman encoding
│   │   └── lz77.rs      # LZ77 compression
│   └── utils/
│       ├── mod.rs
│       ├── bit_writer.rs    # Bit-level output
│       └── package_merge.rs # Limited code length algorithm
└── tests/
    └── compressor_tests.rs  # Integration tests
```

## Important Notes

- The project uses Rust edition 2024
- No external dependencies (pure Rust implementation)
- Implements DEFLATE with dynamic Huffman blocks (BTYPE=10)
- Maximum Huffman code length is 15 bits (DEFLATE standard)
- Window size for LZ77 is 32KB (2^15 bytes)
