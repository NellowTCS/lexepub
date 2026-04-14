---
title: "Quick Start"
description: "Get started parsing EPUB files with LexEpub"
---

# Quick start

This guide outlines how to configure your project to use LexEpub. It assumes you are setting up the Rust adapter inside a standard Cargo project.

For other language adapters, refer to the [Adapters](/adapters/index.md) index.

## Installation

Add the core library to your Rust dependencies in `Cargo.toml`:

```toml
[dependencies]
lexepub = "0.1.0"
tokio = { version = "1", features = ["full"] }
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

::: callout tip
The `open` invocation guarantees that the underlying archive structure is verified without fully loading the uncompressed data into memory. Check the adapter documentation for advanced usage regarding streams.
:::
