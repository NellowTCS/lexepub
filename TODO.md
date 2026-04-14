# Todo

- [ ] Add rendering support (probably beyond v0.1.0 sadly)

- [ ] Add JavaScript execution support for interactive EPUBs (probably beyond v0.1.0)

- [ ] Integrate the ✨ fancy ✨ EPUB library features (advanced layout, multimedia, accessibility, etc. ) (probably beyond v0.1.0)

## Done

- [X] making links work to navigate inside the book

- [X] alt text for images

- [X] chapter images

- [X] proper table of contents using page titles rather than name of html files

- [X] making sure css from <link>'s also works, 

- [X] Add Demo

- [X] Ensure 1-1 API functionality between C/C++, Rust, and WASM

- [X] Implement CSS parsing and application

- [x] Add streaming cover image support
  - Implemented `cover_image_to_writer` allowing data forwarding using zero allocations.

- [x] Add docs

- [x] Add metadata validation
  - Implemented `validate_metadata` method and `ValidationError` handling.

- [x] Add image format detection for covers

- [x] Add EPUB version detection

- [x] Add image format detection for covers
  - Parses `media-type` from OPF manifest for the cover image.

- [x] Add EPUB version detection
  - Parses `<package version="X.X">` in OPF file

- [x] Proper WASM support

- [x] Implement AST parsing in main API
  - Currently `extract_chapters()` sets `ast: None`
  - Should use `ChapterParser::with_ast()` instead of `extract_text_content()`

- [x] Add missing fields to EpubMetadata
  - [x] Add `spine: Vec<String>` for chapter order
  - [x] Add `has_cover: bool` for cover image presence
  - [x] Add `chapter_count: usize` for number of chapters
  - [x] Rename `date` to `publication_date` for API consistency

- [x] Add cover image support
  - [x] Add `has_cover()`
  - [x] Add `cover_image()`
  - [x] Parse OPF manifest for cover image detection
  - [x] Extract cover image data from EPUB files
