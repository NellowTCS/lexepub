use super::Adapter;
use anyhow::Result;
use std::path::Path;

pub struct EpubieLibAdapter;

impl Adapter for EpubieLibAdapter {
    fn name(&self) -> &'static str {
        "epubie-lib"
    }

    fn load(&self, path: &Path) -> Result<()> {
        epubie_lib::Epub::new(path.display().to_string())
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn metadata(&self, path: &Path) -> Result<()> {
        let epub = epubie_lib::Epub::new(path.display().to_string())
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let _ = epub.get_title();
        let _ = epub.get_creator();
        let _ = epub.get_language();
        Ok(())
    }

    fn extraction(&self, path: &Path) -> Result<()> {
        let epub = epubie_lib::Epub::new(path.display().to_string())
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        for chapter in epub.get_chapters() {
            let _ = chapter.get_title();
            let _ = chapter.get_file_count();
            for file in chapter.get_files() {
                let _ = file.get_content();
            }
        }
        Ok(())
    }

    fn analysis(&self, path: &Path) -> Result<()> {
        let epub = epubie_lib::Epub::new(path.display().to_string())
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let mut word_count = 0usize;
        let mut char_count = 0usize;
        for chapter in epub.get_chapters() {
            for file in chapter.get_files() {
                let content = file.get_content();
                word_count += content.split_whitespace().count();
                char_count += content.chars().count();
            }
        }
        let _ = (word_count, char_count);
        Ok(())
    }
}
