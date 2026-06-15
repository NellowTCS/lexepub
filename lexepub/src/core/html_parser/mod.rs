use crate::core::chapter::FormattingRun;
use crate::error::Result;

#[cfg(feature = "lowmem")]
pub mod streaming;
#[cfg(not(feature = "lowmem"))]
mod tl;

/// Extract cleaned text from an XHTML chapter (all formatting stripped,
/// just the raw words).  Used for word/char counts and search indexes.
#[cfg(feature = "lowmem")]
pub fn extract_text_content(html: &str) -> Result<String> {
    streaming::extract_text_content(html)
}
#[cfg(not(feature = "lowmem"))]
pub fn extract_text_content(html: &str) -> Result<String> {
    tl::extract_text_content(html)
}

/// Extract formatted runs from an XHTML chapter, preserving bold, italic,
/// headings, and code spans.  Used for styled e-ink rendering.
///
/// With `lowmem` the extraction is done via a quick-xml streaming pass
/// (no full DOM tree).  Without `lowmem` it walks the tl DOM AST.
pub fn extract_formatting(html: &str) -> Result<Vec<FormattingRun>> {
    #[cfg(feature = "lowmem")]
    {
        streaming::extract_formatting(html)
    }
    #[cfg(not(feature = "lowmem"))]
    {
        tl::extract_formatting(html)
    }
}

/// Parse HTML into the full AST representation (always uses tl).
#[cfg(not(feature = "lowmem"))]
pub fn parse_html_ast(html: &str) -> Result<crate::core::chapter::AstNode> {
    self::tl::parse_html_ast(html)
}

/// ChapterParser: configurable full-feature parser.
#[cfg(not(feature = "lowmem"))]
pub use tl::ChapterParser;
