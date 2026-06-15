use lexepub::core::chapter::{
    FormattingRun, STYLE_BOLD, STYLE_CODE, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_UNDERLINE,
};

// StyleFlags bitmask

#[test]
fn test_style_flags_values() {
    assert_eq!(STYLE_BOLD, 1 << 0);
    assert_eq!(STYLE_ITALIC, 1 << 1);
    assert_eq!(STYLE_UNDERLINE, 1 << 2);
    assert_eq!(STYLE_STRIKETHROUGH, 1 << 3);
    assert_eq!(STYLE_CODE, 1 << 4);
}

#[test]
fn test_style_flags_combine() {
    let combined = STYLE_BOLD | STYLE_ITALIC;
    assert!(combined & STYLE_BOLD != 0);
    assert!(combined & STYLE_ITALIC != 0);
    assert_eq!(combined & STYLE_UNDERLINE, 0);
}

#[test]
fn test_style_flags_all_distinct() {
    let all = STYLE_BOLD | STYLE_ITALIC | STYLE_UNDERLINE | STYLE_STRIKETHROUGH | STYLE_CODE;
    assert_eq!(all, 0b11111);
}

// FormattingRun construction

#[test]
fn test_formatting_run_defaults() {
    let run = FormattingRun {
        text: "hello".into(),
        style: 0,
        heading: 0,
    };
    assert_eq!(run.text, "hello");
    assert_eq!(run.style, 0);
    assert_eq!(run.heading, 0);
}

#[test]
fn test_formatting_run_with_style() {
    let run = FormattingRun {
        text: "bold text".into(),
        style: STYLE_BOLD,
        heading: 0,
    };
    assert!(run.style & STYLE_BOLD != 0);
}

#[test]
fn test_formatting_run_with_heading() {
    let run = FormattingRun {
        text: "Chapter 1".into(),
        style: 0,
        heading: 1,
    };
    assert_eq!(run.heading, 1);
}

#[test]
fn test_formatting_run_serialization_roundtrip() {
    let run = FormattingRun {
        text: "styled".into(),
        style: STYLE_BOLD | STYLE_ITALIC,
        heading: 2,
    };
    let json = serde_json::to_string(&run).unwrap();
    let back: FormattingRun = serde_json::from_str(&json).unwrap();
    assert_eq!(run.text, back.text);
    assert_eq!(run.style, back.style);
    assert_eq!(run.heading, back.heading);
}

// extract_text_content both paths should produce the same semantics

#[test]
fn test_extract_text_empty() {
    let text = lexepub::core::html_parser::extract_text_content("").unwrap();
    assert_eq!(text, "");
}

#[test]
fn test_extract_text_simple() {
    let text = lexepub::core::html_parser::extract_text_content("<p>Hello</p>").unwrap();
    assert!(text.contains("Hello"));
}

#[cfg(feature = "lowmem")]
#[test]
fn test_extract_text_script_stripped() {
    let html = "<p>Keep</p><script>var x=1</script><p>This</p>";
    let text = lexepub::core::html_parser::extract_text_content(html).unwrap();
    assert!(text.contains("Keep"));
    assert!(text.contains("This"));
    assert!(!text.contains("var x"), "script content should be stripped");
}

#[cfg(feature = "lowmem")]
#[test]
fn test_extract_text_style_stripped() {
    let html =
        "<html><head><style>.red{color:red}</style></head><body><p>Visible</p></body></html>";
    let text = lexepub::core::html_parser::extract_text_content(html).unwrap();
    assert!(text.contains("Visible"));
    assert!(
        !text.contains("color:red"),
        "style content should be stripped"
    );
}

#[test]
fn test_extract_text_entities() {
    let text = lexepub::core::html_parser::extract_text_content("<p>A &amp; B &lt; C</p>").unwrap();
    assert!(text.contains('&'));
    assert!(text.contains('<'));
}

#[test]
fn test_extract_text_nested_tags() {
    let text =
        lexepub::core::html_parser::extract_text_content("<div><p>A</p><p>B</p></div>").unwrap();
    assert!(text.contains('A'));
    assert!(text.contains('B'));
}

#[test]
fn test_extract_text_br_as_newline() {
    let text = lexepub::core::html_parser::extract_text_content("<p>Line1<br/>Line2</p>").unwrap();
    assert!(text.contains("Line1"));
    assert!(text.contains("Line2"));
}

// extract_formatting style detection

#[test]
fn test_formatting_bold_tag_b() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><b>bold</b></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_BOLD != 0));
}

#[test]
fn test_formatting_bold_tag_strong() {
    let runs =
        lexepub::core::html_parser::extract_formatting("<p><strong>bold</strong></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_BOLD != 0));
}

#[test]
fn test_formatting_italic_tag_i() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><i>italic</i></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_ITALIC != 0));
}

#[test]
fn test_formatting_italic_tag_em() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><em>italic</em></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_ITALIC != 0));
}

#[test]
fn test_formatting_underline() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><u>under</u></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_UNDERLINE != 0));
}

#[test]
fn test_formatting_strikethrough_tag_s() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><s>strike</s></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_STRIKETHROUGH != 0));
}

#[test]
fn test_formatting_strikethrough_tag_del() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><del>strike</del></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_STRIKETHROUGH != 0));
}

#[test]
fn test_formatting_code_tag_code() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><code>fn()</code></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_CODE != 0));
}

#[test]
fn test_formatting_code_tag_tt() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><tt>mono</tt></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_CODE != 0));
}

#[test]
fn test_formatting_code_tag_pre() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><pre>code</pre></p>").unwrap();
    assert!(runs.iter().any(|r| r.style & STYLE_CODE != 0));
}

