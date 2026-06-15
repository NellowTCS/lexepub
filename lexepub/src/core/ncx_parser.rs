use crate::error::{LexEpubError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// A single entry from the NCX `<navMap>`, corresponding to one `<navPoint>`.
///
/// Hierarchical navPoint nesting is flattened: the traversal is depth-first,
/// matching the document order of `<content src="...">` references.
#[derive(Debug, Clone)]
pub struct NcxEntry {
    /// Display title from `<navLabel><text>`
    pub title: String,
    /// Chapter file reference from `<content src="...">`
    pub src: String,
}

/// Parsed NCX document.
#[derive(Debug, Clone)]
pub struct NcxInfo {
    /// Flattened navPoint entries in depth-first order.
    pub entries: Vec<NcxEntry>,
}

/// Parser for EPUB 2 NCX (Navigation Control XML) files.
///
/// The NCX file (`toc.ncx`) holds the canonical table of contents with
/// chapter titles, as authored by the publisher.
///
/// # Structure
///
/// ```xml
/// <ncx xmlns="http://www.dtd.org/2005/ncx" version="2005-1">
///   <head> … </head>
///   <docTitle><text>The Book</text></docTitle>
///   <navMap>
///     <navPoint id="np-1" playOrder="1">
///       <navLabel><text>Chapter One</text></navLabel>
///       <content src="Text/ch01.xhtml"/>
///     </navPoint>
///   </navMap>
/// </ncx>
/// ```
///
/// Hierarchical navPoints (parts containing chapters) are flattened into
/// a depth-first Vec. Entries without a `<content src>` (section headers
/// that group children) are omitted from the output as they have no
/// corresponding chapter file to map against the OPF spine.
pub struct NcxParser;

impl NcxParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse NCX byte data and return a flat list of TOC entries.
    ///
    /// Navigation hierarchy is handled with a parent-state stack so that
    /// a child navPoint's content does not overwrite its parent's state:
    /// when descending into a child, the current (title, src) is pushed;
    /// when returning, the parent's values are restored.
    pub fn parse_ncx(&mut self, data: &[u8]) -> Result<NcxInfo> {
        let mut reader = Reader::from_reader(std::io::Cursor::new(data));
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut entries: Vec<NcxEntry> = Vec::new();

        let mut in_nav_map = false;
        let mut in_nav_label = false;
        let mut nav_point_depth: u32 = 0;

        // State for the navPoint currently being processed
        let mut cur_title = String::new();
        let mut cur_src = String::new();

        // Stack of parent (title, src) saved before descending into a child
        let mut parent_stack: Vec<(String, String)> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    let len = name.len();

                    if len == 6 && name.eq_ignore_ascii_case(b"navmap") {
                        in_nav_map = true;
                    } else if len == 8 && name.eq_ignore_ascii_case(b"navpoint") && in_nav_map {
                        if nav_point_depth > 0 {
                            parent_stack.push((cur_title.clone(), cur_src.clone()));
                        }
                        cur_title.clear();
                        cur_src.clear();
                        nav_point_depth += 1;
                    } else if len == 8
                        && name.eq_ignore_ascii_case(b"navlabel")
                        && nav_point_depth > 0
                    {
                        in_nav_label = true;
                    } else if len == 7
                        && name.eq_ignore_ascii_case(b"content")
                        && nav_point_depth > 0
                    {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"src" {
                                cur_src = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                }

                Ok(Event::Text(ref e)) => {
                    let text = e.decode().unwrap_or_default().to_string();
                    if text.trim().is_empty() {
                        continue;
                    }
                    if in_nav_label && cur_title.is_empty() {
                        cur_title = text;
                    }
                }

                Ok(Event::End(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    let len = name.len();

                    if len == 6 && name.eq_ignore_ascii_case(b"navmap") {
                        in_nav_map = false;
                    } else if len == 8
                        && name.eq_ignore_ascii_case(b"navlabel")
                        && nav_point_depth > 0
                    {
                        in_nav_label = false;
                    } else if len == 8
                        && name.eq_ignore_ascii_case(b"navpoint")
                        && nav_point_depth > 0
                    {
                        nav_point_depth -= 1;

                        if !cur_src.is_empty() {
                            entries.push(NcxEntry {
                                title: cur_title.clone(),
                                src: cur_src.clone(),
                            });
                        }

                        cur_title.clear();
                        cur_src.clear();
                        if let Some((pt, ps)) = parent_stack.pop() {
                            cur_title = pt;
                            cur_src = ps;
                        }
                    }
                }

                Ok(Event::Eof) => break,
                Err(e) => return Err(LexEpubError::Xml(e)),
                _ => buf.clear(),
            }
        }

        Ok(NcxInfo { entries })
    }
}

impl Default for NcxParser {
    fn default() -> Self {
        Self::new()
    }
}
