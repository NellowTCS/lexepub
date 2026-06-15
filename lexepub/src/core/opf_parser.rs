use crate::error::{LexEpubError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;

/// Metadata extracted from OPF file
#[derive(Debug, Clone)]
pub struct OpfMetadata {
    pub title: Option<String>,
    pub version: Option<String>,
    pub creators: Vec<String>,
    pub description: Option<String>,
    pub languages: Vec<String>,
    pub subjects: Vec<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub identifiers: Vec<String>,
    pub rights: Option<String>,
    pub contributors: Vec<String>,
    pub spine: Vec<String>,
    pub manifest: HashMap<String, (String, String)>,
    pub cover_image_id: Option<String>,
}

pub struct OpfParser;

impl OpfParser {
    /// Create a new OPF parser
    pub fn new() -> Self {
        Self
    }

    /// Parse OPF file for metadata
    pub fn parse_metadata(&mut self, data: &[u8]) -> Result<OpfMetadata> {
        let mut reader = Reader::from_reader(std::io::Cursor::new(data));
        reader.config_mut().trim_text(true);

        let mut metadata = OpfMetadata {
            title: None,
            version: None,
            creators: Vec::new(),
            description: None,
            languages: Vec::new(),
            subjects: Vec::new(),
            publisher: None,
            date: None,
            identifiers: Vec::new(),
            rights: None,
            contributors: Vec::new(),
            spine: Vec::new(),
            manifest: HashMap::new(),
            cover_image_id: None,
        };

        let mut in_metadata = false;
        let mut in_manifest = false;
        let mut in_spine = false;
        let mut current_element = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    let len = name.len();
                    let is_package = len == 7 && name.eq_ignore_ascii_case(b"package");
                    if is_package {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"version" {
                                metadata.version =
                                    Some(String::from_utf8_lossy(&attr.value).to_string());
                                break;
                            }
                        }
                    }

                    current_element.clear();
                    if len == 4 && name.eq_ignore_ascii_case(b"meta") {
                        current_element = "meta".to_string();
                        if in_metadata {
                            let mut name_attr = String::new();
                            let mut content = String::new();
                            for attr in e.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"name" => {
                                        name_attr = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    b"content" => {
                                        content = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    _ => {}
                                }
                            }
                            if name_attr == "cover"
                                && !content.is_empty()
                                && metadata.cover_image_id.is_none()
                            {
                                metadata.cover_image_id = Some(content);
                            }
                        }
                    } else if len == 8 && name.eq_ignore_ascii_case(b"metadata") {
                        in_metadata = true;
                    } else if len == 4 && name.eq_ignore_ascii_case(b"item") && in_manifest {
                        let mut id = String::new();
                        let mut href = String::new();
                        let mut media_type = String::new();
                        let mut is_cover = false;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                                b"href" => href = String::from_utf8_lossy(&attr.value).to_string(),
                                b"media-type" => {
                                    media_type = String::from_utf8_lossy(&attr.value).to_string()
                                }
                                b"properties" => {
                                    let props = String::from_utf8_lossy(&attr.value);
                                    if props.contains("cover-image") {
                                        is_cover = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if !id.is_empty() && !href.is_empty() {
                            if is_cover {
                                metadata.cover_image_id = Some(id.clone());
                            }
                            metadata.manifest.insert(id, (href, media_type));
                        }
                    } else if len == 7 && name.eq_ignore_ascii_case(b"itemref") && in_spine {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"idref" {
                                let idref = String::from_utf8_lossy(&attr.value).to_string();
                                metadata.spine.push(idref);
                                break;
                            }
                        }
                    } else if len == 8 && name.eq_ignore_ascii_case(b"manifest") {
                        in_manifest = true;
                    } else if len == 5 && name.eq_ignore_ascii_case(b"spine") {
                        in_spine = true;
                    } else if in_metadata {
                        // Track element name for text content matching
                        current_element = String::from_utf8_lossy(name).to_string();
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.decode().unwrap_or_default().to_string();
                    if text.trim().is_empty() {
                        continue;
                    }

                    if in_metadata {
                        match current_element.as_str() {
                            "dc:title" | "title" => {
                                metadata.title = Some(text);
                            }
                            "dc:creator" | "creator" => {
                                metadata.creators.push(text);
                            }
                            "dc:description" | "description" => {
                                metadata.description = Some(text);
                            }
                            "dc:language" | "language" => {
                                metadata.languages.push(text);
                            }
                            "dc:subject" | "subject" => {
                                metadata.subjects.push(text);
                            }
                            "dc:publisher" | "publisher" => {
                                metadata.publisher = Some(text);
                            }
                            "dc:date" | "date" => {
                                metadata.date = Some(text);
                            }
                            "dc:identifier" | "identifier" => {
                                metadata.identifiers.push(text);
                            }
                            "dc:rights" | "rights" => {
                                metadata.rights = Some(text);
                            }
                            "dc:contributor" | "contributor" => {
                                metadata.contributors.push(text);
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    let len = name.len();
                    if len == 8 && name.eq_ignore_ascii_case(b"metadata") {
                        in_metadata = false;
                    } else if len == 8 && name.eq_ignore_ascii_case(b"manifest") {
                        in_manifest = false;
                    } else if len == 5 && name.eq_ignore_ascii_case(b"spine") {
                        in_spine = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(LexEpubError::Xml(e)),
                _ => buf.clear(),
            }
        }

        Ok(metadata)
    }

    /// Parse spine from OPF data
    pub fn parse_spine(&mut self, data: &[u8]) -> Result<Vec<String>> {
        let mut reader = Reader::from_reader(std::io::Cursor::new(data));
        reader.config_mut().trim_text(true);

        let mut spine = Vec::new();
        let mut in_spine = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    let len = name.len();
                    if len == 5 && name.eq_ignore_ascii_case(b"spine") {
                        in_spine = true;
                    } else if len == 7 && name.eq_ignore_ascii_case(b"itemref") && in_spine {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"idref" {
                                let idref = String::from_utf8_lossy(&attr.value).to_string();
                                spine.push(idref);
                                break;
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    if name.len() == 5 && name.eq_ignore_ascii_case(b"spine") {
                        in_spine = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(LexEpubError::Xml(e)),
                _ => buf.clear(),
            }
        }

        Ok(spine)
    }

    /// Get the cover image item ID from OPF metadata
    pub fn get_cover_image_id(&mut self, data: &[u8]) -> Result<Option<String>> {
        let metadata = self.parse_metadata(data)?;
        Ok(metadata.cover_image_id)
    }
}

impl Default for OpfParser {
    fn default() -> Self {
        Self::new()
    }
}
