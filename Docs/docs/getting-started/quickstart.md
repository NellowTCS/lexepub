---
title: "Quick Start"
description: "Get started parsing EPUB files with LexEpub"
---

# Quick start

This guide outlines how to configure your project to use LexEpub. It assumes you are setting up the Rust adapter inside a standard Cargo project.

For other language adapters, refer to:

- [Rust Adapter](/adapters/rust/index)
- [C/C++ Adapter](/adapters/c/index)
- [WASM Adapter](/adapters/wasm/index)

## Installation

Add the core library to your Rust dependencies in `Cargo.toml`:

```toml
[dependencies]
lexepub = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

Optional features:

```toml
lexepub = { version = "0.1.0", features = ["c-ffi", "wasm"] }
```

## Basic usage

The fundamental entry point for parsing is the `LexEpub` struct. Here is how you initialize the parser and request standard metadata:

```rust
use lexepub::LexEpub;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Instantiate a new parser against a local file.
    let mut epub = LexEpub::open("book.epub").await?;
    
    // Asynchronously resolve and extract OPF metadata.
    let metadata = epub.get_metadata().await?;
    
    // Output the detected EPUB version.
    println!("Detected EPUB version: {:?}", metadata.version);
    Ok(())
}
```

## Core async methods

Use these methods for most workflows:

- `extract_text_only()` for plain chapter text.
- `extract_ast()` for chapter data with parsed AST.
- `extract_chapters_stream()` for stream-based chapter processing.
- `total_word_count()` and `total_char_count()` for aggregate analysis.
- `has_cover()` and `cover_image()` for cover lookup/extraction.

## Sync wrappers

For non-async contexts, `LexEpub` also exposes sync wrappers:

- `open_sync()`
- `get_metadata_sync()`
- `validate_metadata_sync()`
- `total_word_count_sync()`
- `total_char_count_sync()`
- `has_cover_sync()`
- `cover_image_sync()`

## Convenience functions

```rust
use lexepub::{extract_ast, extract_text_only, get_metadata};

let text = extract_text_only("book.epub").await?;
let ast = extract_ast("book.epub").await?;
let metadata = get_metadata("book.epub").await?;
```

## CSS behavior in AST mode

When you call `extract_ast()`, LexePub reads `text/css` resources from the OPF manifest, parses them, and applies declarations onto AST elements.

- AST styles are available in `AstNode::Element.styles`.
- Inline `style` attributes are merged too.
- Selector matching is intentionally EPUB-focused (tag, class, id, grouped selectors).

::: callout tip
The `open` invocation guarantees that the underlying archive structure is verified without fully loading the uncompressed data into memory. Check the adapter documentation for advanced usage regarding streams.
:::
