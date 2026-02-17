# TODO

- [ ] Implement AST parsing in main API
  - Currently `extract_chapters()` sets `ast: None`
  - Should use `ChapterParser::with_ast()` instead of `extract_text_content()`

- [ ] Add missing fields to EpubMetadata
  - [ ] Add `spine: Vec<String>` for chapter order
  - [ ] Add `has_cover: bool` for cover image presence
  - [ ] Add `chapter_count: usize` for number of chapters
  - [ ] Rename `date` to `publication_date` for API consistency

- [ ] Proper WASM support

- [ ] Add cover image support
  - [ ] Add `has_cover()`
  - [ ] Add `cover_image()`
  - [ ] Parse OPF manifest for cover image detection
  - [ ] Extract cover image data from EPUB files

- [ ] Add docs

- [ ] Add streaming cover image support

- [ ] Add image format detection for covers

- [ ] Add metadata validation

- [ ] Add EPUB version detection

- [ ] Add rendering support

- [ ] Implement CSS parsing and application

- [ ] Add JavaScript execution support for interactive EPUBs

- [ ] Integrate the ✨ fancy ✨ EPUB library features (advanced layout, multimedia, accessibility, etc. )
