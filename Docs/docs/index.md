---
title: "LexePub"
description: "High-performance, memory-efficient EPUB parsing for multiple runtimes"
---

**LexePub** is a robust parser and extractor for EPUB files. It provides zero-allocation asynchronous streaming, metadata validation, an
d asset extraction across Rust, C/C++, and WebAssembly from a single core implementing a shared set of primitives.

::: callout tip
LexePub focuses heavily on minimizing memory footprint. The idea: you should be able to stream an embedded cover image or parse an entir
e book's HTML AST without pulling out the entire ZIP archive into RAM.
:::

## What it solves

You're building an e-reader or book management system that needs to process EPUB files efficiently. Normally, that means loading massive archives entirely into memory, fighting with poorly documented OPF manifest parsing, or dealing with blocking I/O calls that freeze your UI.

LexePub replaces all of that with a single streamlined core library and a set of thin language adapters. You parse your EPUBs once using memory-safe async streams, LexePub handles the extraction logic, and your application—whether Native, Web, or Embedded—just streams the output.

## Features

::: card Zero-Allocation Streaming
Direct `AsyncWrite` pipelining allows pulling compressed archive contents, like cover images, straight to your buffers or network stream
s without loading huge vectors in memory.
:::

::: card Multi-Platform Adapters
Written in async Rust at the core, but exported with guaranteed 1-1 API parity to C/C++ via Diplomat and JavaScript/TypeScript via WebAssembly.
:::

::: card Strict Metadata Validation
Automatically detects EPUB versions (2.0 vs 3.0), resolves OPF manifest links, and validates required metadata structure according to the standards. 
:::

For complete setup instructions, please see the [Quick Start](/getting-started/quickstart) guide.
