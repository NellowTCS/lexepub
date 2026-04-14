use std::collections::HashMap;

/// Represents a parsed CSS Stylesheet
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssRule {
    /// Standard style rule
    Style(StyleRule),
    /// @media block rule
    Media { query: String, rules: Vec<CssRule> },
    /// @supports block rule
    Supports { query: String, rules: Vec<CssRule> },
    /// @font-face rule
    FontFace(HashMap<String, String>),
    /// @page rule
    Page {
        selectors: String,
        declarations: HashMap<String, String>,
    },
    /// @import rule
    Import(String),
    /// @namespace rule
    Namespace(String),
    /// Other unparsed blocks
    Other { name: String, content: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    /// Raw selector string
    pub selectors: String,
    pub declarations: HashMap<String, String>,
}

impl Stylesheet {
    /// Parse a complete CSS string into a reusable Stylesheet AST.
    pub fn parse(css_text: &str) -> Self {
        let cleaned_css = Self::remove_comments(css_text);
        Self {
            rules: Self::parse_rules(&cleaned_css),
        }
    }

    fn parse_rules(mut css: &str) -> Vec<CssRule> {
        let mut rules = Vec::new();

        while let Some(first_non_whitespace) = css.chars().position(|c| !c.is_whitespace()) {
            css = &css[first_non_whitespace..];

            // Handle at-rules
            if css.starts_with('@') {
                let semi_idx = css.find(';');
                let brace_idx = css.find('{');

                // If there's a semicolon before a brace (like @import, @namespace ugh)
                if let Some(end_idx) = semi_idx {
                    if brace_idx.is_none_or(|b_idx| end_idx < b_idx) {
                        let rule_text = css[..end_idx].trim();
                        if let Some(stripped) = rule_text.strip_prefix("@import") {
                            rules.push(CssRule::Import(stripped.trim().to_string()));
                        } else if let Some(stripped) = rule_text.strip_prefix("@namespace") {
                            rules.push(CssRule::Namespace(stripped.trim().to_string()));
                        } else {
                            // Unsupported single-line at-rule
                            rules.push(CssRule::Other {
                                name: rule_text.to_string(),
                                content: String::new(),
                            });
                        }
                        css = &css[end_idx + 1..];
                        continue;
                    }
                }
            }

            // Find next block `{ ... }`
            let brace_idx = match css.find('{') {
                Some(idx) => idx,
                None => break, // No more blocks, we're done FINALLY
            };

            let prelude = css[..brace_idx].trim().to_string();
            css = &css[brace_idx..];

            // Match the block content
            let mut brace_count = 0;
            let mut content_end = 0;
            for (i, c) in css.char_indices() {
                if c == '{' {
                    brace_count += 1;
                } else if c == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        content_end = i;
                        break;
                    }
                }
            }

            // Unmatched braces edge case
            if brace_count > 0 {
                content_end = css.len();
            }

            let block_content = &css[1..content_end].trim();

            if let Some(stripped) = prelude.strip_prefix("@media") {
                rules.push(CssRule::Media {
                    query: stripped.trim().to_string(),
                    rules: Self::parse_rules(block_content),
                });
            } else if let Some(stripped) = prelude.strip_prefix("@supports") {
                rules.push(CssRule::Supports {
                    query: stripped.trim().to_string(),
                    rules: Self::parse_rules(block_content),
                });
            } else if prelude.starts_with("@font-face") {
                rules.push(CssRule::FontFace(Self::parse_declarations(block_content)));
            } else if let Some(stripped) = prelude.strip_prefix("@page") {
                rules.push(CssRule::Page {
                    selectors: stripped.trim().to_string(),
                    declarations: Self::parse_declarations(block_content),
                });
            } else if prelude.starts_with('@') {
                rules.push(CssRule::Other {
                    name: prelude,
                    content: block_content.to_string(),
                });
            } else if !prelude.is_empty() {
                rules.push(CssRule::Style(StyleRule {
                    selectors: prelude,
                    declarations: Self::parse_declarations(block_content),
                }));
            }

            if content_end + 1 < css.len() {
                css = &css[content_end + 1..];
            } else {
                break;
            }
        }

        rules
    }

    fn remove_comments(css: &str) -> String {
        let mut result = String::with_capacity(css.len());
        let mut in_comment = false;
        let mut chars = css.char_indices().peekable();

        while let Some((_, c)) = chars.next() {
            if in_comment {
                if c == '*' {
                    if let Some(&(_, '/')) = chars.peek() {
                        chars.next();
                        in_comment = false;
                    }
                }
            } else if c == '/' {
                if let Some(&(_, '*')) = chars.peek() {
                    chars.next();
                    in_comment = true;
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    fn parse_declarations(block: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        // Custom simple parser instead of loop to handle string literals with semicolons because CSS is a nightmare.
        let mut decl_start = 0;
        let mut in_string = false;
        let mut string_char = ' ';

        for (i, c) in block.char_indices() {
            if c == '"' || c == '\'' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if string_char == c {
                    // Could check for basic escaping here, omitting for simplicity since EPUB CSS is generally clean
                    // TODO: Add escape handling if needed
                    in_string = false;
                }
            } else if c == ';' && !in_string {
                let decl = &block[decl_start..i].trim();
                Self::insert_declaration(decl, &mut map);
                decl_start = i + 1;
            }
        }

        // Final trailing declaration if missing semicolon
        if decl_start < block.len() {
            Self::insert_declaration(block[decl_start..].trim(), &mut map);
        }

        map
    }

    fn insert_declaration(decl: &str, map: &mut HashMap<String, String>) {
        if decl.is_empty() {
            return;
        }
        if let Some(colon_idx) = decl.find(':') {
            let name = decl[..colon_idx].trim().to_string();
            let value = decl[colon_idx + 1..].trim().to_string();
            if !name.is_empty() && !value.is_empty() {
                map.insert(name, value);
            }
        }
    }
}

impl Stylesheet {
    /// Apply this stylesheet's rules directly to an AstNode tree (computes inline styles based on selectors).
    pub fn apply_to_ast(&self, ast: &mut crate::core::chapter::AstNode) {
        let mut parent_path = Vec::new();
        self.apply_to_node_recursive(ast, &mut parent_path);
    }

    fn apply_to_node_recursive<'a>(
        &self,
        node: &mut crate::core::chapter::AstNode,
        mut _parent_path: &mut Vec<(&'a str, &'a HashMap<String, String>)>,
    ) {
        if let crate::core::chapter::AstNode::Element {
            tag,
            attrs,
            styles,
            children,
        } = node
        {
            // Temporary collection of matched styles for this element
            let mut computed_styles = HashMap::new();

            for rule in &self.rules {
                if let CssRule::Style(style_rule) = rule {
                    if Self::matches_selector_group(&style_rule.selectors, tag, attrs) {
                        for (k, v) in &style_rule.declarations {
                            computed_styles.insert(k.clone(), v.clone());
                        }
                    }
                }
            }

            // Also append existing inline styles if any are explicitly defined
            if let Some(inline) = attrs.get("style") {
                let inline_map = Self::parse_declarations(inline);
                for (k, v) in inline_map {
                    computed_styles.insert(k, v);
                }
            }

            for (k, v) in computed_styles {
                styles.insert(k, v);
            }

            // I need unsafe tricks or separate structural recursions to push into parent_path sigh
            // Since EPUB is mostly flat selectors, I'll keep it simple for now.
            // TODO: Implement full parent path tracking for combinator selectors when needed in the future

            for child in children {
                self.apply_to_node_recursive(child, _parent_path);
            }
        }
    }

    /// Evaluates a selector group like `h1, h2, p.highlight`
    fn matches_selector_group(selectors: &str, tag: &str, attrs: &HashMap<String, String>) -> bool {
        for selector in selectors.split(',') {
            if Self::matches_single_selector(selector.trim(), tag, attrs) {
                return true;
            }
        }
        false
    }

    /// Evaluates a single selector like `p.highlight`
    fn matches_single_selector(selector: &str, tag: &str, attrs: &HashMap<String, String>) -> bool {
        let mut current = selector;

        // Very basic simple selector parsing matching node target against tag/class/id
        let mut expected_tag = None;
        if let Some(c) = current.chars().next() {
            if c != '.' && c != '#' {
                // Find end of tag
                let end = current.find(['.', '#']).unwrap_or(current.len());
                expected_tag = Some(&current[..end]);
                current = &current[end..];
            }
        }

        if let Some(t) = expected_tag {
            if t != tag && t != "*" {
                return false;
            }
        }

        while !current.is_empty() {
            if current.starts_with('.') {
                let end = current[1..]
                    .find(['.', '#'])
                    .map(|i| i + 1)
                    .unwrap_or(current.len());
                let class_name = &current[1..end];

                let has_class = attrs
                    .get("class")
                    .map(|c| c.split_whitespace().any(|cls| cls == class_name))
                    .unwrap_or(false);

                if !has_class {
                    return false;
                }
                current = &current[end..];
            } else if current.starts_with('#') {
                let end = current[1..]
                    .find(['.', '#'])
                    .map(|i| i + 1)
                    .unwrap_or(current.len());
                let id_name = &current[1..end];

                let has_id = attrs.get("id").map(|i| i == id_name).unwrap_or(false);
                if !has_id {
                    return false;
                }
                current = &current[end..];
            } else {
                break;
            }
        }

        true
    }
}
