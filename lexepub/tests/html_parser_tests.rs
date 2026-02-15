#[cfg(test)]
mod tests {
    use lexepub::core::chapter::{AstNode, Chapter};
    use lexepub::core::html_parser::ChapterParser;
    use std::collections::HashMap;

    #[test]
    fn test_chapter_parser_creation() {
        let parser = ChapterParser::new();
        assert!(parser.text_only);
        assert!(!parser.with_ast);

        let text_only = ChapterParser::new().text_only();
        assert!(text_only.text_only);
        assert!(!text_only.with_ast);

        let with_ast = ChapterParser::new().with_ast();
        assert!(!with_ast.text_only);
        assert!(with_ast.with_ast);

        let with_both = ChapterParser::new().with_both();
        assert!(!with_both.text_only);
        assert!(with_both.with_ast);
    }

    #[test]
    fn test_extract_text_content() {
        let html = r#"
            <html>
                <body>
                    <h1>Title</h1>
                    <p>This is a paragraph.</p>
                    <p>Another paragraph.</p>
                </body>
            </html>
        "#;

        let result = lexepub::core::html_parser::extract_text_content(html);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Title"));
        assert!(text.contains("This is a paragraph"));
        assert!(text.contains("Another paragraph"));
    }

    #[test]
    fn test_parse_simple_chapter() {
        let chapter = Chapter {
            href: "chapter1.xhtml".to_string(),
            id: "chapter1".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            content: b"<html><body><p>Hello world</p></body></html>".to_vec(),
        };

        let parser = ChapterParser::new().text_only();
        let result = parser.parse_chapter(chapter);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.content, "Hello world");
        assert_eq!(parsed.word_count, 2);
        assert_eq!(parsed.char_count, 11);
        assert!(parsed.ast.is_none());
    }

    #[test]
    fn test_parse_chapter_with_ast() {
        let chapter = Chapter {
            href: "chapter1.xhtml".to_string(),
            id: "chapter1".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            content: b"<html><body><p>Hello world</p></body></html>".to_vec(),
        };

        let parser = ChapterParser::new().with_ast();
        let result = parser.parse_chapter(chapter);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.content, "Hello world");
        assert!(parsed.ast.is_some());
    }

    #[test]
    fn test_ast_node_serialization() {
        let node = AstNode::Element {
            tag: "p".to_string(),
            attrs: HashMap::new(),
            children: vec![AstNode::Text {
                content: "Hello".to_string(),
            }],
        };

        let serialized = serde_json::to_string(&node);
        assert!(serialized.is_ok());

        let deserialized: Result<AstNode, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}
