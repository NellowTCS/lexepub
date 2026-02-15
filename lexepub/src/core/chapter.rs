use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a raw EPUB chapter
#[derive(Debug, Clone)]
pub struct Chapter {
    pub href: String,
    pub id: String,
    pub media_type: String,
    pub content: Vec<u8>,
}

/// AST node representation for parsed HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AstNode {
    Element {
        tag: String,
        attrs: HashMap<String, String>,
        children: Vec<AstNode>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
}

/// A parsed EPUB chapter with content and metadata
///
/// Contains the extracted text content, optional AST representation,
/// and statistics about the chapter.
#[derive(Debug, Clone)]
pub struct ParsedChapter {
    /// Raw chapter information (ID, href, media type)
    pub chapter_info: Chapter,
    /// Extracted text content
    pub content: String,
    /// Optional HTML AST representation
    pub ast: Option<AstNode>,
    /// Word count in the content
    pub word_count: usize,
    /// Character count in the content
    pub char_count: usize,
}

/// Chapter stream for async iteration
pub struct ChapterStream {
    chapters: Vec<ParsedChapter>,
    index: usize,
}

impl ChapterStream {
    pub fn new(chapters: Vec<ParsedChapter>) -> Self {
        Self { chapters, index: 0 }
    }
}

impl futures::Stream for ChapterStream {
    type Item = Result<ParsedChapter>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.index < self.chapters.len() {
            let chapter = self.chapters[self.index].clone();
            self.index += 1;
            std::task::Poll::Ready(Some(Ok(chapter)))
        } else {
            std::task::Poll::Ready(None)
        }
    }
}
