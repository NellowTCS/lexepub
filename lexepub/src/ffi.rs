#[diplomat::bridge]
#[allow(clippy::module_inception)]
mod ffi {
    #[diplomat::opaque]
    #[allow(dead_code)]
    pub struct EpubExtractor(Box<crate::LexEpub>);

    impl EpubExtractor {
        pub fn create() -> Box<EpubExtractor> {
            // TODO: Create a dummy extractor for now, I need sync API
            Box::new(EpubExtractor(Box::new(unsafe { std::mem::zeroed() })))
        }

        pub fn get_total_word_count(&self) -> usize {
            // Placeholder, would need sync API
            0
        }

        pub fn get_total_char_count(&self) -> usize {
            // Placeholder, would need sync API
            0
        }
    }
}
