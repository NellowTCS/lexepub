#[diplomat::bridge]
#[allow(clippy::module_inception)]
mod ffi {
    use core::fmt::Write as _;

    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct EpubExtractor(Box<crate::LexEpub>);

    impl EpubExtractor {
        fn write_string(to: &mut diplomat_runtime::DiplomatWriteable, s: &str) -> Result<(), ()> {
            to.write_str(s).map_err(|_| ())
        }

        fn write_json<T: serde::Serialize>(
            to: &mut diplomat_runtime::DiplomatWriteable,
            value: &T,
        ) -> Result<(), ()> {
            let json = serde_json::to_string(value).map_err(|_| ())?;
            Self::write_string(to, &json)
        }

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
            let title = self
                .0
                .get_metadata_sync()
                .ok()
                .and_then(|m| m.title)
                .unwrap_or_default();
            Self::write_string(to, &title)
        }

        pub fn get_metadata_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            let metadata = self.0.get_metadata_sync().map_err(|_| ())?;
            Self::write_json(to, &metadata)
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
            let chapters =
                futures::executor::block_on(self.0.extract_text_only()).map_err(|_| ())?;
            Self::write_json(to, &chapters)
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
            let text = match futures::executor::block_on(self.0.extract_text_only()) {
                Ok(chapters) => chapters.get(index).cloned().ok_or(())?,
                Err(_) => return Err(()),
            };
            Self::write_string(to, &text)
        }

        pub fn get_chapter_json(
            &mut self,
            index: usize,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            let chapter = match futures::executor::block_on(self.0.extract_ast()) {
                Ok(chapters) => chapters.get(index).cloned().ok_or(())?,
                Err(_) => return Err(()),
            };
            Self::write_json(to, &chapter)
        }

        pub fn get_toc_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            let toc = futures::executor::block_on(self.0.get_toc()).map_err(|_| ())?;
            Self::write_json(to, &toc)
        }

        pub fn get_toc(&mut self, to: &mut diplomat_runtime::DiplomatWriteable) -> Result<(), ()> {
            self.get_toc_json(to)
        }

        pub fn resolve_chapter_resource_path(
            &mut self,
            chapter_index: usize,
            href: &str,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            let resolved = futures::executor::block_on(
                self.0.resolve_chapter_resource_path(chapter_index, href),
            )
            .map_err(|_| ())?;
            Self::write_string(to, &resolved)
        }

        pub fn get_resource_json(
            &mut self,
            path: &str,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            let bytes = futures::executor::block_on(self.0.read_resource(path)).map_err(|_| ())?;
            Self::write_json(to, &bytes)
        }

        pub fn get_chapter_resource_json(
            &mut self,
            chapter_index: usize,
            href: &str,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            let bytes =
                futures::executor::block_on(self.0.read_chapter_resource(chapter_index, href))
                    .map_err(|_| ())?;
            Self::write_json(to, &bytes)
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
            let mime = self
                .0
                .get_metadata_sync()
                .ok()
                .and_then(|m| m.cover_image_format)
                .unwrap_or_default();
            Self::write_string(to, &mime)
        }

        pub fn get_cover_image_json(
            &mut self,
            to: &mut diplomat_runtime::DiplomatWriteable,
        ) -> Result<(), ()> {
            let bytes = self.0.cover_image_sync().map_err(|_| ())?;
            Self::write_json(to, &bytes)
        }
    }
}
