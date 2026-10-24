//! Lossless Java text.
//!
//! Modified UTF-8 encodes UTF-16 code units. Some of those code units can be
//! unpaired surrogates, which a Rust `String` cannot represent. `JavaText`
//! stores the code units directly and converts to `String` only when safe.

use std::fmt;

use super::error::{ParseError, ParseErrorKind, ParseResult};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct JavaText(Box<[u16]>);

impl JavaText {
    pub(crate) fn from_modified_utf8(bytes: &[u8], file_offset: usize) -> ParseResult<Self> {
        let mut units = Vec::with_capacity(bytes.len());
        let mut i = 0;

        while i < bytes.len() {
            let b0 = bytes[i];
            match b0 {
                0x01..=0x7F => {
                    units.push(b0 as u16);
                    i += 1;
                }
                0xC0 => {
                    if bytes.get(i + 1) != Some(&0x80) {
                        return Err(invalid_text(file_offset, bytes));
                    }
                    units.push(0);
                    i += 2;
                }
                0xC2..=0xDF => {
                    let b1 = continuation(bytes, i + 1, file_offset)?;
                    units.push((((b0 & 0x1F) as u16) << 6) | ((b1 & 0x3F) as u16));
                    i += 2;
                }
                0xE0..=0xEF => {
                    let b1 = continuation(bytes, i + 1, file_offset)?;
                    let b2 = continuation(bytes, i + 2, file_offset)?;
                    let unit = (((b0 & 0x0F) as u16) << 12)
                        | (((b1 & 0x3F) as u16) << 6)
                        | ((b2 & 0x3F) as u16);
                    if unit < 0x0800 {
                        return Err(invalid_text(file_offset, bytes));
                    }
                    // Surrogate code units are valid in modified UTF-8.
                    units.push(unit);
                    i += 3;
                }
                _ => return Err(invalid_text(file_offset, bytes)),
            }
        }

        Ok(Self(units.into_boxed_slice()))
    }

    pub(crate) fn from_str(s: &str) -> Self {
        Self(s.encode_utf16().collect())
    }

    pub(crate) fn from_utf16(units: Vec<u16>) -> Self {
        Self(units.into_boxed_slice())
    }

    pub(crate) fn as_utf16(&self) -> &[u16] {
        &self.0
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `None` if the code units contain an unpaired surrogate.
    pub(crate) fn to_rust_string(&self) -> Option<String> {
        String::from_utf16(&self.0).ok()
    }

    pub(crate) fn eq_str(&self, s: &str) -> bool {
        self.0.iter().copied().eq(s.encode_utf16())
    }

    /// Re-encode to modified UTF-8. This is lossless, including unpaired
    /// surrogates, and is used for diagnostics and round-trip tests.
    pub(crate) fn to_modified_utf8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.0.len() * 2);
        for &unit in self.0.iter() {
            match unit {
                0x0000 => {
                    out.push(0xC0);
                    out.push(0x80);
                }
                0x0001..=0x007F => out.push(unit as u8),
                0x0080..=0x07FF => {
                    out.push(0xC0 | ((unit >> 6) as u8));
                    out.push(0x80 | ((unit & 0x3F) as u8));
                }
                _ => {
                    out.push(0xE0 | ((unit >> 12) as u8));
                    out.push(0x80 | (((unit >> 6) & 0x3F) as u8));
                    out.push(0x80 | ((unit & 0x3F) as u8));
                }
            }
        }
        out
    }
}

fn continuation(bytes: &[u8], index: usize, file_offset: usize) -> ParseResult<u8> {
    match bytes.get(index) {
        Some(&byte @ 0x80..=0xBF) => Ok(byte),
        _ => Err(invalid_text(file_offset, bytes)),
    }
}

pub(crate) fn invalid_text(offset: usize, bytes: &[u8]) -> ParseError {
    ParseError::new(
        offset,
        ParseErrorKind::InvalidText {
            bytes: bytes.to_vec().into_boxed_slice(),
        },
    )
}

fn write_escaped(f: &mut fmt::Formatter<'_>, text: &JavaText, quote: bool) -> fmt::Result {
    if quote {
        f.write_str("\"")?;
    }
    for decoded in char::decode_utf16(text.0.iter().copied()) {
        match decoded {
            Ok(c) if quote => write!(f, "{}", c.escape_debug())?,
            Ok(c) => f.write_str(c.encode_utf8(&mut [0; 4]))?,
            Err(error) => write!(f, "\\u{{{:04X}}}", error.unpaired_surrogate())?,
        }
    }
    if quote {
        f.write_str("\"")?;
    }
    Ok(())
}

impl fmt::Debug for JavaText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_escaped(f, self, true)
    }
}

impl fmt::Display for JavaText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_escaped(f, self, false)
    }
}

impl From<&str> for JavaText {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn decodes_ascii_nul_and_bmp() {
        let ascii = JavaText::from_modified_utf8(b"abc", 0).unwrap();
        assert_eq!(ascii.as_utf16(), utf16("abc"));

        let nul = JavaText::from_modified_utf8(&[0xC0, 0x80], 0).unwrap();
        assert_eq!(nul.as_utf16(), [0]);

        let euro = JavaText::from_modified_utf8(&[0xE2, 0x82, 0xAC], 0).unwrap();
        assert_eq!(euro.as_utf16(), utf16("€"));
    }

    #[test]
    fn decodes_surrogate_pairs_and_preserves_unpaired_surrogates() {
        let pair = JavaText::from_modified_utf8(&[0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80], 0).unwrap();
        assert_eq!(pair.as_utf16(), [0xD83D, 0xDE00]);
        assert_eq!(pair.to_rust_string().as_deref(), Some("😀"));

        let unpaired = JavaText::from_modified_utf8(&[0xED, 0xA0, 0x80], 0).unwrap();
        assert_eq!(unpaired.as_utf16(), [0xD800]);
        assert_eq!(unpaired.to_rust_string(), None);
        assert_eq!(unpaired.to_modified_utf8(), [0xED, 0xA0, 0x80]);
    }

    #[test]
    fn rejects_malformed_modified_utf8_and_keeps_bytes() {
        for bytes in [
            &[0x00][..],
            &[0x80][..],
            &[0xC0, 0x81][..],
            &[0xC1, 0xBF][..],
            &[0xE0, 0x80, 0x80][..],
            &[0xF0, 0x9F, 0x98, 0x80][..],
            &[0xED, 0xA0][..],
        ] {
            let error = JavaText::from_modified_utf8(bytes, 7).unwrap_err();
            assert_eq!(error.offset, 7);
            match error.kind {
                ParseErrorKind::InvalidText { bytes: kept } => {
                    assert_eq!(&*kept, bytes);
                }
                other => panic!("unexpected error: {other:?}"),
            }
        }
    }

    #[test]
    fn rust_string_round_trip() {
        let text = JavaText::from_str("a\u{0}b€😀");
        assert_eq!(text.to_rust_string().as_deref(), Some("a\u{0}b€😀"));
        let bytes = text.to_modified_utf8();
        assert_eq!(JavaText::from_modified_utf8(&bytes, 0).unwrap(), text);
    }
}
