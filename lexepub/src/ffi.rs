#[diplomat::bridge]
#[allow(clippy::module_inception)]
mod ffi {
    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct EpubExtractor(Box<crate::LexEpub>);

    impl EpubExtractor {
        pub fn create(path: &str) -> Option<Box<EpubExtractor>> {
            // Run the async `LexEpub::open` synchronously via a Tokio runtime
            let path_buf = std::path::PathBuf::from(path);
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(_) => return None,
            };

            match rt.block_on(crate::LexEpub::open(path_buf)) {
                Ok(lexepub) => Some(Box::new(EpubExtractor(Box::new(lexepub)))),
                Err(_) => None,
            }
        }

        pub fn get_total_word_count(&mut self) -> usize {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(_) => return 0,
            };

            rt.block_on(self.0.total_word_count()).unwrap_or(0)
        }

        pub fn get_total_char_count(&mut self) -> usize {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(_) => return 0,
            };

            rt.block_on(self.0.total_char_count()).unwrap_or(0)
        }
    }
}
