//! WebAssembly bindings for lexepub

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::Uint8Array;
use crate::{LexEpub, Result, EpubMetadata, ParsedChapter};
use futures::StreamExt;

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

    /// Load EPUB from byte array
    #[wasm_bindgen]
    pub async fn load_from_bytes(&mut self, data: Uint8Array) -> Result<(), JsValue> {
        let bytes = data.to_vec();
        match LexEpub::from_bytes(bytes.into()).await {
            Ok(extractor) => {
                self.inner = Some(extractor);
                Ok(())
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to load EPUB: {}", e))),
        }
    }

    /// Get EPUB metadata as JSON
    #[wasm_bindgen]
    pub async fn get_metadata(&mut self) -> Result<JsValue, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let metadata = extractor.get_metadata().await
                    .map_err(|e| JsValue::from_str(&format!("Failed to get metadata: {}", e)))?;
                serde_wasm_bindgen::to_value(&metadata)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get all chapters as text array
    #[wasm_bindgen]
    pub async fn get_chapters_text(&mut self) -> Result<JsValue, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let chapters = extractor.extract_text_only().await
                    .map_err(|e| JsValue::from_str(&format!("Failed to extract chapters: {}", e)))?;
                serde_wasm_bindgen::to_value(&chapters)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get chapter by index
    #[wasm_bindgen]
    pub async fn get_chapter(&mut self, index: usize) -> Result<JsValue, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let chapters = extractor.extract_with_ast().await  // TODO: change to extract_ast(), method doesn't exist
                    .map_err(|e| JsValue::from_str(&format!("Failed to extract chapters: {}", e)))?;
                
                if index >= chapters.len() {
                    return Err(JsValue::from_str("Chapter index out of bounds"));
                }
                
                serde_wasm_bindgen::to_value(&chapters[index])
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get total word count
    #[wasm_bindgen]
    pub async fn get_total_word_count(&mut self) -> Result<usize, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                extractor.total_word_count().await
                    .map_err(|e| JsValue::from_str(&format!("Failed to count words: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get total character count
    #[wasm_bindgen]
    pub async fn get_total_char_count(&mut self) -> Result<usize, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                extractor.total_char_count().await
                    .map_err(|e| JsValue::from_str(&format!("Failed to count characters: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Check if EPUB has a cover image
    #[wasm_bindgen]
    pub async fn has_cover(&mut self) -> Result<bool, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                extractor.has_cover().await  // TODO: implement has_cover method on LexEpub
                    .map_err(|e| JsValue::from_str(&format!("Failed to check cover: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get cover image as byte array
    #[wasm_bindgen]
    pub async fn get_cover_image(&mut self) -> Result<Uint8Array, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let cover_data = extractor.cover_image().await  // TODO: implement cover_image method on LexEpub
                    .map_err(|e| JsValue::from_str(&format!("Failed to get cover: {}", e)))?;
                
                match cover_data {
                    Some(data) => Ok(Uint8Array::from(&data[..])),
                    None => Err(JsValue::from_str("No cover image found")),
                }
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }
}