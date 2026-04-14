---
title: "WASM Adapter"
description: "Cross-platform bindings for browsers and Node.js environments"
---

# WebAssembly Adapter

LexePub exposes its async features natively to JavaScript and TypeScript contexts using `wasm-pack`. This adapter provides exact feature parity with the native bindings, allowing memory-efficient parsing running directly inside browsers or on Edge network workers.

## Publishing and Setup

The library will eventually will be added to NPM. But, currently, the package must be built locally using `wasm-pack`.

```bash
# Build the artifact for web invocation
wasm-pack build --target web
```

All standard calls mapped in the Rust ABI map seamlessly into JS asynchronous primitives.
