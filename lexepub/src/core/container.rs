use crate::error::{LexEpubError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::Cursor;

#[derive(Debug)]
pub struct ContainerInfo {
    pub rootfile_path: String,
}

pub struct ContainerParser {
    reader: Reader<Cursor<Vec<u8>>>,
}

impl ContainerParser {
    /// Create a new container parser
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
}

impl Default for ContainerParser {
    fn default() -> Self {
        Self::new()
    }
}
