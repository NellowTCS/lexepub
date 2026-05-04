use super::Adapter;
use anyhow::Result;
use bytes::Bytes;
use std::path::Path;

pub struct LexEpubAdapter;

impl Adapter for LexEpubAdapter {
    fn name(&self) -> &'static str {
        "lexepub"
    }

    fn load(&self, path: &Path) -> Result<()> {
        let data = std::fs::read(path)?;
        let _ = futures::executor::block_on(lexepub::LexEpub::from_bytes(Bytes::from(data)))?;
        Ok(())
    }

    fn metadata(&self, path: &Path) -> Result<()> {
        let data = std::fs::read(path)?;
        let mut epub =
            futures::executor::block_on(lexepub::LexEpub::from_bytes(Bytes::from(data)))?;
        let _ = futures::executor::block_on(epub.get_metadata())?;
        Ok(())
    }

    fn extraction(&self, path: &Path) -> Result<()> {
        let data = std::fs::read(path)?;
        let mut epub =
            futures::executor::block_on(lexepub::LexEpub::from_bytes(Bytes::from(data)))?;
        let _ = futures::executor::block_on(epub.extract_text_only())?;
        Ok(())
    }

    fn analysis(&self, path: &Path) -> Result<()> {
        let data = std::fs::read(path)?;
        let mut epub =
            futures::executor::block_on(lexepub::LexEpub::from_bytes(Bytes::from(data)))?;
        let _ = futures::executor::block_on(epub.total_word_count())?;
        let _ = futures::executor::block_on(epub.total_char_count())?;
        Ok(())
    }
}
