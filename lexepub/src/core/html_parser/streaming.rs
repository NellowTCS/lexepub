use crate::core::chapter::{
    FormattingRun, STYLE_BOLD, STYLE_CODE, STYLE_ITALIC, STYLE_STRIKETHROUGH, STYLE_UNDERLINE,
};
use crate::error::{LexEpubError, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Cursor;

fn tag_is_style_start(name: &[u8]) -> Option<u8> {
    let len = name.len();
    if len == 0 {
        return None;
    }
    let c0 = name[0] | 0x20;
    if len == 1 {
        return match c0 {
            b'b' => Some(STYLE_BOLD),
            b'i' => Some(STYLE_ITALIC),
            b'u' => Some(STYLE_UNDERLINE),
            b's' => Some(STYLE_STRIKETHROUGH),
            _ => None,
        };
    }
    let c1 = name[1] | 0x20;
    if len == 2 {
        return match (c0, c1) {
            (b'e', b'm') => Some(STYLE_ITALIC),
            (b't', b't') => Some(STYLE_CODE),
            _ => None,
        };
    }
    let c2 = name[2] | 0x20;
    if len == 3 {
        return match (c0, c1, c2) {
            (b'd', b'e', b'l') => Some(STYLE_STRIKETHROUGH),
            (b'p', b'r', b'e') => Some(STYLE_CODE),
            _ => None,
        };
    }
    if len == 4 {
        return match (c0, c1, c2, name[3] | 0x20) {
            (b'c', b'o', b'd', b'e') => Some(STYLE_CODE),
            _ => None,
        };
    }
    if len == 5 {
        return match (c0, c1, c2, name[3] | 0x20) {
            (b's', b't', b'r', b'o') if (name[4] | 0x20) == b'n' => Some(STYLE_BOLD),
            (b's', b't', b'r', b'i') if (name[4] | 0x20) == b'k' => Some(STYLE_STRIKETHROUGH),
            _ => None,
        };
    }
    if len == 6 {
        if name.eq_ignore_ascii_case(b"strong") {
            return Some(STYLE_BOLD);
        }
        if name.eq_ignore_ascii_case(b"strike") {
            return Some(STYLE_STRIKETHROUGH);
        }
    }
    None
}

fn is_heading_tag(name: &[u8]) -> Option<u8> {
    if name.len() == 2 && (name[0] | 0x20) == b'h' {
        let d = name[1];
        if d.is_ascii_digit() && d != b'0' {
            return Some(d - b'0');
        }
    }
    None
}

fn is_block_tag(name: &[u8]) -> bool {
    let len = name.len();
    if len == 0 {
        return false;
    }
    let c0 = name[0] | 0x20;
    if len == 1 {
        return c0 == b'p';
    }
    let c1 = name[1] | 0x20;
    if len == 2 {
        if c0 == b'l' && c1 == b'i' {
            return true;
        }
        if c0 == b'b' && c1 == b'r' {
            return true;
        }
        if c0 == b'h' && c1.is_ascii_digit() {
            return true;
        }
        return false;
    }
    if len == 3 {
        return c0 == b'd' && c1 == b'i' && (name[2] | 0x20) == b'v';
    }
    false
}

/// Streaming HTML-to-text extractor using quick-xml StAX parsing.
///
/// Produces a `Vec<FormattingRun>` preserving bold, italic, headings, code spans,
/// and line breaks.
///
/// Unlike the `tl`-based path, this never builds a DOM tree.  The input HTML is
/// consumed in a single pass with no per-event heap allocations.
pub struct FormattingExtractor {
    style_stack: Vec<u8>,
    heading_level: u8,
    runs: Vec<FormattingRun>,
}

impl FormattingExtractor {
    pub fn new() -> Self {
        Self {
            style_stack: Vec::new(),
            heading_level: 0,
            runs: Vec::new(),
        }
    }

    fn active_style(&self) -> u8 {
        self.style_stack.iter().copied().fold(0, |acc, s| acc | s)
    }

    fn push_run(&mut self, text: &str) {
        let style = self.active_style();
        let hl = self.heading_level;
        if text.is_empty() {
            return;
        }
        if let Some(last) = self.runs.last_mut() {
            if last.style == style && last.heading == hl {
                last.text.push_str(text);
                return;
            }
        }
        self.runs.push(FormattingRun {
            text: text.to_string(),
            style,
            heading: hl,
        });
    }

    fn push_newline(&mut self) {
        let style = self.active_style();
        let hl = self.heading_level;
        if let Some(last) = self.runs.last_mut() {
            if last.style == style && last.heading == hl && last.text == "\n" {
                return;
            }
        }
        self.runs.push(FormattingRun {
            text: "\n".into(),
            style,
            heading: hl,
        });
    }

    /// Feed a full HTML string through the streaming parser and collect runs.
    pub fn feed_str(&mut self, html: &str) -> Result<()> {
        let mut reader = Reader::from_reader(Cursor::new(html.as_bytes()));
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    if let Some(s) = tag_is_style_start(name) {
                        self.style_stack.push(s);
                    } else if let Some(h) = is_heading_tag(name) {
                        self.heading_level = h;
                    } else if name.len() == 2
                        && (name[0] | 0x20) == b'b'
                        && (name[1] | 0x20) == b'r'
                    {
                        self.push_newline();
                    }
                }

                Ok(Event::Empty(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    if name.len() == 2 && (name[0] | 0x20) == b'b' && (name[1] | 0x20) == b'r' {
                        self.push_newline();
                    }
                }

                Ok(Event::End(ref e)) => {
                    let qn = e.name();
                    let name = qn.as_ref();
                    if tag_is_style_start(name).is_some() {
                        self.style_stack.pop();
                    } else if is_heading_tag(name).is_some() {
                        self.heading_level = 0;
                        self.push_newline();
                    }
                    if is_block_tag(name) {
                        self.push_newline();
                    }
                }

                Ok(Event::Text(ref e)) => {
                    if let Ok(s) = std::str::from_utf8(e) {
                        if let Ok(decoded) = quick_xml::escape::unescape(s) {
                            self.push_run(&decoded);
                        }
                    }
                }

                Ok(Event::Eof) => break,
                Err(e) => return Err(LexEpubError::Xml(e)),
                _ => buf.clear(),
            }
        }

        Ok(())
    }

    /// Consume self and return the collected formatting runs.
    pub fn finish(self) -> Vec<FormattingRun> {
        let mut runs = self.runs;
        runs.retain(|r| !(r.text.is_empty() || (r.text == " " && r.style == 0 && r.heading == 0)));
        runs
    }
}

