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
        let manifest_count = doc.manifest.len();
        let spine_count = doc.spine.len();
        let metadata_count = doc.metadata.len();
        let _ = manifest_count + spine_count + metadata_count;
        Ok(())
    }
}
