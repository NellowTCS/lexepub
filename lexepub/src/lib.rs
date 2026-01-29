pub mod epub;
pub mod error;
pub mod metadata;
pub mod parser;
pub mod reader;

#[cfg(feature = "c-ffi")]
pub mod ffi;

pub use epub::LexEpub;
pub use error::{LexEpubError, Result};
pub use metadata::EpubMetadata;
pub use parser::{AstNode, Chapter, ChapterParser, ChapterStream, ParsedChapter};
pub use reader::EpubExtractor;

/// Re-export common types for convenience
pub mod prelude {
    pub use crate::epub::LexEpub;
    pub use crate::error::{LexEpubError, Result};
    pub use crate::metadata::EpubMetadata;
    pub use crate::parser::{AstNode, Chapter, ChapterParser, ChapterStream, ParsedChapter};
    pub use crate::reader::EpubExtractor;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
