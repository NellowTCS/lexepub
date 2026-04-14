---
title: "Rust Adapter"
description: "Native asynchronous integration for Rust developers"
---

# Rust Adapter

The native Rust API exposes the full capabilities of LexePub through `tokio` or generic asynchronous executors. It acts as the backbone reference implementation for all other language adapters, supporting low-overhead async I/O streams and full AST HTML parsing.

## Building and Testing

Simply add LexePub as a Cargo dependency. Integration tests and API tests ensure the parsing logic remains deterministic and resilient across versions.

```bash
cargo build --release
cargo test
```

::: callout tip
When pulling large datasets, use the `read_file_to_writer` or `cover_image_to_writer` abstractions with the Rust adapter. These functions take generic arguments implementing `futures::AsyncWrite`, which keeps allocation sizes tightly bounded.
:::
