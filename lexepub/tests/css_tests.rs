use lexepub::core::css::*;

#[test]
fn test_epub_css_parsing() {
    let css = r#"
        /* EPUB base CSS */
        @namespace epub "http://www.idpf.org/2007/ops";
        @import url("fonts.css");

        @font-face {
            font-family: "MyFont";
            src: url("fonts/MyFont.otf") format("opentype");
        }

        body { font-size: 16px; margin: 0; }
        
        @media screen and (min-width: 480px) {
            body { font-size: 18px; }
        }
        
        @page {
            margin: 2em;
        }

        h1, h2 {
            color: red;
            background-image: url("data:image/svg+xml;base64,PHN2Zy...");
        }
    "#;
    let stylesheet = Stylesheet::parse(css);

    assert_eq!(stylesheet.rules.len(), 7); // @namespace, @import, @font-face, body, @media, @page, h1,h2

    match &stylesheet.rules[0] {
        CssRule::Namespace(ns) => assert_eq!(ns, "epub \"http://www.idpf.org/2007/ops\""),
        _ => panic!("Expected @namespace"),
    }

    match &stylesheet.rules[1] {
        CssRule::Import(url) => assert_eq!(url, "url(\"fonts.css\")"),
        _ => panic!("Expected @import"),
    }

    match &stylesheet.rules[2] {
        CssRule::FontFace(decls) => {
            assert_eq!(decls.get("font-family").unwrap(), "\"MyFont\"");
            assert_eq!(
                decls.get("src").unwrap(),
                "url(\"fonts/MyFont.otf\") format(\"opentype\")"
            );
        }
        _ => panic!("Expected @font-face"),
    }

    match &stylesheet.rules[3] {
        CssRule::Style(rule) => {
            assert_eq!(rule.selectors, "body");
            assert_eq!(rule.declarations.get("margin").unwrap(), "0");
        }
        _ => panic!("Expected body style"),
    }

    match &stylesheet.rules[4] {
        CssRule::Media { query, rules } => {
            assert_eq!(query, "screen and (min-width: 480px)");
            assert_eq!(rules.len(), 1);
            match &rules[0] {
                CssRule::Style(r) => assert_eq!(r.declarations.get("font-size").unwrap(), "18px"),
                _ => panic!("Expected nested style inside @media"),
            }
        }
        _ => panic!("Expected @media"),
    }

    match &stylesheet.rules[5] {
        CssRule::Page {
            selectors,
            declarations,
        } => {
            assert_eq!(selectors, "");
            assert_eq!(declarations.get("margin").unwrap(), "2em");
        }
        _ => panic!("Expected @page"),
    }

    match &stylesheet.rules[6] {
        CssRule::Style(rule) => {
            assert_eq!(rule.selectors, "h1, h2");
            assert_eq!(
                rule.declarations.get("background-image").unwrap(),
                "url(\"data:image/svg+xml;base64,PHN2Zy...\")"
            );
        }
        _ => panic!("Expected h1,h2 style"),
    }
}
