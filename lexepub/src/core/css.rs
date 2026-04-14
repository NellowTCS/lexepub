use std::collections::HashMap;

/// Represents a parsed CSS Stylesheet
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssRule {
    /// Standard style rule (e.g., `p { color: red; }`)
    Style(StyleRule),
    /// At-rule or unsupported rule we didn't fully parse but want to keep as text
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
    /// Uses basic character matching to isolate rule blocks and properties.
    pub fn parse(css_text: &str) -> Self {
        let mut rules = Vec::new();
        // Remove comments
        let cleaned_css = Self::remove_comments(css_text);

        let mut current_pos = 0;
        let chars_len = cleaned_css.len();
        
        while current_pos < chars_len {
            // Find next '{'
            let next_open = cleaned_css[current_pos..].find('{');
            if let Some(open_idx) = next_open {
                let absolute_open = current_pos + open_idx;
                
                // Extract prelude (selector or at-rule name)
                let prelude = cleaned_css[current_pos..absolute_open].trim().to_string();
                
                // Find matching '}'
                let mut brace_count = 1;
                let mut content_end = absolute_open + 1;
                for (i, c) in cleaned_css[absolute_open + 1..].char_indices() {
                    if c == '{' {
                        brace_count += 1;
                    } else if c == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            content_end = absolute_open + 1 + i;
                            break;
                        }
                    }
                }
                
                // If we didn't find a matching brace, break out softly
                if brace_count > 0 {
                    break;
                }

                let block_content = &cleaned_css[absolute_open + 1..content_end];
                
                if prelude.starts_with('@') {
                    rules.push(CssRule::Other {
                        name: prelude,
                        content: block_content.trim().to_string(),
                    });
                } else if !prelude.is_empty() {
                    let declarations = Self::parse_declarations(block_content);
                    rules.push(CssRule::Style(StyleRule {
                        selectors: prelude,
                        declarations,
                    }));
                }

                current_pos = content_end + 1;
            } else {
                break;
            }
        }

        Self { rules }
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
        // Split by ';' but respect potential enclosed strings
        for decl in block.split(';') {
            let decl = decl.trim();
            if decl.is_empty() {
                continue;
            }
            if let Some(colon_idx) = decl.find(':') {
                let name = decl[..colon_idx].trim().to_string();
                let value = decl[colon_idx + 1..].trim().to_string();
                if !name.is_empty() && !value.is_empty() {
                    map.insert(name, value);
                }
            }
        }
        map
    }
}
