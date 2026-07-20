use std::fmt::Display;
use std::ops::Deref;
use std::{borrow::Cow, io::Write};

use crate::{Formatter, PdfError, PdfWriter, WritePdf};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfString<'a> {
    pub bytes: Cow<'a, [u8]>,
}

impl<'a> PdfString<'a> {
    pub fn new(bytes: Cow<'a, [u8]>) -> Self {
        Self { bytes }
    }
}

impl<'a> Deref for PdfString<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl<'a> Display for PdfString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.bytes)
    }
}

impl<'a> WritePdf for PdfString<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        w.write_token(b"(")?;

        let mut start = 0;

        for (i, &b) in self.bytes.iter().enumerate() {
            let escape = match b {
                b'(' => Some(b"\\(" as &[u8]),
                b')' => Some(b"\\)" as &[u8]),
                b'\\' => Some(b"\\\\" as &[u8]),
                b'\n' => Some(b"\\n" as &[u8]),
                b'\r' => Some(b"\\r" as &[u8]),
                b'\t' => Some(b"\\t" as &[u8]),
                0x08 => Some(b"\\b" as &[u8]), // Backspace
                0x0C => Some(b"\\f" as &[u8]), // Form feed
                _ => None,
            };

            if let Some(escape_sequence) = escape {
                if start < i {
                    w.write_all(&self.bytes[start..i])?;
                }
                w.write_all(escape_sequence)?;
                start = i + 1;
            }
        }

        if start < self.bytes.len() {
            w.write_all(&self.bytes[start..])?;
        }

        w.write_all(b")")
    }
}

impl<'a> From<Cow<'a, [u8]>> for PdfString<'a> {
    fn from(bytes: Cow<'a, [u8]>) -> Self {
        Self { bytes }
    }
}

impl<'a> From<&'a [u8]> for PdfString<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes: Cow::Borrowed(bytes),
        }
    }
}

impl<'a> From<Vec<u8>> for PdfString<'a> {
    fn from(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Cow::Owned(bytes),
        }
    }
}

impl<'a> From<&PdfString<'a>> for Vec<u8> {
    fn from(pdf_string: &PdfString<'a>) -> Self {
        pdf_string.bytes.to_vec()
    }
}

impl<'a> TryFrom<&PdfString<'a>> for String {
    type Error = PdfError;

    fn try_from(value: &PdfString<'a>) -> Result<Self, Self::Error> {
        String::from_utf8(value.bytes.to_vec()).map_err(PdfError::from)
    }
}

impl<'a> TryInto<String> for PdfString<'a> {
    type Error = PdfError;

    fn try_into(self) -> Result<String, Self::Error> {
        String::from_utf8(self.bytes.into_owned()).map_err(PdfError::from)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[test]
    fn test_pdf_string_from_str() {
        let pdf_string = PdfString::from(b"hello".as_ref());
        assert_eq!(pdf_string.bytes.as_ref(), b"hello");
    }

    #[test]
    fn test_pdf_string_write_compact_pdf() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        let pdf_string = PdfString::from(b"hello".as_ref());
        pdf_string.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "(hello)");
    }

    #[test]
    fn test_pdf_string_write_compact_pdf_escaped() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        let pdf_string = PdfString::from(b"hello\nworld".as_ref());
        pdf_string.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "(hello\\nworld)");
    }

    #[test]
    fn test_pdf_string_write_pretty_pdf() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        let pdf_string = PdfString::from(b"hello".as_ref());
        pdf_string.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "(hello)");
    }

    #[test]
    fn test_pdf_string_write_pretty_pdf_escaped() {
        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        let pdf_string = PdfString::from(b"hello\nworld".as_ref());
        pdf_string.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "(hello\\nworld)");
    }
}
