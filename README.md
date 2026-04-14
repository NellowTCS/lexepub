# lexepub

lexepub is a high-performance, streaming EPUB parser and extractor for Rust, WASM, and C/C++.

## Features

- **Streaming Processing**: Process EPUBs chapter-by-chapter without loading everything into memory
- **Fast HTML Parsing**: Uses the `scraper` crate for efficient HTML/XHTML parsing
- **Multiple Output Formats**: Extract plain text or AST representations
- **Async Support**: Built on Tokio for efficient I/O operations
- **Cross-Platform**: Works on desktop, web (WASM), and embedded systems
- **C/C++ Bindings**: Use from C/C++ applications via Diplomat
- **Sync + Async Parity**: Core Rust methods have synchronous wrappers where needed
- **CSS Parsing + Application**: Stylesheets are parsed and applied onto AST element styles
- **Zero-Copy**: Efficient memory usage with minimal allocations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lexepub = "0.1"
```

## Demo

A modified version of [HTMLPlayer](https://github.com/HTMLToolkit/HTMLReader) is available as a demo at <https://nellowtcs.me/lexepub/>!
The demo uses the WASM adapter for LexePub.

## Documentation

Docs are available at <https://nellowtcs.me/lexepub/docs>!

- Rust adapter API: <https://nellowtcs.me/lexepub/docs/adapters/rust>
- C/C++ adapter API: <https://nellowtcs.me/lexepub/docs/adapters/c>
- WASM adapter API: <https://nellowtcs.me/lexepub/docs/adapters/wasm>
- Quickstart: <https://nellowtcs.me/lexepub/docs/getting-started/quickstart>

## Building

### Standard Build

```bash
cd lexepub
cargo build --release
```

### With C/C++ FFI Support

To generate C bindings, you need the `diplomat-tool`:

```bash
# Install diplomat-tool
cargo install diplomat-tool

# Build with FFI support
cd lexepub
cargo build --release --features c-ffi

# Generate C headers and bindings
diplomat-tool c include/
```

**Note**: The C FFI bindings require the `diplomat` and `diplomat-runtime` crates. If `diplomat-tool` is not available, the build will skip FFI generation but still compile successfully.

### WASM Build

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build for web
cd lexepub
wasm-pack build --release --target web
```

## Performance

lexepub is designed for high performance:

- **Memory Efficient**: Streaming processing prevents loading entire EPUBs into memory
- **Fast Parsing**: Uses optimized HTML parsing with `scraper`
- **Async I/O**: Non-blocking file operations with Tokio
- **Zero-Copy**: Minimal allocations and copying where possible

## Examples

Run the included CLI tool:

```bash
cargo run --bin lexepub -- path/to/your/book.epub
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under the Apache License, Version 2.0.
