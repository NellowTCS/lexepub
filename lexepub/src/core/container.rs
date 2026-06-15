use crate::error::{LexEpubError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Debug)]
pub struct ContainerInfo {
    pub rootfile_path: String,
}

pub struct ContainerParser;

impl ContainerParser {
    /// Create a new container parser
    pub fn new() -> Self {
        Self
    }

    /// Parse container.xml to find rootfile path
    pub fn parse_container(&mut self, data: &[u8]) -> Result<ContainerInfo> {
        let mut reader = Reader::from_reader(std::io::Cursor::new(data));
        reader.config_mut().trim_text(true);

        let mut rootfile_path: Option<String> = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qn = e.name();
                    if qn.as_ref() == b"rootfile" {
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
