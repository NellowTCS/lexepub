// Streaming JSON serializer that writes directly to DiplomatWriteable
// without allocating an intermediate String (avoids OOM on memory-constrained targets).

struct DiplomatWriter<'a>(&'a mut diplomat_runtime::DiplomatWrite);

impl<'a> std::io::Write for DiplomatWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "not utf-8"))?;
        <diplomat_runtime::DiplomatWrite as core::fmt::Write>::write_str(&mut *self.0, s)
            .map_err(|_| std::io::Error::other("write failed"))?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn write_json_streaming<T: serde::Serialize>(
    to: &mut diplomat_runtime::DiplomatWrite,
    value: &T,
) -> Result<(), ()> {
    serde_json::to_writer(DiplomatWriter(to), value).map_err(|_| ())
}

#[diplomat::bridge]
#[allow(clippy::module_inception)]
mod ffi {
    use core::fmt::Write as _;

    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct EpubExtractor(Box<crate::LexEpub>);

    impl EpubExtractor {
        fn write_string(to: &mut diplomat_runtime::DiplomatWrite, s: &str) -> Result<(), ()> {
            to.write_str(s).map_err(|_| ())
        }

        pub fn create(path: &str) -> Option<Box<EpubExtractor>> {
            let path_buf = std::path::PathBuf::from(path);
            match crate::LexEpub::open_sync(path_buf) {
                Ok(lexepub) => Some(Box::new(EpubExtractor(Box::new(lexepub)))),
                Err(_) => None,
            }
        }

        pub fn create_from_bytes(data: &[u8]) -> Option<Box<EpubExtractor>> {
            let bytes = bytes::Bytes::copy_from_slice(data);
            match futures::executor::block_on(crate::LexEpub::from_bytes(bytes)) {
                Ok(lexepub) => Some(Box::new(EpubExtractor(Box::new(lexepub)))),
                Err(_) => None,
            }
        }

        pub fn get_metadata_is_valid(&mut self) -> bool {
            self.0.validate_metadata_sync().is_ok()
        }

        pub fn get_chapter_count(&mut self) -> usize {
            self.0
                .get_metadata_sync()
                .map(|m| m.chapter_count)
                .unwrap_or(0)
        }

        pub fn get_title(&mut self, to: &mut diplomat_runtime::DiplomatWrite) -> Result<(), ()> {
            let title = self
                .0
                .get_metadata_sync()
                .ok()
                .and_then(|m| m.title)
                .unwrap_or_default();
            Self::write_string(to, &title)
        }

        pub fn get_metadata_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let metadata = self.0.get_metadata_sync().map_err(|_| ())?;
            super::write_json_streaming(to, &metadata)
        }

        pub fn get_metadata(&mut self, to: &mut diplomat_runtime::DiplomatWrite) -> Result<(), ()> {
            self.get_metadata_json(to)
        }

