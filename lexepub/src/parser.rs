use crate::error::Result;
use scraper::Html;
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

/// Parsed chapter with metadata and content
#[derive(Debug, Clone)]
pub struct ParsedChapter {
    pub chapter_info: Chapter,
    pub content: String,
    pub ast: Option<AstNode>,
    pub word_count: usize,
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

/// Clean chapter parser with configurable strategies
#[derive(Clone)]
pub struct ChapterParser {
    pub text_only: bool,
    pub with_ast: bool,
}

impl Default for ChapterParser {
    fn default() -> Self {
        Self {
            text_only: true,
            with_ast: false,
        }
    }
}

impl ChapterParser {
    /// Create a new chapter parser with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse chapter text only (fastest)
    pub fn text_only(mut self) -> Self {
        self.text_only = true;
        self.with_ast = false;
        self
    }

    /// Include AST generation
    pub fn with_ast(mut self) -> Self {
        self.text_only = false;
        self.with_ast = true;
        self
    }

    /// Both text and AST
    pub fn with_both(mut self) -> Self {
        self.text_only = false;
        self.with_ast = true;
        self
    }

    /// Parse a chapter into the requested format
    pub fn parse_chapter(&self, chapter: Chapter) -> Result<ParsedChapter> {
        let content_str = std::str::from_utf8(&chapter.content)?;

        let ast = if self.with_ast {
            Some(parse_html_ast(content_str)?)
        } else {
            None
        };

        let content = if !self.text_only && !self.with_ast {
            content_str.to_string()
        } else {
            extract_text_content(content_str)?
        };

        let word_count = content.split_whitespace().count();
        let char_count = content.chars().count();

        Ok(ParsedChapter {
            chapter_info: chapter,
            content,
            ast,
            word_count,
            char_count,
        })
    }
}

/// Extract clean text content from HTML using scraper
fn extract_text_content(html: &str) -> Result<String> {
    let fragment = Html::parse_fragment(html);

    let mut text = String::new();

    // Extract text from body content
    for element in fragment.root_element().descendants() {
        match element.value() {
            scraper::Node::Text(text_node) => {
                text.push_str(text_node);
            }
            scraper::Node::Element(element_ref) => {
                // Add newlines after block elements
                if matches!(
                    element_ref.name(),
                    "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "br" | "li"
                ) {
                    text.push('\n');
                }
            }
            _ => {}
        }
    }

    // Clean up excess whitespace and newlines
    let cleaned = text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(cleaned)
}

/// Parse HTML into AST structure using scraper
fn parse_html_ast(html: &str) -> Result<AstNode> {
    let fragment = Html::parse_fragment(html);

    // Convert scraper tree to our AST format
    Ok(element_to_ast(&fragment.root_element()))
}

/// Convert scraper element to our AST format
fn element_to_ast(element: &scraper::ElementRef) -> AstNode {
    let mut attrs = HashMap::new();

    // Get attributes from the element
    for attr in element.value().attrs() {
        attrs.insert(attr.0.to_string(), attr.1.to_string());
    }

    let children: Vec<AstNode> = element
        .children()
        .map(|child| match child.value() {
            scraper::Node::Text(text_node) => AstNode::Text {
                content: text_node.to_string(),
            },
            scraper::Node::Comment(comment_node) => AstNode::Comment {
                content: comment_node.to_string(),
            },
            scraper::Node::Element(_) => element_to_ast(&scraper::ElementRef::wrap(child).unwrap()),
            _ => AstNode::Text {
                content: String::new(),
            },
        })
        .collect();

    AstNode::Element {
        tag: element.value().name().to_string(),
        attrs,
        children,
    }
}
