---
title: "Rust Adapter"
description: "Native asynchronous integration for Rust developers"
---

# Rust Adapter

The native Rust API exposes the full LexePub capability set through `tokio` or any compatible async executor.

## Primary types

- `LexEpub`: main parser/entry point.
- `EpubMetadata`: normalized metadata model.
- `ParsedChapter`: chapter payload with text, counts, and optional AST.
- `AstNode`: HTML AST node model (`Element`, `Text`, `Comment`).

## Async API reference

- `LexEpub::open(path)`
- `LexEpub::from_bytes(data)`
- `LexEpub::from_reader(reader)`
- `LexEpub::extract_text_only()`
- `LexEpub::extract_ast()`
- `LexEpub::extract_chapters_stream()`
- `LexEpub::get_metadata()`
- `LexEpub::validate_metadata()`
- `LexEpub::get_toc()`
- `LexEpub::read_resource(path)`
- `LexEpub::resolve_chapter_resource_path(chapter_index, href)`
- `LexEpub::read_chapter_resource(chapter_index, href)`
- `LexEpub::total_word_count()`
- `LexEpub::total_char_count()`
- `LexEpub::has_cover()`
- `LexEpub::cover_image()`
- `LexEpub::cover_image_to_writer(writer)`

## Sync wrapper API

- `LexEpub::open_sync(path)`
- `LexEpub::from_sync_reader(reader)`
- `LexEpub::get_metadata_sync()`
- `LexEpub::validate_metadata_sync()`
- `LexEpub::total_word_count_sync()`
- `LexEpub::total_char_count_sync()`
- `LexEpub::has_cover_sync()`
- `LexEpub::cover_image_sync()`

## CSS and AST integration

Calling `extract_ast()` performs CSS-aware AST generation:

- OPF `text/css` manifest items are parsed.
- Rules are applied to AST element nodes.
- Applied declarations are stored in `AstNode::Element.styles`.
- Inline style attributes are merged during application.

Also, chapter-relative `href`/`src` values are normalized during extraction to make runtime rendering/link navigation simpler across adapters.

## Building and Testing

Simply add LexePub as a Cargo dependency. Integration tests and API tests ensure the parsing logic remains deterministic and resilient across versions.

```bash
cargo build --release
cargo test
```

## Convenience functions

The crate also exports async helpers:

- `extract_text_only(path)`
- `extract_ast(path)`
- `get_metadata(path)`

## Extra analysis helpers

The core module also exposes high-level analysis entry points:

- `analyze_reader(reader)`
- `analyze_sync_reader(reader)`