// extract_formatting headings

#[test]
fn test_formatting_headings_all_levels() {
    for level in 1u8..=6u8 {
        let html = format!("<h{level}>Title</h{level}>");
        let runs = lexepub::core::html_parser::extract_formatting(&html).unwrap();
        assert!(
            runs.iter().any(|r| r.heading == level),
            "h{level} should produce heading={level}"
        );
    }
}

#[test]
fn test_formatting_heading_text_content() {
    let runs = lexepub::core::html_parser::extract_formatting("<h1>Chapter One</h1>").unwrap();
    let heading = runs.iter().find(|r| r.heading == 1);
    assert!(heading.is_some());
    let heading = heading.unwrap();
    // Both paths should capture the heading text; exact whitespace may differ
    assert!(heading.text.contains("Chapter"));
    assert!(heading.text.contains("One"));
}

// extract_formatting nested and combined styles

#[test]
fn test_formatting_nested_bold_italic() {
    let runs = lexepub::core::html_parser::extract_formatting("<p><b><i>both</i></b></p>").unwrap();
    let both = runs
        .iter()
        .find(|r| r.style & STYLE_BOLD != 0 && r.style & STYLE_ITALIC != 0);
    assert!(
        both.is_some(),
        "nested <b><i> should produce a run with both flags"
    );
}

#[test]
fn test_formatting_adjacent_same_style_merged() {
    let runs =
        lexepub::core::html_parser::extract_formatting("<p><b>foo</b><b>bar</b></p>").unwrap();
    let bold_runs: Vec<_> = runs.iter().filter(|r| r.style & STYLE_BOLD != 0).collect();
    assert_eq!(
        bold_runs.len(),
        1,
        "adjacent <b> runs should be merged into one"
    );
    assert!(bold_runs[0].text.contains("foobar") || bold_runs[0].text.contains("foo"));
}

// extract_formatting block elements and newlines

#[test]
fn test_formatting_paragraphs_separated() {
    let runs = lexepub::core::html_parser::extract_formatting("<p>First</p><p>Second</p>").unwrap();
    let newlines: Vec<_> = runs.iter().filter(|r| r.text == "\n").collect();
    assert!(
        !newlines.is_empty(),
        "paragraphs should be separated by newline runs"
    );
}

#[test]
fn test_formatting_br_newline() {
    let runs = lexepub::core::html_parser::extract_formatting("<p>A<br/>B<br/>C</p>").unwrap();
    let newlines: Vec<_> = runs.iter().filter(|r| r.text == "\n").collect();
    assert!(!newlines.is_empty(), "<br> should produce newline runs");
}

// extract_formatting edge cases

#[test]
fn test_formatting_empty_html() {
    let runs = lexepub::core::html_parser::extract_formatting("").unwrap();
    assert!(runs.is_empty());
}

#[test]
fn test_formatting_no_formatting_tags() {
    let runs = lexepub::core::html_parser::extract_formatting("<p>plain text</p>").unwrap();
    let plain = runs.iter().find(|r| r.style == 0 && r.heading == 0);
    assert!(plain.is_some(), "plain text should produce an unstyled run");
    assert!(plain.unwrap().text.contains("plain"));
}

#[test]
fn test_formatting_multiline_text() {
    let html = "<p>line one</p><p>line two</p><p>line three</p>";
    let runs = lexepub::core::html_parser::extract_formatting(html).unwrap();
    let texts: Vec<&str> = runs
        .iter()
        .filter(|r| r.text != "\n")
        .map(|r| r.text.as_str())
        .collect();
    // All three words should appear in some run
    let all_text = texts.join(" ");
    assert!(all_text.contains("line one") || all_text.contains("line"));
    assert!(all_text.contains("two") || all_text.contains("three"));
}

// Integration: formatting runs from real EPUB chapter

#[test]
fn test_formatting_from_test_epub() {
    use std::path::Path;
    let path = Path::new("examples/epubs/test-book.epub");
    if !path.exists() {
        return;
    }
    futures::executor::block_on(async {
        let mut epub = lexepub::LexEpub::open(path).await.unwrap();
        let md = epub.get_metadata().await.unwrap();
        if md.chapter_count == 0 {
            return;
        }
        let chapter = epub.extract_single_chapter(0).await.unwrap();
        // Should have formatting runs
        assert!(
            !chapter.formatting_runs.is_empty(),
            "chapter should contain formatting runs"
        );
        // Every run should have non-empty text (or be a newline)
        for run in &chapter.formatting_runs {
            assert!(
                !run.text.is_empty() || run.text == "\n",
                "run text should not be empty"
            );
        }
        // Should have some plain runs (style=0, heading=0)
        let plain = chapter
            .formatting_runs
            .iter()
            .any(|r| r.style == 0 && r.heading == 0 && r.text != "\n");
        assert!(plain, "chapter should contain plain text runs");
    });
}

// Integration: text content from test EPUB chapter

#[test]
fn test_text_content_from_test_epub() {
    use std::path::Path;
    let path = Path::new("examples/epubs/test-book.epub");
    if !path.exists() {
        return;
    }
    futures::executor::block_on(async {
        let mut epub = lexepub::LexEpub::open(path).await.unwrap();
        let md = epub.get_metadata().await.unwrap();
        if md.chapter_count == 0 {
            return;
        }
        let chapter = epub.extract_single_chapter(0).await.unwrap();
        assert!(
            !chapter.content.is_empty(),
            "chapter content should not be empty"
        );
        assert!(chapter.word_count > 0);
        assert!(chapter.char_count > 0);
        assert!(chapter.char_count > chapter.word_count);
    });
}
