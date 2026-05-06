use super::Adapter;
use anyhow::Result;
use std::path::Path;

pub struct LibEpubAdapter;

impl Adapter for LibEpubAdapter {
    fn name(&self) -> &'static str {
        "lib-epub"
    }

    fn load(&self, path: &Path) -> Result<()> {
        let _ = lib_epub::epub::EpubDoc::new(path)?;
        Ok(())
    }

    fn metadata(&self, path: &Path) -> Result<()> {
        let doc = lib_epub::epub::EpubDoc::new(path)?;
        let _ = doc.get_title();
        let _ = doc.get_metadata_value("creator");
        let _ = doc.get_metadata_value("language");
        Ok(())
    }

    fn extraction(&self, path: &Path) -> Result<()> {
        let doc = lib_epub::epub::EpubDoc::new(path)?;
        let _ = doc.spine_current();

        // Walk linearly through available spine items.
        for _ in 0..doc.spine.len() {
            let _ = doc.spine_next();
        }
        Ok(())
    }

    fn analysis(&self, path: &Path) -> Result<()> {
        let mut doc = lib_epub::epub::EpubDoc::new(path)?;
        let mut word_count = 0usize;
        let mut char_count = 0usize;

        // Iterate through spine items and accumulate text statistics
        for _ in 0..doc.spine.len() {
            if let Ok(content) = doc.get_current_str() {
                word_count += content.split_whitespace().count();
                char_count += content.chars().count();
            }
            doc.spine_next();
        }

        let _ = (word_count, char_count);
        Ok(())
    }
}
