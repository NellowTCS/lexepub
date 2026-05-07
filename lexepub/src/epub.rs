use crate::core::chapter::{Chapter, ChapterStream, ParsedChapter};
use crate::core::container::ContainerParser;
use crate::core::extractor::EpubExtractor;
use crate::core::opf_parser::OpfParser;
use crate::error::{LexEpubError, Result};
use bytes::Bytes;
use std::path::Path;

/// Main EPUB processing struct
pub struct LexEpub {
    extractor: EpubExtractor,
    metadata: Option<EpubMetadata>,
    /// Cached full (AST + text) chapter extraction
    chapters: Option<Vec<ParsedChapter>>,
    /// Cached text-only extraction (cheaper than full AST parse)
    text_chapters: Option<Vec<String>>,
    /// Cached aggregate word count — avoids re-extracting just for counts
    cached_word_count: Option<usize>,
    /// Cached aggregate char count
    cached_char_count: Option<usize>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub version: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub languages: Vec<String>,
    pub subjects: Vec<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<String>,
    pub identifiers: Vec<String>,
    pub rights: Option<String>,
    pub contributors: Vec<String>,
    pub spine: Vec<String>,
    pub has_cover: bool,
    pub cover_image_format: Option<String>,
    pub chapter_count: usize,
}

