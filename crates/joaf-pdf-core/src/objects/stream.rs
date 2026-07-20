use std::{borrow::Cow, io::Write};

use crate::{Formatter, PdfError, PdfWriter, WritePdf};

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum StreamData<'a> {
    Offset(usize),
    Buffer(Cow<'a, [u8]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream<'a> {
    pub entries: PdfDictionary<'a>,
    pub data: StreamData<'a>,
}

impl<'a> From<usize> for StreamData<'a> {
    fn from(value: usize) -> Self {
        StreamData::Offset(value)
    }
}

impl<'a> From<Vec<u8>> for StreamData<'a> {
    fn from(value: Vec<u8>) -> Self {
        StreamData::Buffer(Cow::Owned(value))
    }
}

impl<'a> From<&'a [u8]> for StreamData<'a> {
    fn from(value: &'a [u8]) -> Self {
        StreamData::Buffer(Cow::Borrowed(value))
    }
}

impl<'a> WritePdf for PdfStream<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        let StreamData::Buffer(buffer) = &self.data else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream data not loaded",
            ));
        };
        self.entries.write_pdf(w)?;
        w.write_newline()?;
        w.write_token(b"stream\n")?;
        w.write_all(buffer)?;
        if !buffer.is_empty() && buffer.last() != Some(&b'\n') {
            w.write_whitespace(&[b'\n'])?;
        }
        w.write_token(b"endstream")
    }
}

impl<'a> PdfStream<'a> {
    pub fn new(entries: PdfDictionary<'a>) -> Self {
        Self {
            entries,
            data: StreamData::Buffer(Cow::Borrowed(b"")),
        }
    }

    pub fn with(mut self, data: StreamData<'a>) -> Self {
        self.data = data;
        self
    }

    pub fn length(&self) -> Option<usize> {
        match self.entries.get_required(&PdfName::LENGTH) {
            Ok(PdfObject::Integer(length)) => Some(*length as usize),
            _ => None,
        }
    }

    pub fn load_from_buffer(&mut self, buffer: &'a [u8]) -> Result<(), PdfError> {
        match self.data {
            StreamData::Buffer(_) => Err(PdfError::from("Stream already loaded")),
            StreamData::Offset(offset) => {
                let len = self.length().ok_or(PdfError::from("Unknown length"))?;
                self.data = StreamData::Buffer(buffer[offset..offset + len].into());
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompactFormatter, PdfObject, PrettyFormatter};

    use indoc::indoc;

    #[test]
    fn test_pdf_stream_write_empty() {
        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::LENGTH, PdfObject::Integer(0));
        let stream = PdfObject::Stream(PdfStream::new(dict));

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        stream.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                <</Length 0>>stream
                endstream"
            )
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        stream.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                <<
                    /Length 0
                >>
                stream
                endstream"
            )
        );
    }

    #[test]
    fn test_pdf_stream_write_with_data() {
        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::LENGTH, PdfObject::Integer(5));
        let stream = PdfObject::Stream(PdfStream::new(dict).with(b"Hello".as_ref().into()));

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        stream.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                <</Length 5>>stream
                Hello
                endstream"
            )
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        stream.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                <<
                    /Length 5
                >>
                stream
                Hello
                endstream"
            )
        );
    }

    #[test]
    fn test_pdf_stream_write_with_data_ending_with_new_line() {
        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::LENGTH, PdfObject::Integer(6));
        let stream = PdfObject::Stream(PdfStream::new(dict).with(b"Hello\n".as_ref().into()));

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        stream.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                <</Length 6>>stream
                Hello
                endstream"
            )
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        stream.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                <<
                    /Length 6
                >>
                stream
                Hello
                endstream"
            )
        );
    }
}
