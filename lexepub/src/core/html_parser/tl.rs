use crate::core::chapter::{
    AstNode, Chapter, FormattingRun, ParsedChapter, STYLE_BOLD, STYLE_CODE, STYLE_ITALIC,
    STYLE_STRIKETHROUGH, STYLE_UNDERLINE,
};
use crate::error::{LexEpubError, Result};
use std::collections::HashMap;
use tl::ParserOptions;

/// Configurable chapter parser using the `tl` DOM library.
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text_only(mut self) -> Self {
        self.text_only = true;
        self.with_ast = false;
        self
    }

    pub fn with_ast(mut self) -> Self {
        self.text_only = false;
        self.with_ast = true;
        self
    }

    pub fn with_both(mut self) -> Self {
        self.text_only = false;
        self.with_ast = true;
        self
    }

    pub fn parse_chapter(&self, chapter: Chapter) -> Result<ParsedChapter> {
        let content_str = std::str::from_utf8(&chapter.content)?;

        let ast = if self.with_ast {
            Some(super::parse_html_ast(content_str)?)
        } else {
            None
        };

        let (content, formatting_runs) = if !self.text_only && !self.with_ast {
            (content_str.to_string(), Vec::new())
        } else {
            (
                super::extract_text_content(content_str)?,
                super::extract_formatting(content_str)?,
            )
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
            formatting_runs,
            word_count,
            char_count,
        })
    }
}

pub fn extract_text_content(html: &str) -> Result<String> {
    let dom = tl::parse(html, ParserOptions::default())
        .map_err(|e| LexEpubError::Html(format!("Failed to parse HTML: {}", e)))?;

    let parser = dom.parser();
    let mut text = String::new();

    for handle in dom.children() {
        extract_text_recursive(*handle, parser, &mut text);
    }

    let cleaned = text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(cleaned)
}

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

                for child_handle in tag.children().top().iter() {
                    extract_text_recursive(*child_handle, parser, output);
                }

                if is_block {
                    output.push('\n');
                }
            }
            tl::Node::Comment(_) => {}
        }
    }
}

/// Derive formatting runs from a parsed tl DOM by walking the AST.
pub fn extract_formatting(html: &str) -> Result<Vec<FormattingRun>> {
    let dom = tl::parse(html, ParserOptions::default())
        .map_err(|e| LexEpubError::Html(format!("Failed to parse HTML: {}", e)))?;

    let parser = dom.parser();
    let mut runs = Vec::new();
    let mut style_stack = Vec::new();
    let mut heading_level: u8 = 0;

    for handle in dom.children() {
        collect_runs(
            *handle,
            parser,
            &mut runs,
            &mut style_stack,
            &mut heading_level,
        );
    }

    // Flush trailing whitespace-only runs
    runs.retain(|r| !r.text.is_empty() && r.text != " ");
    Ok(runs)
}

fn collect_runs(
    handle: tl::NodeHandle,
    parser: &tl::Parser,
    runs: &mut Vec<FormattingRun>,
    style_stack: &mut Vec<u8>,
    heading_level: &mut u8,
) {
    let Some(node) = handle.get(parser) else {
        return;
    };

    match node {
        tl::Node::Raw(text_bytes) => {
            let text_str = text_bytes.as_utf8_str();
            let decoded = html_escape::decode_html_entities(&text_str);
            let style = style_stack.iter().fold(0u8, |acc, s| acc | s);
            if let Some(last) = runs.last_mut() {
                if last.style == style && last.heading == *heading_level {
                    last.text.push_str(&decoded);
                    return;
                }
            }
            runs.push(FormattingRun {
                text: decoded.into_owned(),
                style,
                heading: *heading_level,
            });
        }
        tl::Node::Tag(tag) => {
            let tag_lower = tag.name().as_utf8_str().to_lowercase();
            let prev_heading = *heading_level;

            match tag_lower.as_str() {
                "b" | "strong" => style_stack.push(STYLE_BOLD),
                "i" | "em" => style_stack.push(STYLE_ITALIC),
                "u" => style_stack.push(STYLE_UNDERLINE),
                "s" | "strike" | "del" => style_stack.push(STYLE_STRIKETHROUGH),
                "code" | "tt" | "pre" => style_stack.push(STYLE_CODE),
                h if h.starts_with('h') && h.len() == 2 => {
                    if let Ok(n) = h[1..].parse::<u8>() {
                        if (1..=6).contains(&n) {
                            *heading_level = n;
                        }
                    }
                }
                "br" => {
                    runs.push(FormattingRun {
                        text: "\n".into(),
                        style: 0,
                        heading: 0,
                    });
                }
                _ => {}
            }

            for child_handle in tag.children().top().iter() {
                collect_runs(*child_handle, parser, runs, style_stack, heading_level);
            }

            // Pop style
            match tag_lower.as_str() {
                "b" | "strong" => {
                    style_stack.pop();
                }
                "i" | "em" => {
                    style_stack.pop();
                }
                "u" => {
                    style_stack.pop();
                }
                "s" | "strike" | "del" => {
                    style_stack.pop();
                }
                "code" | "tt" | "pre" => {
                    style_stack.pop();
                }
                h if h.starts_with('h') && h.len() == 2 => {
                    *heading_level = prev_heading;
                }
                _ => {}
            }

            // Block elements emit newline
            let is_block = matches!(
                tag_lower.as_str(),
                "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li"
            );
            if is_block {
                runs.push(FormattingRun {
                    text: "\n".into(),
                    style: 0,
                    heading: 0,
                });
            }
        }
        tl::Node::Comment(_) => {}
    }
}

pub fn parse_html_ast(html: &str) -> Result<AstNode> {
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