/// Convert OPF metadata to EPUB metadata
impl From<crate::core::opf_parser::OpfMetadata> for EpubMetadata {
    fn from(opf: crate::core::opf_parser::OpfMetadata) -> Self {
        let cover_image_format = opf
            .cover_image_id
            .as_ref()
            .and_then(|id| opf.manifest.get(id).map(|(_, mime)| mime.clone()));
        Self {
            title: opf.title,
            version: opf.version,
            authors: opf.creators,
            description: opf.description,
            languages: opf.languages,
            subjects: opf.subjects,
            publisher: opf.publisher,
            publication_date: opf.date,
            identifiers: opf.identifiers,
            rights: opf.rights,
            contributors: opf.contributors,
            spine: opf.spine.clone(),
            has_cover: opf.cover_image_id.is_some(),
            cover_image_format,
            chapter_count: opf.spine.len(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TocEntry {
    pub chapter_index: usize,
    pub chapter_id: String,
    pub chapter_href: String,
    pub title: String,
}

impl EpubMetadata {
    /// Validates the metadata per EPUB standards (requires title, language, and identifier)
    pub fn validate(&self) -> std::result::Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.title.as_ref().is_none_or(|t| t.trim().is_empty()) {
            errors.push("Missing required field: title".to_string());
        }
        if self.languages.is_empty() {
            errors.push("Missing required field: language".to_string());
        }
        if self.identifiers.is_empty() {
            errors.push("Missing required field: identifier".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl LexEpub {
    pub async fn get_toc(&mut self) -> Result<Vec<TocEntry>> {
        // TOC only needs chapter hrefs/ids/titles, not full AST.
        // Use text-only path if AST hasn't been computed yet.
        if let Some(ref chapters) = self.chapters {
            return Ok(Self::toc_from_parsed(chapters));
        }
        // Fall back to text-only extraction which is cheaper
        let chapters = self.extract_chapters_text_only_internal().await?;
        Ok(Self::toc_from_parsed(&chapters))
    }

    fn toc_from_parsed(chapters: &[ParsedChapter]) -> Vec<TocEntry> {
        chapters
            .iter()
            .enumerate()
            .map(|(index, chapter)| TocEntry {
                chapter_index: index,
                chapter_id: chapter.chapter_info.id.clone(),
                chapter_href: chapter.chapter_info.href.clone(),
                title: chapter
                    .title
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| {
                        std::path::Path::new(&chapter.chapter_info.href)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled Chapter")
                            .to_string()
                    }),
            })
            .collect()
    }

    pub async fn read_resource(&self, path: &str) -> Result<Vec<u8>> {
        self.extractor.read_file(path).await
    }

    pub async fn resolve_chapter_resource_path(
        &mut self,
        chapter_index: usize,
        href: &str,
    ) -> Result<String> {
        let href_clean = href.trim();
        if href_clean.is_empty()
            || href_clean.starts_with('#')
            || href_clean.starts_with("http://")
            || href_clean.starts_with("https://")
            || href_clean.starts_with("mailto:")
            || href_clean.starts_with("data:")
            || href_clean.starts_with("blob:")
        {
            return Ok(href_clean.to_string());
        }

        let path_only = href_clean.split('#').next().unwrap_or(href_clean);
        if !path_only.is_empty() && self.extractor.read_file(path_only).await.is_ok() {
            return Ok(href_clean.replace('\\', "/"));
        }

        let normalized_candidate = normalize_internal_path(path_only);
        if !normalized_candidate.is_empty()
            && self
                .extractor
                .read_file(&normalized_candidate)
                .await
                .is_ok()
        {
            let mut out = normalized_candidate;
            if let Some(fragment) = href_clean.split('#').nth(1) {
                out.push('#');
                out.push_str(fragment);
            }
            return Ok(out);
        }

        // Use text-only chapters to avoid triggering full AST parse just for path resolution
        let chapters = self.extract_chapters_text_only_internal().await?;
        let chapter = chapters
            .get(chapter_index)
            .ok_or_else(|| LexEpubError::ChapterError("Chapter index out of bounds".to_string()))?;
        Ok(resolve_href_against(&chapter.chapter_info.href, href))
    }

    pub async fn read_chapter_resource(
        &mut self,
        chapter_index: usize,
        href: &str,
    ) -> Result<Vec<u8>> {
        let href_clean = href.trim();
        if !href_clean.is_empty() {
            let direct_path = href_clean.split('#').next().unwrap_or(href_clean);
            if !direct_path.is_empty() {
                if let Ok(bytes) = self.extractor.read_file(direct_path).await {
                    return Ok(bytes);
                }

                let normalized_direct = normalize_internal_path(direct_path);
                if !normalized_direct.is_empty() {
                    if let Ok(bytes) = self.extractor.read_file(&normalized_direct).await {
                        return Ok(bytes);
                    }
                }
            }
        }

        let resolved = self
            .resolve_chapter_resource_path(chapter_index, href)
            .await?;
        let resolved_path = resolved.split('#').next().unwrap_or(&resolved);
        self.extractor.read_file(resolved_path).await
    }

    /// Open an EPUB from a file path
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let extractor = EpubExtractor::open(path.as_ref().to_path_buf()).await?;
        Ok(Self::with_extractor(extractor))
    }

    /// Synchronous wrapper for `open` (used by FFI and sync callers)
    pub fn open_sync<P: AsRef<Path>>(path: P) -> Result<Self> {
        futures::executor::block_on(LexEpub::open(path))
    }

    /// Create an EPUB from bytes
    pub async fn from_bytes(data: Bytes) -> Result<Self> {
        let extractor = EpubExtractor::from_bytes(data).await?;
        Ok(Self::with_extractor(extractor))
    }

    /// Create an EPUB from an async reader (streaming, does not copy the whole
    /// archive into memory). Useful for SD/LittleFS/flash-backed readers.
    pub async fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: futures::AsyncBufRead + futures::AsyncSeek + Unpin + Send + 'static,
    {
        let extractor = EpubExtractor::from_reader(reader)?;
        Ok(Self::with_extractor(extractor))
    }

    /// Create an EPUB from a blocking reader by wrapping it with
    /// `futures::io::AllowStdIo` (convenience for platforms with sync FS APIs).
    pub fn from_sync_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read + std::io::Seek + Send + 'static,
    {
        let extractor = EpubExtractor::from_sync_reader(reader)?;
        Ok(Self::with_extractor(extractor))
    }

    /// Create a LexEpub with the given extractor and default cache state.
    /// This is the SSOT for constructing LexEpub from an existing extractor.
    fn with_extractor(extractor: EpubExtractor) -> Self {
        Self {
            extractor,
            metadata: None,
            chapters: None,
            text_chapters: None,
            cached_word_count: None,
            cached_char_count: None,
        }
    }

    /// Extract only text content from all chapters.
    ///
    /// Uses a cheaper text-only parsing path (no CSS, no AST) when possible.
    pub async fn extract_text_only(&mut self) -> Result<Vec<String>> {
        // If we already have the full AST parse, derive text from it (free)
        if let Some(ref chapters) = self.chapters {
            return Ok(chapters.iter().map(|c| c.content.clone()).collect());
        }
        // Use (or populate) the cheaper text-only cache
        if self.text_chapters.is_none() {
            let parsed = self.extract_chapters_text_only_internal().await?;
            let texts: Vec<String> = parsed.iter().map(|c| c.content.clone()).collect();

            // Cache aggregate counts while we have the data
            if self.cached_word_count.is_none() {
                self.cached_word_count = Some(parsed.iter().map(|c| c.word_count).sum());
            }
            if self.cached_char_count.is_none() {
                self.cached_char_count = Some(parsed.iter().map(|c| c.char_count).sum());
            }

            self.text_chapters = Some(texts);
        }
        Ok(self.text_chapters.clone().unwrap())
    }

    /// Extract chapters with AST for advanced processing
    pub async fn extract_ast(&mut self) -> Result<Vec<ParsedChapter>> {
        self.extract_chapters().await
    }

    /// Extract chapters as a stream for memory-efficient processing
    pub async fn extract_chapters_stream(&mut self) -> Result<ChapterStream> {
        let (opf_path, opf_data) = self.read_opf().await?;
        let mut opf_parser = OpfParser::new();
        // parse_metadata() already populates spine
        let metadata = opf_parser.parse_metadata(&opf_data)?;
        let spine = metadata.spine.clone();

        // Resolve full paths for spine entries and return a streaming iterator
        let mut entries = Vec::new();
        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));

        for item_id in spine {
            if let Some(href) = metadata.manifest.get(&item_id) {
                let full_path = opf_base.join(&href.0);
                let full_path_str = full_path.to_string_lossy().to_string();
                entries.push(full_path_str);
            }
        }

        Ok(ChapterStream::from_extractor(
            self.extractor.clone(),
            entries,
        ))
    }

    /// Get metadata
    pub fn get_metadata_sync(&mut self) -> Result<EpubMetadata> {
        futures::executor::block_on(self.get_metadata())
    }

    pub async fn get_metadata(&mut self) -> Result<EpubMetadata> {
        if let Some(ref metadata) = self.metadata {
            return Ok(metadata.clone());
        }

        // Get OPF location
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;

        // Parse OPF metadata
        let opf_data = self.extractor.read_file(&opf_path).await?;
        let mut opf_parser = OpfParser::new();
        let opf_metadata = opf_parser.parse_metadata(&opf_data)?;

        let epub_metadata: EpubMetadata = opf_metadata.into();
        self.metadata = Some(epub_metadata.clone());
        Ok(epub_metadata)
    }

    /// Validates the metadata against basic EPUB standard requirements
    pub async fn validate_metadata(&mut self) -> Result<()> {
        let metadata = self.get_metadata().await?;
        metadata
            .validate()
            .map_err(crate::error::LexEpubError::ValidationError)
    }

    pub fn validate_metadata_sync(&mut self) -> Result<()> {
        futures::executor::block_on(self.validate_metadata())
    }

    /// Get total word count across all chapters.
    ///
    /// Shares the extraction cache with `total_char_count` — calling both in
    /// sequence only parses the EPUB once.
    pub async fn total_word_count(&mut self) -> Result<usize> {
        if let Some(count) = self.cached_word_count {
            return Ok(count);
        }
        // Populate both caches in one pass
        self.populate_count_cache().await?;
        Ok(self.cached_word_count.unwrap())
    }

    /// Synchronous wrapper for `total_word_count`
    pub fn total_word_count_sync(&mut self) -> Result<usize> {
        futures::executor::block_on(self.total_word_count())
    }

    /// Get total character count across all chapters.
    ///
    /// Shares the extraction cache with `total_word_count` — calling both in
    /// sequence only parses the EPUB once.
    pub async fn total_char_count(&mut self) -> Result<usize> {
        if let Some(count) = self.cached_char_count {
            return Ok(count);
        }
        self.populate_count_cache().await?;
        Ok(self.cached_char_count.unwrap())
    }

    /// Synchronous wrapper for `total_char_count`
    pub fn total_char_count_sync(&mut self) -> Result<usize> {
        futures::executor::block_on(self.total_char_count())
    }

    /// Internal: populate word + char count caches in one pass, reusing
    /// whichever chapter cache is already warm.
    async fn populate_count_cache(&mut self) -> Result<()> {
        // If we have the full AST cache, derive counts from it (free)
        if let Some(ref chapters) = self.chapters {
            self.cached_word_count = Some(chapters.iter().map(|c| c.word_count).sum());
            self.cached_char_count = Some(chapters.iter().map(|c| c.char_count).sum());
            return Ok(());
        }
        // Use the cheaper text-only path if the text cache is warm
        if let Some(ref texts) = self.text_chapters {
            let (words, chars) = texts.iter().fold((0usize, 0usize), |(w, c), t| {
                (w + t.split_whitespace().count(), c + t.chars().count())
            });
            self.cached_word_count = Some(words);
            self.cached_char_count = Some(chars);
            return Ok(());
        }
        // Cold path: run text-only extraction and cache everything
        let parsed = self.extract_chapters_text_only_internal().await?;
        let mut words = 0usize;
        let mut chars = 0usize;
        let mut texts = Vec::with_capacity(parsed.len());
        for c in &parsed {
            words += c.word_count;
            chars += c.char_count;
            texts.push(c.content.clone());
        }
        self.cached_word_count = Some(words);
        self.cached_char_count = Some(chars);
        self.text_chapters = Some(texts);
        Ok(())
    }

    /// Check if the EPUB has a cover image
    pub fn has_cover_sync(&mut self) -> Result<bool> {
        futures::executor::block_on(self.has_cover())
    }

    pub async fn has_cover(&mut self) -> Result<bool> {
        // Reuse metadata cache if available — avoids re-reading container/OPF
        if let Some(ref meta) = self.metadata {
            return Ok(meta.has_cover);
        }

        // Use read_opf() SSOT for container.xml parsing
        let (_opf_path, opf_data) = self.read_opf().await?;
        let mut opf_parser = OpfParser::new();
        let cover_id = opf_parser.get_cover_image_id(&opf_data)?;

        Ok(cover_id.is_some())
    }

    /// Extract the cover image bytes from the EPUB
    pub fn cover_image_sync(&mut self) -> Result<Vec<u8>> {
        futures::executor::block_on(self.cover_image())
    }

    pub async fn cover_image(&mut self) -> Result<Vec<u8>> {
        // Use read_opf() SSOT for container.xml parsing
        let (opf_path, opf_data) = self.read_opf().await?;
        let mut opf_parser = OpfParser::new();
        let metadata = opf_parser.parse_metadata(&opf_data)?;

        let cover_id = metadata
            .cover_image_id
            .ok_or_else(|| LexEpubError::MissingFile("No cover image found in EPUB".to_string()))?;

        let cover_href = metadata.manifest.get(&cover_id).ok_or_else(|| {
            LexEpubError::MissingFile(format!("Cover image item '{}' not in manifest", cover_id))
        })?;

        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let full_path = opf_base.join(&cover_href.0);
        let full_path_str = full_path.to_string_lossy();

        self.extractor.read_file(&full_path_str).await
    }

    /// Stream the cover image bytes directly to a given parameter implementing futures::AsyncWrite.
    pub async fn cover_image_to_writer<W: futures::AsyncWrite + Unpin + Send>(
        &mut self,
        writer: &mut W,
    ) -> Result<u64> {
        // Use read_opf() SSOT for container.xml parsing
        let (opf_path, opf_data) = self.read_opf().await?;
        let mut opf_parser = OpfParser::new();
        let metadata = opf_parser.parse_metadata(&opf_data)?;

        let cover_id = metadata
            .cover_image_id
            .ok_or_else(|| LexEpubError::MissingFile("No cover image found in EPUB".to_string()))?;

        let cover_href = metadata.manifest.get(&cover_id).ok_or_else(|| {
            LexEpubError::MissingFile(format!("Cover image item '{}' not in manifest", cover_id))
        })?;

        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        let full_path = opf_base.join(&cover_href.0);
        let full_path_str = full_path.to_string_lossy();

        self.extractor
            .read_file_to_writer(&full_path_str, writer)
            .await
    }

    pub async fn extract_with_ast(&mut self) -> Result<Vec<ParsedChapter>> {
        self.extract_ast().await
    }

    // -----------------------------------------------------------------------
    // Internal extraction helpers
    // -----------------------------------------------------------------------

    /// Text-only chapter extraction (no CSS parsing, no AST).
    /// Results are stored in `self.text_chapters` but NOT in `self.chapters`.
    async fn extract_chapters_text_only_internal(&mut self) -> Result<Vec<ParsedChapter>> {
        // Read OPF once
        let (opf_path, opf_data) = self.read_opf().await?;
        let mut opf_parser = OpfParser::new();
        // parse_metadata() already populates spine, no need for separate parse_spine()
        let metadata = opf_parser.parse_metadata(&opf_data)?;
        let spine = metadata.spine.clone();

        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_path_buf();

        let mut chapters = Vec::new();
        for item_id in spine {
            if let Some(href) = metadata.manifest.get(&item_id) {
                let full_path = opf_base.join(&href.0);
                let full_path_str = full_path.to_string_lossy();
                match self.extractor.read_file(&full_path_str).await {
                    Ok(content) => {
                        let chapter =
                            Chapter::new(full_path_str.to_string(), item_id.clone(), content);
                        // Text-only parse: no AST, no CSS
                        let parser = crate::core::html_parser::ChapterParser::new().text_only();
                        match parser.parse_chapter(chapter) {
                            Ok(parsed) => chapters.push(parsed),
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        Ok(chapters)
    }

    /// Full AST+CSS chapter extraction. Results cached in `self.chapters`.
    async fn extract_chapters(&mut self) -> Result<Vec<ParsedChapter>> {
        if let Some(ref chapters) = self.chapters {
            return Ok(chapters.clone());
        }

        let (opf_path, opf_data) = self.read_opf().await?;
        let mut opf_parser = OpfParser::new();
        // parse_metadata() already populates spine, no need for separate parse_spine()
        let metadata = opf_parser.parse_metadata(&opf_data)?;
        let spine = metadata.spine.clone();

        let opf_base = std::path::Path::new(&opf_path)
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_path_buf();

        // Parse all CSS once
        let mut css_text = String::new();
        for (href, media_type) in metadata.manifest.values() {
            if media_type == "text/css" {
                let css_path = opf_base.join(href);
                let css_path_str = css_path.to_string_lossy();
                if let Ok(css_data) = self.extractor.read_file(&css_path_str).await {
                    css_text.push_str(&String::from_utf8_lossy(&css_data));
                    css_text.push('\n');
                }
            }
        }
        let stylesheet = crate::core::css::Stylesheet::parse(&css_text);

        let mut chapters = Vec::new();
        let parser = crate::core::html_parser::ChapterParser::new();
        for item_id in spine {
            if let Some(href) = metadata.manifest.get(&item_id) {
                let full_path = opf_base.join(&href.0);
                let full_path_str = full_path.to_string_lossy();
                match self.extractor.read_file(&full_path_str).await {
                    Ok(content) => {
                        let chapter =
                            Chapter::new(full_path_str.to_string(), item_id.clone(), content);
                        let mut parsed_chapter = match parser.parse_chapter(chapter) {
                            Ok(p) => p,
                            Err(_) => continue,
                        };

                        if let Some(ref mut ast) = parsed_chapter.ast {
                            normalize_ast_links(ast, &full_path_str);
                            stylesheet.apply_to_ast(ast);
                        }

                        chapters.push(parsed_chapter);
                    }
                    Err(_) => continue,
                }
            }
        }

        // Populate derived caches from the now-available full parse
        if self.cached_word_count.is_none() {
            self.cached_word_count = Some(chapters.iter().map(|c| c.word_count).sum());
        }
        if self.cached_char_count.is_none() {
            self.cached_char_count = Some(chapters.iter().map(|c| c.char_count).sum());
        }
        // Also populate text cache so extract_text_only() is free after this
        if self.text_chapters.is_none() {
            self.text_chapters = Some(chapters.iter().map(|c| c.content.clone()).collect());
        }

        #[cfg(not(feature = "lowmem"))]
        {
            self.chapters = Some(chapters.clone());
        }

        Ok(chapters)
    }

    /// Read and return (opf_path, opf_data), reusing the metadata cache's
    /// knowledge of opf_path when available to avoid re-reading container.xml.
    async fn read_opf(&mut self) -> Result<(String, Vec<u8>)> {
        let container_data = self.extractor.read_file("META-INF/container.xml").await?;
        let mut container_parser = ContainerParser::new();
        let opf_path = container_parser
            .parse_container(&container_data)?
            .rootfile_path;
        let opf_data = self.extractor.read_file(&opf_path).await?;
        Ok((opf_path, opf_data))
    }
}

fn resolve_href_against(base_path: &str, href: &str) -> String {
    if href.trim().is_empty() {
        return base_path.to_string();
    }

    if href.starts_with('#')
        || href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with("mailto:")
        || href.starts_with("data:")
        || href.starts_with("blob:")
    {
        return href.to_string();
    }

    let (path_part, fragment_part) = match href.split_once('#') {
        Some((path, frag)) => (path, Some(frag)),
        None => (href, None),
    };

    let mut joined = if path_part.starts_with('/') {
        std::path::PathBuf::from(path_part.trim_start_matches('/'))
    } else {
        let base_dir = std::path::Path::new(base_path)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        base_dir.join(path_part)
    };

    if path_part.is_empty() {
        joined = std::path::PathBuf::from(base_path);
    }

    let mut normalized = normalize_internal_path(&joined.to_string_lossy());
    if let Some(fragment) = fragment_part {
        normalized.push('#');
        normalized.push_str(fragment);
    }

    normalized
}

fn normalize_internal_path(path: &str) -> String {
    let mut parts = Vec::new();
    let replaced = path.replace('\\', "/");
    for segment in replaced.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(segment),
        }
    }
    parts.join("/")
}

fn normalize_ast_links(ast: &mut crate::core::chapter::AstNode, chapter_href: &str) {
    use crate::core::chapter::AstNode;

    if let AstNode::Element {
        attrs, children, ..
    } = ast
    {
        if let Some(href) = attrs.get_mut("href") {
            let resolved = resolve_href_against(chapter_href, href);
            *href = resolved;
        }

        if let Some(src) = attrs.get_mut("src") {
            let resolved = resolve_href_against(chapter_href, src);
            *src = resolved;
        }

        for child in children {
            normalize_ast_links(child, chapter_href);
        }
    }
}

/// Convenience function for quick text extraction
pub async fn extract_text_only<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut epub = LexEpub::open(path).await?;
    epub.extract_text_only().await
}

/// Convenience function for AST extraction
pub async fn extract_ast<P: AsRef<Path>>(path: P) -> Result<Vec<ParsedChapter>> {
    let mut epub = LexEpub::open(path).await?;
    epub.extract_ast().await
}

/// Convenience function for metadata extraction
pub async fn get_metadata<P: AsRef<Path>>(path: P) -> Result<EpubMetadata> {
    let mut epub = LexEpub::open(path).await?;
    epub.get_metadata().await
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnalysisReport {
    pub metadata: Option<EpubMetadata>,
    pub chapter_count: usize,
    pub total_words: usize,
    pub total_chars: usize,
    pub first_chapter_preview: Option<String>,
}

/// Analyze an EPUB from an async reader (streaming, does not require full-copy).
pub async fn analyze_reader<R>(reader: R) -> Result<AnalysisReport>
where
    R: futures::AsyncBufRead + futures::AsyncSeek + Unpin + Send + 'static,
{
    let extractor = EpubExtractor::from_reader(reader)?;
    analyze_with_extractor(extractor).await
}

/// Analyze from a blocking reader
pub fn analyze_sync_reader<R>(reader: R) -> Result<AnalysisReport>
where
    R: std::io::Read + std::io::Seek + Send + 'static,
{
    let extractor = EpubExtractor::from_sync_reader(reader)?;
    futures::executor::block_on(analyze_with_extractor(extractor))
}

/// Analyze from file path
pub async fn analyze_path<P: AsRef<Path>>(path: P) -> Result<AnalysisReport> {
    let extractor = EpubExtractor::open(path.as_ref().to_path_buf()).await?;
    analyze_with_extractor(extractor).await
}

/// Core analysis logic that reuses EpubExtractor
async fn analyze_with_extractor(extractor: EpubExtractor) -> Result<AnalysisReport> {
    use crate::core::html_parser::extract_text_content;

    // Read OPF using the extractor (same pattern as read_opf but without &mut self)
    let container_data = extractor.read_file("META-INF/container.xml").await?;
    let mut container_parser = ContainerParser::new();
    let opf_path = container_parser
        .parse_container(&container_data)?
        .rootfile_path;

    let opf_data = extractor.read_file(&opf_path).await?;
    let mut opf_parser = OpfParser::new();
    // parse_metadata() already populates spine, no need for separate parse_spine()
    let metadata = opf_parser.parse_metadata(&opf_data)?;
    let spine = metadata.spine.clone();

    let mut chapters_parsed = Vec::new();
    let opf_base = std::path::Path::new(&opf_path)
        .parent()
        .unwrap_or(std::path::Path::new(""));

    for item_id in spine {
        if let Some(href) = metadata.manifest.get(&item_id) {
            let full_path = opf_base.join(&href.0);
            let full_path_str = full_path.to_string_lossy().to_string();
            if let Ok(content) = extractor.read_file(&full_path_str).await {
                let html_content = String::from_utf8_lossy(&content);
                let text_content = extract_text_content(&html_content)?;
                let word_count = text_content.split_whitespace().count();
                let char_count = text_content.chars().count();

                chapters_parsed.push((text_content, word_count, char_count));
            }
        }
    }

    let chapter_count = chapters_parsed.len();
    let total_words: usize = chapters_parsed.iter().map(|(_, w, _)| *w).sum();
    let total_chars: usize = chapters_parsed.iter().map(|(_, _, c)| *c).sum();
    let first_chapter_preview = chapters_parsed
        .first()
        .map(|(s, _, _)| s.chars().take(300).collect::<String>());

    let epub_metadata: EpubMetadata = metadata.into();

    Ok(AnalysisReport {
        metadata: Some(epub_metadata),
        chapter_count,
        total_words,
        total_chars,
        first_chapter_preview,
    })
}
