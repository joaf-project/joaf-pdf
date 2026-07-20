use std::io::Write;

use crate::{Formatter, PdfWriter, WritePdf};

use super::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfStream<'a> {
    pub entries: PdfDictionary<'a>,
    pub offset: usize,
}

impl<'a> WritePdf for PdfStream<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        self.entries.write_pdf(w)?;
        w.write_newline()?;
        w.write_token(b"stream\n")?;
        // TODO: Write stream bytes
        w.write_token(b"endstream")
    }
}

impl<'a> PdfStream<'a> {
    pub fn new(entries: PdfDictionary<'a>, offset: usize) -> Self {
        Self { entries, offset }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompactFormatter, PdfObject, PrettyFormatter};

    use indoc::indoc;

    #[test]
    fn test_pdf_stream_write_pdf() {
        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::LENGTH, PdfObject::Integer(0));
        let stream = PdfObject::Stream(PdfStream::new(dict, 0));

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
}
