use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a raw EPUB chapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub href: String,
    pub id: String,
    pub media_type: String,
    pub content: Vec<u8>,
}

impl Chapter {
    /// Create a new Chapter with default media_type
    pub fn new(href: String, id: String, content: Vec<u8>) -> Self {
        Self {
            href,
            id,
            media_type: "application/xhtml+xml".to_string(),
            content,
        }
    }
}

/// AST node representation for parsed HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AstNode {
    Element {
        tag: String,
        attrs: HashMap<String, String>,
        styles: HashMap<String, String>,
        children: Vec<AstNode>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
}

/// Style flags bitmask for a formatting run.
pub type StyleFlags = u8;
pub const STYLE_BOLD: StyleFlags = 1 << 0;
pub const STYLE_ITALIC: StyleFlags = 1 << 1;
pub const STYLE_UNDERLINE: StyleFlags = 1 << 2;
pub const STYLE_STRIKETHROUGH: StyleFlags = 1 << 3;
pub const STYLE_CODE: StyleFlags = 1 << 4;

/// A segment of text with uniform formatting, produced by HTML text extraction.
///
/// Multiple consecutive runs with the same style can be merged by the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingRun {
    pub text: String,
    pub style: StyleFlags,
    /// 0 = not a heading, 1-6 = heading level
    pub heading: u8,
}

/// A parsed EPUB chapter with content and metadata
///
/// Contains the extracted text content, optional AST representation,
/// formatting runs for styled rendering, and statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedChapter {
    /// Raw chapter information (ID, href, media type)
    pub chapter_info: Chapter,
    /// Chapter display title inferred from HTML content when available
    pub title: Option<String>,
    /// Extracted text content
    pub content: String,
    /// Optional HTML AST representation
    pub ast: Option<AstNode>,
    /// Styled text segments with heading/style info
    pub formatting_runs: Vec<FormattingRun>,
    /// Word count in the content
    pub word_count: usize,
    /// Character count in the content
    pub char_count: usize,
}

/// Chapter stream for async iteration
pub struct ChapterStream {
    extractor: crate::core::extractor::EpubExtractor,
    entries: Vec<String>,
    index: usize,
    /// in-flight future for the currently reading/parsing chapter
    inflight: Option<
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<ParsedChapter>> + 'static>>,
    >,
}

impl ChapterStream {
    /// Create a streaming chapter stream backed by an `EpubExtractor` and a
    /// list of resolved entry paths (relative paths inside the EPUB).
    pub fn from_extractor(
        extractor: crate::core::extractor::EpubExtractor,
        entries: Vec<String>,
    ) -> Self {
        Self {
            extractor,
            entries,
            index: 0,
            inflight: None,
        }
    }
}

impl futures::Stream for ChapterStream {
    type Item = Result<ParsedChapter>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // If no in-flight future, create one for the next chapter (if any)
        if self.inflight.is_none() {
            if self.index >= self.entries.len() {
                return std::task::Poll::Ready(None);
            }

            let path = self.entries[self.index].clone();
            let ex = self.extractor.clone();

            // create a future that reads & parses a single chapter
            let fut = async move {
                // read file bytes from the archive
                let content = ex.read_file(&path).await?;

                // parse html -> plain text + formatting runs
                let html_content = String::from_utf8_lossy(&content);
                let text_content = crate::core::html_parser::extract_text_content(&html_content)?;
                let formatting_runs = crate::core::html_parser::extract_formatting(&html_content)?;

                let word_count = text_content.split_whitespace().count();
                let char_count = text_content.chars().count();

                let chapter = crate::core::chapter::Chapter {
                    href: path.clone(),
                    id: String::new(),
                    media_type: "application/xhtml+xml".to_string(),
                    content: content.clone(),
                };

                Ok(crate::core::chapter::ParsedChapter {
                    chapter_info: chapter,
                    title: None,
                    content: text_content,
                    ast: None,
                    formatting_runs,
                    word_count,
                    char_count,
                })
            };

            self.inflight = Some(Box::pin(fut));
        }

        // Poll the in-flight future
        if let Some(fut) = self.inflight.as_mut() {
            match fut.as_mut().poll(cx) {
                std::task::Poll::Ready(Ok(parsed)) => {
                    // consume the future and advance index
                    self.inflight = None;
                    self.index += 1;
                    return std::task::Poll::Ready(Some(Ok(parsed)));
                }
                std::task::Poll::Ready(Err(e)) => {
                    self.inflight = None;
                    self.index += 1; // skip this entry on error
                    return std::task::Poll::Ready(Some(Err(e)));
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }

        std::task::Poll::Pending
    }
}
