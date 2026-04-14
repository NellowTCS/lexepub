use lexepub::core::chapter::AstNode;
use lexepub::core::css::Stylesheet;
use std::collections::HashMap;

#[test]
fn test_apply_css_to_ast() {
    let css = "
    p { color: red; font-size: 12px; }
    .highlight { font-weight: bold; }
    #main { background: blue; }
    p.highlight { color: yellow; }
    ";

    let stylesheet = Stylesheet::parse(css);

    let pt1_attrs = HashMap::new();
    let mut pt2_attrs = HashMap::new();
    pt2_attrs.insert("class".to_string(), "highlight".to_string());

    let mut div_attrs = HashMap::new();
    div_attrs.insert("id".to_string(), "main".to_string());

    let mut root = AstNode::Element {
        tag: "div".to_string(),
        attrs: div_attrs,
        styles: HashMap::new(),
        children: vec![
            AstNode::Element {
                tag: "p".to_string(),
                attrs: pt1_attrs,
                styles: HashMap::new(),
                children: vec![],
            },
            AstNode::Element {
                tag: "p".to_string(),
                attrs: pt2_attrs,
                styles: HashMap::new(),
                children: vec![],
            },
        ],
    };

    stylesheet.apply_to_ast(&mut root);

    if let AstNode::Element {
        styles: div_styles,
        children,
        ..
    } = &root
    {
        assert_eq!(div_styles.get("background").unwrap(), "blue");

        if let AstNode::Element {
            styles: p1_styles, ..
        } = &children[0]
        {
            assert_eq!(p1_styles.get("color").unwrap(), "red");
            assert_eq!(p1_styles.get("font-size").unwrap(), "12px");
        } else {
            panic!("Expected p element");
        }

        if let AstNode::Element {
            styles: p2_styles, ..
        } = &children[1]
        {
            assert_eq!(p2_styles.get("color").unwrap(), "yellow");
            assert_eq!(p2_styles.get("font-weight").unwrap(), "bold");
            assert_eq!(p2_styles.get("font-size").unwrap(), "12px");
        } else {
            panic!("Expected p element");
        }
    } else {
        panic!("Expected div element");
    }
}
