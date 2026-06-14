use crate::core::chapter::{AstNode, Chapter, ParsedChapter};
use crate::error::{LexEpubError, Result};
use std::collections::HashMap;
use tl::ParserOptions;

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

        let title = ast.as_ref().and_then(extract_title_from_ast).or_else(|| {
            content
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(|s| s.trim().to_string())
        });

        Ok(ParsedChapter {
            chapter_info: chapter,
            title,
            content,
            ast,
            word_count,
            char_count,
        })
    }
}

// extract_text_content
#[cfg(not(feature = "lowmem"))]
/// Extract clean text content from HTML using tl (full DOM).
pub fn extract_text_content(html: &str) -> Result<String> {
    let dom = tl::parse(html, ParserOptions::default())
        .map_err(|e| LexEpubError::Html(format!("Failed to parse HTML: {}", e)))?;

    let parser = dom.parser();
    let mut text = String::new();

    // Extract text from top-level children
    for handle in dom.children() {
        extract_text_recursive(*handle, parser, &mut text);
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

#[cfg(not(feature = "lowmem"))]
fn extract_text_recursive(handle: tl::NodeHandle, parser: &tl::Parser, output: &mut String) {
    if let Some(node) = handle.get(parser) {
        match node {
            tl::Node::Raw(text_bytes) => {
                let text_str = text_bytes.as_utf8_str();
                let decoded = html_escape::decode_html_entities(&text_str);
                output.push_str(&decoded);
            }
            tl::Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str();
                let is_block = matches!(
                    tag_name.as_ref(),
                    "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "br" | "li"
                );

                // Recursively process children
                for child_handle in tag.children().top().iter() {
                    extract_text_recursive(*child_handle, parser, output);
                }

                // Add newlines after block elements
                if is_block {
                    output.push('\n');
                }
            }
            tl::Node::Comment(_) => {}
        }
    }
}

/// Lightweight streaming HTML-to-text extractor.
///
/// Processes byte chunks incrementally without holding the full HTML input
/// in memory
pub struct StreamingTextExtractor {
    pub in_tag: bool,
    pub last_was_space: bool,
    pub output: String,
    tag_buf: String,
}

impl Default for StreamingTextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingTextExtractor {
    pub fn new() -> Self {
        Self::with_output(String::new())
    }

    pub fn with_output(output: String) -> Self {
        Self {
            in_tag: false,
            last_was_space: false,
            output,
            tag_buf: String::new(),
        }
    }

    /// Feed a chunk of HTML bytes. Chunks are processed char-by-char with
    /// simple tag-stripping and whitespace compaction. Multi-byte UTF-8
    /// sequences that span chunk boundaries are handled via `from_utf8_lossy`.
    pub fn feed(&mut self, chunk: &[u8]) {
        let s = String::from_utf8_lossy(chunk);
        for c in s.chars() {
            if self.in_tag {
                if c == '>' {
                    self.in_tag = false;
                    let tag = self.tag_buf.trim().trim_start_matches('/').to_ascii_lowercase();
                    if tag.starts_with('p')
                        || tag.starts_with("div")
                        || tag.starts_with("br")
                        || tag.starts_with('h')
                        || tag.starts_with("li")
                    {
                        self.output.push('\n');
                    }
                    self.tag_buf.clear();
                } else {
                    self.tag_buf.push(c);
                }
            } else if c == '<' {
                self.in_tag = true;
                self.tag_buf.clear();
            } else if c.is_whitespace() {
                if !self.last_was_space {
                    self.output.push(' ');
                    self.last_was_space = true;
                }
            } else {
                self.output.push(c);
                self.last_was_space = false;
            }
        }
    }

    /// Finalize extraction, trim trailing whitespace from each line,
    /// compact multiple blank lines, and return the result.
    pub fn finish(mut self) -> Result<String> {
        let bytes = unsafe { self.output.as_mut_vec() };
        let len = bytes.len();
        let mut write = 0usize;
        let mut first_line = true;
        let mut i = 0usize;
        while i < len {
            let line_start = i;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            let line_end = i;
            // trim trailing whitespace on the line
            let mut trim_end = line_end;
            while trim_end > line_start && bytes[trim_end - 1].is_ascii_whitespace() {
                trim_end -= 1;
            }
            if trim_end > line_start {
                if !first_line {
                    bytes[write] = b'\n';
                    write += 1;
                }
                let seg_len = trim_end - line_start;
                if write != line_start {
                    bytes.copy_within(line_start..trim_end, write);
                }
                write += seg_len;
                first_line = false;
            }
            if i < len {
                i += 1; // skip '\n'
            }
        }
        bytes.truncate(write);
        Ok(self.output)
    }
}

#[cfg(feature = "lowmem")]
/// Extract text from HTML using the streaming char-by-char path (no tl dependency).
pub fn extract_text_content(html: &str) -> Result<String> {
    let mut extractor = StreamingTextExtractor::new();
    extractor.feed(html.as_bytes());
    extractor.finish()
}

// AST parsing always uses tl (full HTML must be in memory)
fn parse_html_ast(html: &str) -> Result<AstNode> {
    let dom = tl::parse(html, ParserOptions::default())
        .map_err(|e| LexEpubError::Html(format!("Failed to parse HTML: {}", e)))?;

    let parser = dom.parser();

    let mut root_children = Vec::new();

    for handle in dom.children() {
        if let Some(ast_child) = node_to_ast(*handle, parser) {
            root_children.push(ast_child);
        }
    }

    Ok(AstNode::Element {
        tag: "root".to_string(),
        attrs: HashMap::new(),
        styles: HashMap::new(),
        children: root_children,
    })
}

fn node_to_ast(handle: tl::NodeHandle, parser: &tl::Parser) -> Option<AstNode> {
    let node = handle.get(parser)?;

    match node {
        tl::Node::Tag(tag) => {
            let mut attrs = HashMap::new();

            for attr in tag.attributes().iter() {
                let key = attr.0.to_string();
                let value = attr
                    .1
                    .map(|v| html_escape::decode_html_entities(&v.to_string()).into_owned())
                    .unwrap_or_default();
                attrs.insert(key, value);
            }

            let mut children = Vec::new();
            for child_handle in tag.children().top().iter() {
                if let Some(child_ast) = node_to_ast(*child_handle, parser) {
                    children.push(child_ast);
                }
            }

            Some(AstNode::Element {
                tag: tag.name().as_utf8_str().to_string(),
                attrs,
                styles: HashMap::new(),
                children,
            })
        }
        tl::Node::Raw(text_ref) => Some(AstNode::Text {
            content: html_escape::decode_html_entities(&text_ref.as_utf8_str()).into_owned(),
        }),
        tl::Node::Comment(comment_ref) => Some(AstNode::Comment {
            content: html_escape::decode_html_entities(&comment_ref.as_utf8_str()).into_owned(),
        }),
    }
}

fn extract_title_from_ast(ast: &AstNode) -> Option<String> {
    fn first_non_empty_text(node: &AstNode) -> Option<String> {
        match node {
            AstNode::Text { content } => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
            AstNode::Element { children, .. } => {
                for child in children {
                    if let Some(text) = first_non_empty_text(child) {
                        return Some(text);
                    }
                }
                None
            }
            AstNode::Comment { .. } => None,
        }
    }

    fn find_by_tag(node: &AstNode, target: &str) -> Option<String> {
        match node {
            AstNode::Element { tag, children, .. } => {
                if tag.eq_ignore_ascii_case(target) {
                    return first_non_empty_text(node);
                }
                for child in children {
                    if let Some(found) = find_by_tag(child, target) {
                        return Some(found);
                    }
                }
                None
            }
            _ => None,
        }
    }

    for tag in ["h1", "h2", "title"] {
        if let Some(found) = find_by_tag(ast, tag) {
            return Some(found);
        }
    }

    first_non_empty_text(ast)
}
