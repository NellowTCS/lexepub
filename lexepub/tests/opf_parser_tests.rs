#[cfg(test)]
mod tests {
    use lexepub::core::opf_parser::OpfParser;

    #[test]
    fn test_opf_parser_creation() {
        let _parser = OpfParser::new();
        // Note: reader field is private, so we can't test buffer_position
        // This test just ensures the parser can be created
        // TODO: FIX HAHA
        assert!(true);
    }

    #[test]
    fn test_parse_metadata_minimal() {
        let xml = r#"<?xml version="1.0"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <metadata>
    <dc:title>Test Book</dc:title>
    <dc:creator>Test Author</dc:creator>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chapter1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#;

        let mut parser = OpfParser::new();
        let result = parser.parse_metadata(xml.as_bytes());
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.title, Some("Test Book".to_string()));
        assert_eq!(metadata.creators, vec!["Test Author"]);
        assert_eq!(metadata.languages, vec!["en"]);
        assert_eq!(metadata.spine, vec!["chapter1"]);
        assert_eq!(
            metadata.manifest.get("chapter1"),
            Some(&"chapter1.xhtml".to_string())
        );
    }

    #[test]
    fn test_parse_spine() {
        let xml = r#"<?xml version="1.0"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <spine>
    <itemref idref="chapter1"/>
    <itemref idref="chapter2"/>
  </spine>
</package>"#;

        let mut parser = OpfParser::new();
        let result = parser.parse_spine(xml.as_bytes());
        assert!(result.is_ok());

        let spine = result.unwrap();
        assert_eq!(spine, vec!["chapter1", "chapter2"]);
    }
}
