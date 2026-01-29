use crate::error::{LexEpubError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct EpubMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub languages: Vec<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<String>,
    pub identifiers: HashMap<String, String>,
    pub cover_image: Option<String>,
    pub spine: Vec<String>,
    pub manifest: HashMap<String, String>,
    pub toc: Vec<String>,
    pub metadata_map: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ContainerInfo {
    pub rootfile_path: String,
}

pub struct MetadataParser {
    reader: Reader<Cursor<Vec<u8>>>,
}

impl MetadataParser {
    /// Create a new metadata parser
    pub fn new() -> Self {
        Self {
            reader: Reader::from_reader(Cursor::new(Vec::new())),
        }
    }

    /// Parse container.xml to find rootfile path
    pub fn parse_container(&mut self, data: &[u8]) -> Result<ContainerInfo> {
        self.reader = Reader::from_reader(std::io::Cursor::new(data.to_vec()));
        self.reader.config_mut().trim_text(true);

        let mut rootfile_path = None;
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    if e.name().as_ref() == b"rootfile" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"full-path" {
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                rootfile_path = Some(value);
                                break;
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(LexEpubError::Xml(e)),
                _ => buf.clear(),
            }
        }

        let rootfile_path = rootfile_path.ok_or_else(|| {
            LexEpubError::InvalidFormat("No rootfile found in container.xml".to_string())
        })?;

        Ok(ContainerInfo { rootfile_path })
    }

    /// Parse OPF file for complete metadata
    pub fn parse_opf(&mut self, data: &[u8]) -> Result<EpubMetadata> {
        self.reader = Reader::from_reader(std::io::Cursor::new(data.to_vec()));
        self.reader.config_mut().trim_text(true);

        let mut metadata = EpubMetadata {
            title: String::new(),
            authors: Vec::new(),
            languages: Vec::new(),
            publisher: None,
            publication_date: None,
            identifiers: HashMap::new(),
            cover_image: None,
            spine: Vec::new(),
            manifest: HashMap::new(),
            toc: Vec::new(),
            metadata_map: HashMap::new(),
        };

        let mut in_metadata = false;
        let mut in_manifest = false;
        let mut in_spine = false;
        let mut current_element = String::new();
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = e.name();
                    current_element = String::from_utf8_lossy(tag_name.as_ref()).to_lowercase();

                    match current_element.as_str() {
                        "metadata" => in_metadata = true,
                        "manifest" => in_manifest = true,
                        "spine" => {
                            in_spine = true;
                            // Process toc attribute if present
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"toc" {
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    metadata.toc.push(value);
                                }
                            }
                        }
                        "item" if in_manifest => {
                            let mut id = String::new();
                            let mut href = String::new();

                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                                let value = String::from_utf8_lossy(&attr.value).to_string();

                                match key.as_str() {
                                    "id" => id = value,
                                    "href" => href = value,
                                    "media-type" => {
                                        // Media type read but not used for now
                                    }
                                    _ => {}
                                }
                            }

                            if !id.is_empty() && !href.is_empty() {
                                metadata.manifest.insert(id, href);
                            }
                        }
                        "itemref" if in_spine => {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"idref" {
                                    let idref = String::from_utf8_lossy(&attr.value).to_string();
                                    metadata.spine.push(idref);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text_owned = std::string::String::from_utf8_lossy(e);
                    let text_str = text_owned.trim();

                    if in_metadata {
                        match current_element.as_str() {
                            "title" if !text_str.is_empty() => {
                                if metadata.title.is_empty() {
                                    metadata.title = text_str.to_string();
                                }
                            }
                            "creator" if !text_str.is_empty() => {
                                metadata.authors.push(text_str.to_string());
                            }
                            "language" if !text_str.is_empty() => {
                                metadata.languages.push(text_str.to_string());
                            }
                            "publisher" if !text_str.is_empty() => {
                                metadata.publisher = Some(text_str.to_string());
                            }
                            "date" if !text_str.is_empty() => {
                                metadata.publication_date = Some(text_str.to_string());
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = e.name();
                    let tag = String::from_utf8_lossy(tag_name.as_ref()).to_lowercase();

                    match tag.as_str() {
                        "metadata" => in_metadata = false,
                        "manifest" => in_manifest = false,
                        "spine" => in_spine = false,
                        _ => {}
                    }
                    current_element.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(LexEpubError::Xml(e)),
                _ => buf.clear(),
            }
        }

        // Find cover image in metadata
        for (key, value) in &metadata.metadata_map {
            if key.contains("cover") || value.to_lowercase().contains("cover") {
                metadata.cover_image = Some(value.clone());
                break;
            }
        }

        Ok(metadata)
    }
}

impl Default for MetadataParser {
    fn default() -> Self {
        Self::new()
    }
}
