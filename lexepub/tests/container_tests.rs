#[cfg(test)]
mod tests {
    use lexepub::core::container::ContainerParser;

    #[test]
    fn test_container_parser_creation() {
        let _parser = ContainerParser::new();
        // Test just ensures the parser can be created
        assert!(true);
    }

    #[test]
    fn test_parse_container_minimal() {
        let xml = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

        let mut parser = ContainerParser::new();
        let result = parser.parse_container(xml.as_bytes());
        assert!(result.is_ok());

        let container = result.unwrap();
        assert_eq!(container.rootfile_path, "OEBPS/content.opf");
    }

    #[test]
    fn test_parse_container_invalid_xml() {
        let xml = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <!-- Missing rootfile element -->
  </rootfiles>
</container>"#;

        let mut parser = ContainerParser::new();
        let result = parser.parse_container(xml.as_bytes());
        assert!(result.is_err());
    }
}
