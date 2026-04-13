# Todo

- [ ] Proper WASM support

- [ ] Add docs

- [ ] Add streaming cover image support

- [ ] Add image format detection for covers

- [ ] Add metadata validation

- [ ] Add EPUB version detection

- [ ] Add rendering support

- [ ] Implement CSS parsing and application

- [ ] Add JavaScript execution support for interactive EPUBs (probably beyond v0.1.0)

- [ ] Integrate the ✨ fancy ✨ EPUB library features (advanced layout, multimedia, accessibility, etc. ) (probably beyond v0.1.0)

## Done

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
