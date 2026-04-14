#[diplomat::bridge]
#[allow(clippy::module_inception)]
mod ffi {
    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct EpubExtractor(Box<crate::LexEpub>);

    impl EpubExtractor {
        pub fn create(path: &str) -> Option<Box<EpubExtractor>> {
            let path_buf = std::path::PathBuf::from(path);
            match crate::LexEpub::open_sync(path_buf) {
                Ok(lexepub) => Some(Box::new(EpubExtractor(Box::new(lexepub)))),
                Err(_) => None,
            }
        }

        pub fn create_from_bytes(data: &[u8]) -> Option<Box<EpubExtractor>> {
            let bytes = bytes::Bytes::copy_from_slice(data);
            match futures::executor::block_on(crate::LexEpub::from_bytes(bytes)) {
                Ok(lexepub) => Some(Box::new(EpubExtractor(Box::new(lexepub)))),
                Err(_) => None,
            }
        }

        pub fn get_metadata_is_valid(&mut self) -> bool {
            self.0.validate_metadata_sync().is_ok()
        }

        pub fn get_chapter_count(&mut self) -> usize {
            self.0
                .get_metadata_sync()
                .map(|m| m.chapter_count)
                .unwrap_or(0)
        }

        pub fn get_title(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            use core::fmt::Write as _;
            let title = self
                .0
                .get_metadata_sync()
                .ok()
                .and_then(|m| m.title)
                .unwrap_or_default();
            to.write_str(&title).map_err(|_| ())
        }

        pub fn get_metadata_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            use core::fmt::Write as _;
            let json = match self.0.get_metadata_sync() {
                Ok(metadata) => serde_json::to_string(&metadata).map_err(|_| ())?,
                Err(_) => return Err(()),
            };
            to.write_str(&json).map_err(|_| ())
        }

        pub fn get_metadata(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            self.get_metadata_json(to)
        }

        pub fn get_chapters_text_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            use core::fmt::Write as _;
            let json = match futures::executor::block_on(self.0.extract_text_only()) {
                Ok(chapters) => serde_json::to_string(&chapters).map_err(|_| ())?,
                Err(_) => return Err(()),
            };
            to.write_str(&json).map_err(|_| ())
        }

        pub fn get_chapters_text(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            self.get_chapters_text_json(to)
        }

        pub fn get_chapter_text(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            use core::fmt::Write as _;
            let text = match futures::executor::block_on(self.0.extract_text_only()) {
                Ok(chapters) => chapters.get(index).cloned().ok_or(())?,
                Err(_) => return Err(()),
            };
            to.write_str(&text).map_err(|_| ())
        }

        pub fn get_chapter_json(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            use core::fmt::Write as _;
            let json = match futures::executor::block_on(self.0.extract_ast()) {
                Ok(chapters) => {
                    let chapter = chapters.get(index).ok_or(())?;
                    serde_json::to_string(chapter).map_err(|_| ())?
                }
                Err(_) => return Err(()),
            };
            to.write_str(&json).map_err(|_| ())
        }

        pub fn get_chapter(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            self.get_chapter_json(index, to)
        }

        pub fn get_total_word_count(&mut self) -> usize {
            self.0.total_word_count_sync().unwrap_or(0)
        }

        pub fn get_total_char_count(&mut self) -> usize {
            self.0.total_char_count_sync().unwrap_or(0)
        }

        pub fn has_cover(&mut self) -> bool {
            self.0.has_cover_sync().unwrap_or(false)
        }

        pub fn get_cover_image_len(&mut self) -> usize {
            self.0
                .cover_image_sync()
                .map(|bytes| bytes.len())
                .unwrap_or(0)
        }

        pub fn get_cover_image_format(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            use core::fmt::Write as _;
            let mime = self
                .0
                .get_metadata_sync()
                .ok()
                .and_then(|m| m.cover_image_format)
                .unwrap_or_default();
            to.write_str(&mime).map_err(|_| ())
        }

        pub fn get_cover_image_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            use core::fmt::Write as _;
            let json = match self.0.cover_image_sync() {
                Ok(bytes) => serde_json::to_string(&bytes).map_err(|_| ())?,
                Err(_) => return Err(()),
            };
            to.write_str(&json).map_err(|_| ())
        }
    }
}
