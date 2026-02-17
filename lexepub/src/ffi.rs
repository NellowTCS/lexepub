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

        pub fn get_total_word_count(&mut self) -> usize {
            self.0.total_word_count_sync().unwrap_or(0)
        }

        pub fn get_total_char_count(&mut self) -> usize {
            self.0.total_char_count_sync().unwrap_or(0)
        }
    }
}
