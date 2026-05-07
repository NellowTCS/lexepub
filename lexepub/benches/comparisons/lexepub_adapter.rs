use super::Adapter;
use anyhow::Result;
use std::path::Path;

pub struct LexEpubAdapter;

impl Adapter for LexEpubAdapter {
    fn name(&self) -> &'static str {
        "lexepub"
    }

    fn load(&self, path: &Path) -> Result<()> {
        let _ = futures::executor::block_on(lexepub::LexEpub::open(path))?;
        Ok(())
    }

    fn metadata(&self, path: &Path) -> Result<()> {
        let mut epub = futures::executor::block_on(lexepub::LexEpub::open(path))?;
        let _ = futures::executor::block_on(epub.get_metadata())?;
        Ok(())
    }

    fn extraction(&self, path: &Path) -> Result<()> {
        let mut epub = futures::executor::block_on(lexepub::LexEpub::open(path))?;
        // Use text-only extraction, cheaper path, no CSS/AST overhead,
        // comparable to what epub-rs / lib-epub do.
        let _ = futures::executor::block_on(epub.extract_text_only())?;
        Ok(())
    }

    fn analysis(&self, path: &Path) -> Result<()> {
        let mut epub = futures::executor::block_on(lexepub::LexEpub::open(path))?;
        // total_char_count() will then return immediately from cache.
        let _ = futures::executor::block_on(epub.total_word_count())?;
        let _ = futures::executor::block_on(epub.total_char_count())?;
        Ok(())
    }
}
