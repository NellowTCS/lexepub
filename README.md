# lexepub

lexepub is a high-performance, streaming EPUB parser and extractor for Rust, WASM, and C/C++.

## Features

- **Streaming Processing**: Process EPUBs chapter-by-chapter without loading everything into memory
- **Fast HTML Parsing**: Uses the `scraper` crate for efficient HTML/XHTML parsing
- **Multiple Output Formats**: Extract plain text, HTML, or AST representations
- **Async Support**: Built on Tokio for efficient I/O operations
- **Cross-Platform**: Works on desktop, web (WASM), and embedded systems
- **C/C++ Bindings**: Use from C/C++ applications via Diplomat
- **Zero-Copy**: Efficient memory usage with minimal allocations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lexepub = "0.1"
```

For WASM support:

```toml
[dependencies]
lexepub = { version = "0.1", features = ["wasm"] }
```

For C/C++ FFI:

```toml
[dependencies]
lexepub = { version = "0.1", features = ["c-ffi"] }
```

## Quick Start

### Basic Usage

```rust
use lexepub::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an EPUB file
    let mut epub = LexEpub::open("path/to/book.epub").await?;

    // Extract metadata
    let metadata = epub.get_metadata().await?;
    println!("Title: {}", metadata.title);
    println!("Authors: {}", metadata.authors.join(", "));

    // Extract all text content
    let chapters = epub.extract_text_only().await?;
    println!("Found {} chapters", chapters.len());

    // Get word and character counts
    let total_words = epub.total_word_count().await?;
    let total_chars = epub.total_char_count().await?;
    println!("Total words: {}, Total characters: {}", total_words, total_chars);

    Ok(())
}
```

### Advanced Usage with AST

```rust
use lexepub::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut epub = LexEpub::open("book.epub").await?;

    // Extract chapters with AST for advanced processing
    let chapters = epub.extract_with_ast().await?;

    for chapter in chapters {
        println!("Chapter content length: {}", chapter.content.len());
        if let Some(ast) = chapter.ast {
            // Process the AST (e.g., extract specific elements)
            process_ast(&ast);
        }
    }

    Ok(())
}

fn process_ast(ast: &AstNode) {
    match ast {
        AstNode::Element { tag, children, .. } => {
            println!("Found element: {}", tag);
            for child in children {
                process_ast(child);
            }
        }
        AstNode::Text { content } => {
            println!("Text content: {}", content);
        }
        _ => {}
    }
}
```

### Convenience Functions

```rust
use lexepub::{extract_text_from_epub, extract_metadata};

// Quick text extraction
let chapters = extract_text_from_epub("book.epub").await?;

// Quick metadata extraction
let metadata = extract_metadata("book.epub").await?;
```

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

## API Reference

### Core Types

- **`LexEpub`**: Main EPUB processor with async methods
- **`EpubMetadata`**: Structured metadata (title, authors, languages, etc.)
- **`ParsedChapter`**: Individual chapter with content, AST, and statistics
- **`AstNode`**: HTML AST node representation

### Key Methods

#### LexEpub

- `open(path)`: Open EPUB from file path
- `from_bytes(data)`: Open EPUB from byte array
- `get_metadata()`: Extract metadata
- `extract_text_only()`: Get plain text chapters
- `extract_with_ast()`: Get chapters with AST
- `total_word_count()`: Count words across all chapters
- `total_char_count()`: Count characters across all chapters
- `has_cover()`: Check for cover image
- `cover_image()`: Extract cover image data

## WASM Usage

```javascript
import init, { WasmEpubExtractor } from './pkg/lexepub.js';

async function processEpub(arrayBuffer) {
    await init();

    const extractor = new WasmEpubExtractor();
    const uint8Array = new Uint8Array(arrayBuffer);

    await extractor.load_from_bytes(uint8Array);

    const metadata = await extractor.get_metadata();
    console.log('Title:', metadata.title);

    const chapters = await extractor.get_chapters_text();
    console.log('Chapters:', chapters.length);

    const wordCount = await extractor.get_total_word_count();
    console.log('Total words:', wordCount);
}
```

## C/C++ Usage

```c
#include "lexepub.h"

int main() {
    // Load EPUB from bytes
    diplomat_result_box_EpubExtractor extractor_result =
        EpubExtractor_from_bytes(data, data_len);

    if (extractor_result.is_ok) {
        EpubExtractor* extractor = extractor_result.ok;

        // Get metadata
        diplomat_result_void metadata_result =
            EpubExtractor_get_metadata(extractor, &output);

        // Get chapter text
        diplomat_result_void chapter_result =
            EpubExtractor_get_chapter_text(extractor, 0, &output);

        // Clean up
        EpubExtractor_destroy(extractor);
    }

    return 0;
}
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
