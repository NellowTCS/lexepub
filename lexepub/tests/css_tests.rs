use lexepub::core::css::*;

#[test]
fn test_css_parsing() {
    let css = r#"
        /* Comment */
        body { font-size: 16px; margin: 0; }
        @media screen and (min-width: 480px) {
            body { font-size: 18px; }
        }
        h1, h2 {
            color: red;
        }
    "#;
    let stylesheet = Stylesheet::parse(css);
    
    assert_eq!(stylesheet.rules.len(), 3);
    
    if let CssRule::Style(ref rule) = stylesheet.rules[0] {
        assert_eq!(rule.selectors, "body");
        assert_eq!(rule.declarations.get("font-size").unwrap(), "16px");
        assert_eq!(rule.declarations.get("margin").unwrap(), "0");
    } else {
        panic!("First rule not a style rule");
    }
    
    if let CssRule::Other { ref name, ref content } = stylesheet.rules[1] {
        assert_eq!(name, "@media screen and (min-width: 480px)");
        assert!(content.contains("body { font-size: 18px; }"));
    } else {
        panic!("Second rule not an @media rule");
    }
    
    if let CssRule::Style(ref rule) = stylesheet.rules[2] {
        assert_eq!(rule.selectors, "h1, h2");
        assert_eq!(rule.declarations.get("color").unwrap(), "red");
    } else {
        panic!("Third rule not a style rule");
    }
}
