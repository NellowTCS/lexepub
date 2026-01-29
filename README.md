# lexepub: Ultra-Fast Rust EPUB Parsing & Extraction

lexepub is a high-performance, streaming EPUB parser and extractor (and hopefully renderer) for Rust, WASM, and C/C++.

## Need to do:
- Stream/process EPUBs chapter-by-chapter, not buffered in memory
- Leverage lexbor for blazing-fast HTML/XHTML parsing
- Simplicity: Provide minimal, clean output (no DOM, no iframe)
- Modularity: Easily plugged into any front-end (Web, CLI, desktop)
- Async support for I/O-bound workloads and efficient resource use
- WASM support and C/C++ bindings


TODO: Finish the readme