use crate::error::{LexEpubError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::Cursor;

/// Metadata extracted from OPF file
#[derive(Debug, Clone)]
pub struct OpfMetadata {
    pub title: Option<String>,
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
    pub manifest: HashMap<String, String>,
}

pub struct OpfParser {
    reader: Reader<Cursor<Vec<u8>>>,
}

impl OpfParser {
    /// Create a new OPF parser
    pub fn new() -> Self {
        Self {
            reader: Reader::from_reader(Cursor::new(Vec::new())),
        }
    }

    /// Parse OPF file for metadata
    pub fn parse_metadata(&mut self, data: &[u8]) -> Result<OpfMetadata> {
        self.reader = Reader::from_reader(std::io::Cursor::new(data.to_vec()));
        self.reader.config_mut().trim_text(true);

        let mut metadata = OpfMetadata {
            title: None,
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
        };

        let mut in_metadata = false;
        let mut in_manifest = false;
        let mut in_spine = false;
        let mut current_element = String::new();
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    current_element = tag_name.clone();

                    match current_element.as_str() {
                        "metadata" => in_metadata = true,
                        "manifest" => in_manifest = true,
                        "spine" => in_spine = true,
                        "item" if in_manifest => {
                            let mut id = String::new();
                            let mut href = String::new();
                            for attr in e.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                                    b"href" => {
                                        href = String::from_utf8_lossy(&attr.value).to_string()
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
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
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
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    match tag_name.as_str() {
                        "metadata" => in_metadata = false,
                        "manifest" => in_manifest = false,
                        "spine" => in_spine = false,
                        _ => {}
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
        self.reader = Reader::from_reader(std::io::Cursor::new(data.to_vec()));
        self.reader.config_mut().trim_text(true);

        let mut spine = Vec::new();
        let mut in_spine = false;
        let mut buf = Vec::new();

        loop {
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();

                    if tag_name == "spine" {
                        in_spine = true;
                    } else if tag_name == "itemref" && in_spine {
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
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    if tag_name == "spine" {
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
}

impl Default for OpfParser {
    fn default() -> Self {
        Self::new()
    }
}