        pub fn get_chapters_text_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let chapters =
                futures::executor::block_on(self.0.extract_text_only()).map_err(|_| ())?;
            super::write_json_streaming(to, &chapters)
        }

        pub fn get_chapters_text(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            self.get_chapters_text_json(to)
        }

        pub fn get_chapter_text(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let text = match futures::executor::block_on(self.0.extract_text_only()) {
                Ok(chapters) => chapters.get(index).cloned().ok_or(())?,
                Err(_) => return Err(()),
            };
            Self::write_string(to, &text)
        }

        #[cfg(not(feature = "lowmem"))]
        pub fn get_chapter_json(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let chapter = match futures::executor::block_on(self.0.extract_ast()) {
                Ok(chapters) => chapters.get(index).cloned().ok_or(())?,
                Err(_) => return Err(()),
            };
            super::write_json_streaming(to, &chapter)
        }

        /// Extract a single chapter without loading all chapters into memory.
        /// Uses extract_single_chapter() to read only the requested chapter file.
        pub fn get_single_chapter_json(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let chapter = match futures::executor::block_on(self.0.extract_single_chapter(index)) {
                Ok(c) => c,
                Err(_) => return Err(()),
            };
            super::write_json_streaming(to, &chapter)?;
            // Keep the text buffer alive across calls to avoid heap fragmentation
            self.0.save_text_buffer(chapter.content);
            Ok(())
        }

        pub fn get_toc_json(&mut self, to: &mut diplomat_runtime::DiplomatWrite) -> Result<(), ()> {
            let toc = futures::executor::block_on(self.0.get_toc()).map_err(|_| ())?;
            super::write_json_streaming(to, &toc)
        }

        pub fn get_toc(&mut self, to: &mut diplomat_runtime::DiplomatWrite) -> Result<(), ()> {
            self.get_toc_json(to)
        }

        pub fn resolve_chapter_resource_path(
            &mut self,
            chapter_index: usize,
            href: &str,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let resolved = futures::executor::block_on(
                self.0.resolve_chapter_resource_path(chapter_index, href),
            )
            .map_err(|_| ())?;
            Self::write_string(to, &resolved)
        }

        pub fn get_resource_json(
            &mut self,
            path: &str,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let bytes = futures::executor::block_on(self.0.read_resource(path)).map_err(|_| ())?;
            super::write_json_streaming(to, &bytes)
        }

        pub fn get_chapter_resource_json(
            &mut self,
            chapter_index: usize,
            href: &str,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let bytes =
                futures::executor::block_on(self.0.read_chapter_resource(chapter_index, href))
                    .map_err(|_| ())?;
            super::write_json_streaming(to, &bytes)
        }

        #[cfg(not(feature = "lowmem"))]
        pub fn get_chapter(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            self.get_chapter_json(index, to)
        }

        pub fn get_total_word_count(&mut self) -> usize {
            self.0.total_word_count_sync().unwrap_or(0)
        }

        pub fn get_total_char_count(&mut self) -> usize {
            self.0.total_char_count_sync().unwrap_or(0)
        }

        pub fn has_cover(&mut self) -> bool {
            self.0.has_cover_sync().unwrap_or(false)
        }

        pub fn get_cover_image_len(&mut self) -> usize {
            self.0
                .cover_image_sync()
                .map(|bytes| bytes.len())
                .unwrap_or(0)
        }

        pub fn get_cover_image_format(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let mime = self
                .0
                .get_metadata_sync()
                .ok()
                .and_then(|m| m.cover_image_format)
                .unwrap_or_default();
            Self::write_string(to, &mime)
        }

        pub fn get_cover_image_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWrite,
        ) -> Result<(), ()> {
            let bytes = self.0.cover_image_sync().map_err(|_| ())?;
            super::write_json_streaming(to, &bytes)
        }

        /// Create a streaming text reader for a single chapter.
        ///
        /// The returned stream yields cleaned text (HTML tags stripped) in
        /// fixed-size chunks.  Call `read_chunk` in a loop until it returns
        /// `false`.
        ///
        /// Unlike `get_single_chapter_json`, this path avoids JSON
        /// serialisation, AST construction, and large contiguous C-side
        /// buffers.  Each chunk is at most 1024 bytes.
        ///
        /// Internally uses the `lowmem` streaming HTML-to-text path so peak
        /// Rust-side allocation during extraction is bounded to a few KB per
        /// decompress cycle, plus the final cleaned text (which the C caller
        /// consumes incrementally).
        pub fn open_chapter_text_stream(&mut self, index: usize) -> Option<Box<ChapterTextStream>> {
            let chapter = futures::executor::block_on(self.0.extract_single_chapter(index)).ok()?;
            let content = chapter.content.into_bytes();
            Some(Box::new(ChapterTextStream { content, pos: 0 }))
        }

    /// Create a streaming formatting-aware reader for a single chapter.
    ///
    /// Unlike the plain-text stream, this returns individual
    /// `FormattingRun` records with style flags and heading level so the
    /// C renderer can style text properly instead of relying on
    /// heuristic line-splitting.
    ///
    /// Usage:
    ///   while (next_run()) {
    ///       style  = run_style();
    ///       heading = run_heading();
    ///       run_text(to);     // writes run text to DiplomatWrite
    ///       // process one styled run
    ///   }
    ///
    /// Internally uses quick-xml StAX parsing driven incrementally by
    /// `next_run()`.  Only one run's text is held in memory at a time,
    /// so chapter size is bounded only by the single largest run (typically
    /// a few hundred bytes).
    pub fn open_chapter_formatting_stream(
        &mut self,
        index: usize,
    ) -> Option<Box<ChapterFormattingStream>> {
        let (html_bytes, _resolved_path) =
            futures::executor::block_on(self.0.read_chapter_raw(index)).ok()?;
        Some(Box::new(ChapterFormattingStream::new(html_bytes)))
    }
    }

    /// Streaming text reader for a single EPUB chapter.
    ///
    /// Created via `EpubExtractor::open_chapter_text_stream`.  Each call to
    /// `read_chunk` writes a fixed-size chunk (up to `CHUNK_SIZE` bytes) of
    /// cleaned chapter text into the provided `DiplomatWrite`.
    /// Returns `false` when all text has been consumed.
    ///
    /// Callers **must** provide a buffer of at least `CHUNK_SIZE + 1` bytes
    /// via `diplomat_simple_write(buf, CHUNK_SIZE + 1)`.  When `read_chunk`
    /// returns `true`, the caller processes `to.len` bytes from `to.buf`,
    /// creates a fresh `DiplomatWrite`, and calls again.
    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct ChapterTextStream {
        content: Vec<u8>,
        pos: usize,
    }

    impl ChapterTextStream {
        /// Read the next chunk of cleaned chapter text.
        ///
        /// Writes up to 1024 bytes of text into `to`.  Returns `true` when
        /// data was written (caller must process `to.len` bytes), `false`
        /// when the stream is exhausted (no data written).
        ///
        /// The C caller **must** create `to` with `diplomat_simple_write(buf, n)`
        /// where `n >= 1025`.  After a `true` return the caller processes the
        /// first `to.len` bytes from the buffer, creates a fresh `DiplomatWrite`,
        /// and calls again.
        pub fn read_chunk(&mut self, to: &mut diplomat_runtime::DiplomatWrite) -> bool {
            if self.pos >= self.content.len() {
                return false;
            }
            let remaining = self.content.len() - self.pos;
            let n = remaining.min(1024);
            let chunk = &self.content[self.pos..self.pos + n];
            // SAFETY: content was created from a String, so it is valid UTF-8.
            let s = unsafe { core::str::from_utf8_unchecked(chunk) };
            // n <= CHUNK_SIZE.  C caller must provide cap >= CHUNK_SIZE + 1
            // so this write always fits in the buffer.
            let _ = to.write_str(s);
            self.pos += n;
            true
        }
    }

    /// Streaming reader for formatting-aware chapter text.
    ///
    /// Created via `EpubExtractor::open_chapter_formatting_stream`.
    /// Iterate styled runs using `next_run()` / `run_style()` / `run_heading()`
    /// / `run_text()`.
    ///
    /// Internally uses quick-xml StAX parsing.  Each call to `next_run()`
    /// advances the XML reader forward to the next text-yielding event and
    /// caches that run's data.  Only one run's text (typically a few hundred
    /// bytes) is held in memory at any time.
    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct ChapterFormattingStream {
        reader: quick_xml::Reader<std::io::Cursor<Vec<u8>>>,
        buf: Vec<u8>,
        style_stack: Vec<u8>,
        heading_level: u8,
        text: String,
        style: u8,
        heading: u8,
        valid: bool,
        eof: bool,
    }

    impl ChapterFormattingStream {
        fn new(html_bytes: Vec<u8>) -> Self {
            let cursor = std::io::Cursor::new(html_bytes);
            let mut reader = quick_xml::Reader::from_reader(cursor);
            reader.config_mut().trim_text(true);
            Self {
                reader,
                buf: Vec::new(),
                style_stack: Vec::new(),
                heading_level: 0,
                text: String::new(),
                style: 0,
                heading: 0,
                valid: false,
                eof: false,
            }
        }

        fn active_style(&self) -> u8 {
            self.style_stack.iter().copied().fold(0, |acc, s| acc | s)
        }

        fn tag_is_style_start(_name: &[u8]) -> Option<u8> {
            #[cfg(feature = "lowmem")]
            { crate::core::html_parser::streaming::tag_is_style_start(_name) }
            #[cfg(not(feature = "lowmem"))]
            { None }
        }

        fn is_heading_tag(name: &[u8]) -> Option<u8> {
            let len = name.len();
            if len == 2 && (name[0] | 0x20) == b'h' {
                let d = name[1];
                if d.is_ascii_digit() && d != b'0' {
                    return Some(d - b'0');
                }
            }
            None
        }

        fn is_br(name: &[u8]) -> bool {
            name.len() == 2
                && (name[0] | 0x20) == b'b'
                && (name[1] | 0x20) == b'r'
        }

        fn is_block_tag(name: &[u8]) -> bool {
            let len = name.len();
            if len == 0 { return false; }
            let c0 = name[0] | 0x20;
            if len == 1 { return c0 == b'p'; }
            let c1 = name[1] | 0x20;
            if len == 2 {
                return (c0 == b'l' && c1 == b'i')
                    || (c0 == b'b' && c1 == b'r')
                    || (c0 == b'h' && c1.is_ascii_digit());
            }
            if len == 3 {
                return c0 == b'd' && c1 == b'i' && (name[2] | 0x20) == b'v';
            }
            false
        }

        fn resolve_entity(name: &[u8]) -> Option<&'static str> {
            match name {
                b"amp" => Some("&"),
                b"lt" => Some("<"),
                b"gt" => Some(">"),
                b"quot" => Some("\""),
                b"apos" => Some("'"),
                _ => None,
            }
        }

        /// Advance to the next formatted run.
        ///
        /// Returns `true` if a run is available (callers can then query
        /// `run_style`, `run_heading`, and `run_text`).  Returns `false`
        /// when the stream is exhausted.
        ///
        /// Each call drives the underlying XML parser incrementally — no
        /// per-chapter allocation beyond a single run's text buffer.
        pub fn next_run(&mut self) -> bool {
            use quick_xml::events::Event;

            if self.eof {
                return false;
            }
            self.text.clear();
            self.valid = false;

            loop {
                self.buf.clear();
                let ev = self.reader.read_event_into(&mut self.buf);

                let mut tag: Vec<u8> = Vec::new();
                let mut text: Option<String> = None;
                let mut is_text = false;
                let mut is_end = false;

                match ev {
                    Ok(Event::Start(ref e)) => {
                        tag.extend_from_slice(e.name().as_ref());
                    }
                    Ok(Event::Empty(ref e)) => {
                        tag.extend_from_slice(e.name().as_ref());
                    }
                    Ok(Event::End(ref e)) => {
                        tag.extend_from_slice(e.name().as_ref());
                        is_end = true;
                    }
                    Ok(Event::Text(ref e)) => {
                        is_text = true;
                        if let Ok(s) = core::str::from_utf8(e) {
                            if let Ok(decoded) = quick_xml::escape::unescape(s) {
                                text = Some(String::from(&*decoded));
                            }
                        }
                    }
                    Ok(Event::GeneralRef(ref e)) => {
                        is_text = true;
                        if let Some(ch) = Self::resolve_entity(e.as_ref()) {
                            text = Some(String::from(ch));
                        }
                    }
                    Ok(Event::Eof) => {
                        self.eof = true;
                        return false;
                    }
                    Err(_) => {
                        self.eof = true;
                        return false;
                    }
                    _ => {}
                }

                // Event data extracted; borrow of self.buf is released.
                if is_text {
                    if let Some(t) = text {
                        let s = self.active_style();
                        let h = self.heading_level;
                        self.text = t;
                        self.style = s;
                        self.heading = h;
                        self.valid = true;
                        return true;
                    }
                    continue;
                }

                if tag.is_empty() {
                    continue;
                }

                if !is_end {
                    // Start/Empty events
                    if let Some(s) = Self::tag_is_style_start(&tag) {
                        self.style_stack.push(s);
                    } else if let Some(h) = Self::is_heading_tag(&tag) {
                        self.heading_level = h;
                    } else if Self::is_br(&tag) {
                        let s = self.active_style();
                        let h = self.heading_level;
                        self.text.clear();
                        self.text.push('\n');
                        self.style = s;
                        self.heading = h;
                        self.valid = true;
                        return true;
                    }
                } else {
                    // End events
                    if Self::tag_is_style_start(&tag).is_some() {
                        self.style_stack.pop();
                    } else if Self::is_heading_tag(&tag).is_some() {
                        self.heading_level = 0;
                        self.text.clear();
                        self.text.push('\n');
                        self.style = 0;
                        self.heading = 0;
                        self.valid = true;
                        return true;
                    } else if Self::is_block_tag(&tag) {
                        let s = self.active_style();
                        let h = self.heading_level;
                        self.text.clear();
                        self.text.push('\n');
                        self.style = s;
                        self.heading = h;
                        self.valid = true;
                        return true;
                    }
                }
            }
        }

        /// Style flags bitmask for the current run.
        ///
        /// Must be called after `next_run()` returns `true`.
        /// Bits: 1=bold, 2=italic, 4=underline, 8=strikethrough, 16=code.
        pub fn run_style(&self) -> u8 {
            debug_assert!(self.valid);
            self.style
        }

        /// Heading level for the current run (0 = not a heading, 1-6).
        ///
        /// Must be called after `next_run()` returns `true`.
        pub fn run_heading(&self) -> u8 {
            debug_assert!(self.valid);
            self.heading
        }

        /// Write the text of the current run into `to`.
        ///
        /// Must be called after `next_run()` returns `true`.
        pub fn run_text(&self, to: &mut diplomat_runtime::DiplomatWrite) -> Result<(), ()> {
            use core::fmt::Write;
            debug_assert!(self.valid);
            to.write_str(&self.text).map_err(|_| ())
        }
    }
}
