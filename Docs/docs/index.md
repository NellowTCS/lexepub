---
title: "LexePub"
description: "High-performance, memory-efficient EPUB parsing for multiple runtimes"
---

**LexePub** is a robust parser and extractor for EPUB files. It provides asynchronous streaming, metadata validation, and asset extraction across Rust, C/C++, and WebAssembly from a single core implementation.

::: callout tip
LexePub focuses heavily on minimizing memory footprint. You can stream chapter extraction and still get structured AST output with CSS styles applied.
:::

## What it solves

You're building an e-reader or book management system that needs to process EPUB files efficiently. Normally, that means loading massive archives entirely into memory, dealing with OPF manifest edge cases, or juggling different APIs for each runtime.

LexePub replaces that with one core parser and thin language adapters. Parse once using memory-safe async streams, then consume the same chapter/metadata behavior from Rust, WASM, and C/C++.

## Features

::: card Streaming-Friendly Core
Read chapters sequentially and process content without requiring full-book materialization in memory.
:::

::: card Multi-Platform Adapters
Written in async Rust at the core, exported to C/C++ via Diplomat and JavaScript/TypeScript via WebAssembly.
:::

::: card Strict Metadata Validation
Automatically detects EPUB versions (2.0 vs 3.0), resolves OPF manifest links, and validates required metadata structure according to the standards. 
:::

::: card CSS-Aware AST
Manifest CSS resources are parsed and applied to chapter AST nodes. Styles are merged into each element's `styles` map.
:::

## API

- Rust adapter: async API plus sync wrappers for metadata/count/cover operations.
- WASM adapter: async JS-facing surface for loading bytes, metadata, chapter text, AST chapter retrieval, counts, and cover image.
- C/C++ adapter: currently focused on constructor plus aggregate counters and cover presence.

For complete setup instructions, see [Quick Start](/getting-started/quickstart).
For runtime-specific APIs, see:

- [Rust Adapter](/adapters/rust/index)
- [C/C++ Adapter](/adapters/c/index)
- [WASM Adapter](/adapters/wasm/index)
