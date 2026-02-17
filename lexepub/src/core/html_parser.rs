use crate::core::chapter::{AstNode, Chapter, ParsedChapter};
use crate::error::Result;
use scraper::Html;
use std::collections::HashMap;

/// Configurable chapter parser
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

#[cfg(not(feature = "lowmem"))]
/// Extract clean text content from HTML using scraper
pub fn extract_text_content(html: &str) -> Result<String> {
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

#[cfg(feature = "lowmem")]
/// Lightweight HTML-to-text extractor for low-memory targets.
// Not as robust as the scraper-based version, but avoids the overhead of building a full DOM tree, haha.
pub fn extract_text_content(html: &str) -> Result<String> {
    let mut out = String::new();
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut last_was_space = false;

    for c in html.chars() {
        if in_tag {
            if c == '>' {
                in_tag = false;
                let tag = tag_buf.trim().trim_start_matches('/').to_ascii_lowercase();
                if tag.starts_with('p')
                    || tag.starts_with("div")
                    || tag.starts_with("br")
                    || tag.starts_with('h')
                    || tag.starts_with("li")
                {
                    out.push('\n');
                }
                tag_buf.clear();
            } else {
                tag_buf.push(c);
            }
        } else if c == '<' {
            in_tag = true;
            tag_buf.clear();
        } else {
            if c.is_whitespace() {
                if !last_was_space {
                    out.push(' ');
                    last_was_space = true;
                }
            } else {
                out.push(c);
                last_was_space = false;
            }
        }
    }

    let cleaned = out
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
        .filter_map(|child| match child.value() {
            scraper::Node::Text(text_node) => Some(AstNode::Text {
                content: text_node.to_string(),
            }),
            scraper::Node::Comment(comment_node) => Some(AstNode::Comment {
                content: comment_node.to_string(),
            }),
            scraper::Node::Element(_) => {
                scraper::ElementRef::wrap(child).map(|elem| element_to_ast(&elem))
            }
            _ => Some(AstNode::Text {
                content: String::new(),
            }),
        })
        .collect();

    AstNode::Element {
        tag: element.value().name().to_string(),
        attrs,
        children,
    }
}
