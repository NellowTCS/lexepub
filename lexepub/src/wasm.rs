//! WebAssembly bindings for lexepub

/// TODO: Finish

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::{LexEpub, Result};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmEpubExtractor {
    inner: Option<LexEpub>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmEpubExtractor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: None }
    }

    #[wasm_bindgen]
    pub async fn open_from_bytes(&mut self, data: &[u8]) -> Result<(), JsValue> {
        match LexEpub::from_bytes(data.to_vec().into()).await {
            Ok(extractor) => {
                self.inner = Some(extractor);
                Ok(())
            }
            Err(e) => {
                Err(JsValue::from_str(&format!("Failed to open EPUB: {}", e)))
            }
        }
    }

    #[wasm_bindgen]
    pub async fn get_metadata(&self) -> Result<JsValue, JsValue> {
        match &self.inner {
            Some(extractor) => {
                let metadata = extractor.metadata().await;
                match serde_wasm_bindgen::to_value(&metadata) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(JsValue::from_str(&format!("Serialization error: {}", e))),
                }
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    #[wasm_bindgen]
    pub async fn get_chapter_text(&self, chapter_index: usize) -> Result<String, JsValue> {
        match &self.inner {
            Some(extractor) => {
                let mut chapter_stream = extractor.chapters_stream().await;
                use wasm_bindgen_futures::JsFuture;
                
                for i in 0..chapter_index + 1 {
                    match JsFuture::from(chapter_stream.next()).await {
                        Ok(Some(chapter_result)) => {
                            match chapter_result {
                                Ok(chapter) => {
                                    if chapter.chapter_info.index == chapter_index {
                                        return Ok(chapter.content);
                                    }
                                }
                                Err(_) => {
                                    return Err(JsValue::from_str("Failed to parse chapter"));
                                }
                            }
                        }
                        Ok(None) => {
                            return Err(JsValue::from_str("Chapter not found"));
                        }
                        Err(_) => {
                            return Err(JsValue::from_str("Stream error"));
                        }
                    }
                }
                
                Err(JsValue::from_str("Chapter not found"))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }
}