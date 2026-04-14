//! WebAssembly bindings for lexepub

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::LexEpub;
use js_sys::Uint8Array;

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
    pub async fn load_from_bytes(&mut self, data: Uint8Array) -> std::result::Result<(), JsValue> {
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
    pub async fn get_metadata(&mut self) -> std::result::Result<JsValue, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let metadata = extractor
                    .get_metadata()
                    .await
                    .map_err(|e| JsValue::from_str(&format!("Failed to get metadata: {}", e)))?;
                serde_wasm_bindgen::to_value(&metadata)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Validate EPUB metadata against required constraints
    #[wasm_bindgen]
    pub async fn get_metadata_is_valid(&mut self) -> std::result::Result<bool, JsValue> {
        match &mut self.inner {
            Some(extractor) => Ok(extractor.validate_metadata().await.is_ok()),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get metadata serialized as JSON string
    #[wasm_bindgen]
    pub async fn get_metadata_json(&mut self) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let metadata = extractor
                    .get_metadata()
                    .await
                    .map_err(|e| JsValue::from_str(&format!("Failed to get metadata: {}", e)))?;
                serde_json::to_string(&metadata)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get chapter count from metadata
    #[wasm_bindgen]
    pub async fn get_chapter_count(&mut self) -> std::result::Result<usize, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .get_metadata()
                .await
                .map(|m| m.chapter_count)
                .map_err(|e| JsValue::from_str(&format!("Failed to get chapter count: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get table of contents entries
    #[wasm_bindgen]
    pub async fn get_toc(&mut self) -> std::result::Result<JsValue, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let toc = extractor
                    .get_toc()
                    .await
                    .map_err(|e| JsValue::from_str(&format!("Failed to get TOC: {}", e)))?;
                serde_wasm_bindgen::to_value(&toc)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get table of contents entries serialized as JSON
    #[wasm_bindgen]
    pub async fn get_toc_json(&mut self) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let toc = extractor
                    .get_toc()
                    .await
                    .map_err(|e| JsValue::from_str(&format!("Failed to get TOC: {}", e)))?;
                serde_json::to_string(&toc)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Resolve a chapter-relative href into a normalized internal EPUB path
    #[wasm_bindgen]
    pub async fn resolve_chapter_resource_path(
        &mut self,
        chapter_index: usize,
        href: String,
    ) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .resolve_chapter_resource_path(chapter_index, &href)
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to resolve resource path: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Read a chapter-relative resource as bytes (for images, linked assets)
    #[wasm_bindgen]
    pub async fn get_chapter_resource(
        &mut self,
        chapter_index: usize,
        href: String,
    ) -> std::result::Result<Uint8Array, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let bytes = extractor
                    .read_chapter_resource(chapter_index, &href)
                    .await
                    .map_err(|e| {
                        JsValue::from_str(&format!("Failed to read chapter resource: {}", e))
                    })?;
                Ok(Uint8Array::from(&bytes[..]))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Read an internal EPUB resource as bytes by normalized path
    #[wasm_bindgen]
    pub async fn get_resource(&mut self, path: String) -> std::result::Result<Uint8Array, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let bytes = extractor
                    .read_resource(&path)
                    .await
                    .map_err(|e| JsValue::from_str(&format!("Failed to read resource: {}", e)))?;
                Ok(Uint8Array::from(&bytes[..]))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get title string from metadata
    #[wasm_bindgen]
    pub async fn get_title(&mut self) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .get_metadata()
                .await
                .map(|m| m.title.unwrap_or_default())
                .map_err(|e| JsValue::from_str(&format!("Failed to get title: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get all chapters as text array
    #[wasm_bindgen]
    pub async fn get_chapters_text(&mut self) -> std::result::Result<JsValue, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let chapters = extractor.extract_text_only().await.map_err(|e| {
                    JsValue::from_str(&format!("Failed to extract chapters: {}", e))
                })?;
                serde_wasm_bindgen::to_value(&chapters)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get all chapter text serialized as JSON string
    #[wasm_bindgen]
    pub async fn get_chapters_text_json(&mut self) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let chapters = extractor.extract_text_only().await.map_err(|e| {
                    JsValue::from_str(&format!("Failed to extract chapters: {}", e))
                })?;
                serde_json::to_string(&chapters)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get chapter by index
    #[wasm_bindgen]
    pub async fn get_chapter(&mut self, index: usize) -> std::result::Result<JsValue, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let chapters = extractor.extract_ast().await.map_err(|e| {
                    JsValue::from_str(&format!("Failed to extract chapters: {}", e))
                })?;

                if index >= chapters.len() {
                    return Err(JsValue::from_str("Chapter index out of bounds"));
                }

                serde_wasm_bindgen::to_value(&chapters[index])
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get chapter by index serialized as JSON string
    #[wasm_bindgen]
    pub async fn get_chapter_json(&mut self, index: usize) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let chapters = extractor.extract_ast().await.map_err(|e| {
                    JsValue::from_str(&format!("Failed to extract chapters: {}", e))
                })?;

                if index >= chapters.len() {
                    return Err(JsValue::from_str("Chapter index out of bounds"));
                }

                serde_json::to_string(&chapters[index])
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get chapter text by index
    #[wasm_bindgen]
    pub async fn get_chapter_text(&mut self, index: usize) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let chapters = extractor.extract_text_only().await.map_err(|e| {
                    JsValue::from_str(&format!("Failed to extract chapters: {}", e))
                })?;

                chapters
                    .get(index)
                    .cloned()
                    .ok_or_else(|| JsValue::from_str("Chapter index out of bounds"))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get total word count
    #[wasm_bindgen]
    pub async fn get_total_word_count(&mut self) -> std::result::Result<usize, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .total_word_count()
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to count words: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get total character count
    #[wasm_bindgen]
    pub async fn get_total_char_count(&mut self) -> std::result::Result<usize, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .total_char_count()
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to count characters: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Check if EPUB has a cover image
    #[wasm_bindgen]
    pub async fn has_cover(&mut self) -> std::result::Result<bool, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .has_cover()
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to check cover: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get cover image as byte array
    #[wasm_bindgen]
    pub async fn get_cover_image(&mut self) -> std::result::Result<Uint8Array, JsValue> {
        match &mut self.inner {
            Some(extractor) => {
                let cover_data = extractor
                    .cover_image()
                    .await
                    .map_err(|e| JsValue::from_str(&format!("Failed to get cover: {}", e)))?;

                Ok(Uint8Array::from(&cover_data[..]))
            }
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get cover image byte length
    #[wasm_bindgen]
    pub async fn get_cover_image_len(&mut self) -> std::result::Result<usize, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .cover_image()
                .await
                .map(|bytes| bytes.len())
                .map_err(|e| JsValue::from_str(&format!("Failed to get cover length: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }

    /// Get cover image MIME format from metadata
    #[wasm_bindgen]
    pub async fn get_cover_image_format(&mut self) -> std::result::Result<String, JsValue> {
        match &mut self.inner {
            Some(extractor) => extractor
                .get_metadata()
                .await
                .map(|m| m.cover_image_format.unwrap_or_default())
                .map_err(|e| JsValue::from_str(&format!("Failed to get cover format: {}", e))),
            None => Err(JsValue::from_str("No EPUB loaded")),
        }
    }
}