/// Extract plain text (no formatting) from HTML using quick-xml.
pub fn extract_text_content(html: &str) -> Result<String> {
    let mut reader = Reader::from_reader(Cursor::new(html.as_bytes()));
    reader.config_mut().trim_text(true);

    let mut out = String::new();
    let mut in_script = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let qn = e.name();
                let name = qn.as_ref();
                let len = name.len();
                in_script = (len == 6 && name.eq_ignore_ascii_case(b"script"))
                    || (len == 5 && name.eq_ignore_ascii_case(b"style"));
                if is_block_tag(name)
                    || (len == 2 && (name[0] | 0x20) == b'b' && (name[1] | 0x20) == b'r')
                {
                    out.push('\n');
                }
            }
            Ok(Event::End(ref e)) => {
                let qn = e.name();
                let name = qn.as_ref();
                in_script = false;
                if is_block_tag(name) {
                    out.push('\n');
                }
            }
            Ok(Event::Text(ref e)) => {
                if !in_script {
                    if let Ok(s) = std::str::from_utf8(e) {
                        if let Ok(decoded) = quick_xml::escape::unescape(s) {
                            out.push_str(&decoded);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(LexEpubError::Xml(e)),
            _ => buf.clear(),
        }
    }

    let cleaned = if out.is_empty() {
        out
    } else {
        let mut bytes = out.into_bytes();
        let mut write = 0usize;
        let mut read = 0usize;
        let mut first = true;
        while read < bytes.len() {
            // Find end of line
            let mut line_end = read;
            while line_end < bytes.len() && bytes[line_end] != b'\n' {
                line_end += 1;
            }
            // Trim trailing whitespace
            let mut trim_end = line_end;
            while trim_end > read && bytes[trim_end - 1] == b' ' {
                trim_end -= 1;
            }
            // Trim leading whitespace
            let mut trim_start = read;
            while trim_start < trim_end && bytes[trim_start] == b' ' {
                trim_start += 1;
            }
            if trim_start < trim_end {
                if !first {
                    bytes[write] = b'\n';
                    write += 1;
                }
                let len = trim_end - trim_start;
                if write != trim_start {
                    bytes.copy_within(trim_start..trim_end, write);
                }
                write += len;
                first = false;
            }
            read = line_end + 1;
        }
        bytes.truncate(write);
        // SAFETY: we only removed whitespace bytes and inserted newlines.
        // The original was valid UTF-8 and all removals preserve UTF-8 validity.
        unsafe { String::from_utf8_unchecked(bytes) }
    };

    Ok(cleaned)
}

/// Extract formatted runs from HTML using quick-xml streaming.
pub fn extract_formatting(html: &str) -> Result<Vec<FormattingRun>> {
    let mut extractor = FormattingExtractor::new();
    extractor.feed_str(html)?;
    Ok(extractor.finish())
}
