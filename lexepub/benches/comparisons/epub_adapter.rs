use super::Adapter;
use anyhow::{anyhow, Result};
use std::path::Path;

pub struct EpubAdapter;

impl Adapter for EpubAdapter {
    fn name(&self) -> &'static str {
        "epub-rs"
    }

    fn load(&self, path: &Path) -> Result<()> {
        let _ = epub::doc::EpubDoc::new(path).map_err(|e| anyhow!(e))?;
        Ok(())
    }

    fn metadata(&self, path: &Path) -> Result<()> {
        let doc = epub::doc::EpubDoc::new(path).map_err(|e| anyhow!(e))?;
        let _ = doc.get_title();
        let _ = doc.metadata.len();
        let _ = doc.spine.len();
        let _ = doc.toc.len();
        Ok(())
    }

    fn extraction(&self, path: &Path) -> Result<()> {
        let mut doc = epub::doc::EpubDoc::new(path).map_err(|e| anyhow!(e))?;
        let chapters = doc.get_num_chapters();
        for idx in 0..chapters {
            let _ = doc.set_current_chapter(idx);
            let _ = doc.get_current_str();
        }
        Ok(())
    }

    fn analysis(&self, path: &Path) -> Result<()> {
        let doc = epub::doc::EpubDoc::new(path).map_err(|e| anyhow!(e))?;
        let _ = doc.get_num_chapters();
        let _ = doc.resources.len();
        let _ = doc.metadata.len();
        let _ = doc.toc.len();
        Ok(())
    }
}
