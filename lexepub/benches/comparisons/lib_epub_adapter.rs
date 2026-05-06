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
        let doc = lib_epub::epub::EpubDoc::new(path)?;
        let mut word_count = 0usize;
        let mut char_count = 0usize;

        // Iterate the spine and read each resource via navigate_by_spine_index,
        // converting the returned bytes to text and accumulating the same
        // word/char metrics used by other adapters.
        for i in 0..doc.spine.len() {
            if let Some((data, _mime)) = doc.navigate_by_spine_index(i) {
                let text = String::from_utf8_lossy(&data);
                word_count += text.split_whitespace().count();
                char_count += text.chars().count();
            }
        }

        let _ = (word_count, char_count);
        Ok(())
    }
}
