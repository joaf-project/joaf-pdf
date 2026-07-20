use std::io::Write;

use crate::{Formatter, PdfWriter, WritePdf};

use super::PdfObject;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfArray<'a> {
    pub items: Vec<PdfObject<'a>>,
}

impl<'a> From<Vec<PdfObject<'a>>> for PdfArray<'a> {
    fn from(items: Vec<PdfObject<'a>>) -> Self {
        Self { items }
    }
}

impl<'a> WritePdf for PdfArray<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        if self.items.is_empty() {
            return w.write_token(b"[]");
        }

        w.write_token(b"[")?;
        w.indent();
        for (i, item) in self.items.iter().enumerate() {
            if i % 4 == 0 {
                w.write_newline()?;
                w.write_indent()?;
            }
            item.write_pdf(w)?;
        }
        w.dedent();
        w.write_newline()?;
        w.write_indent()?;
        w.write_token(b"]")?;

        Ok(())
    }
}

impl<'a> PdfArray<'a> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompactFormatter, ObjectId, PdfName, PrettyFormatter};
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_pdf_array_from_vec() {
        let arr = PdfArray::from(vec![PdfObject::Null, PdfObject::Boolean(true)]);
        assert_eq!(arr.items.len(), 2);
    }

    #[test]
    fn test_pdf_array_write_empty_pdf() {
        let arr = PdfArray::from(vec![]);

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "[]");

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "[]");
    }

    #[test]
    fn test_pdf_array_write_pdf() {
        let arr = PdfArray::from(vec![PdfObject::Null, PdfObject::Boolean(true)]);

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "[null true]");

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                [
                    null true
                ]"
            )
        );
    }

    #[test]
    fn test_pdf_array_write_pdf_8_items() {
        let arr = PdfArray::from(vec![
            PdfObject::NULL,
            PdfObject::TRUE,
            PdfObject::NULL,
            PdfObject::FALSE,
            PdfObject::NULL,
            PdfObject::TRUE,
            PdfObject::NULL,
            PdfObject::FALSE,
        ]);

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "[null true null false null true null false]"
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                [
                    null true null false
                    null true null false
                ]"
            )
        );
    }

    #[test]
    fn test_pdf_array_write_pdf_names() {
        let arr = PdfArray::from(vec![
            PdfObject::Name(PdfName::TYPE),
            PdfObject::Name(PdfName::PAGE),
            PdfObject::Name(PdfName::TYPE),
            PdfObject::Name(PdfName::PAGE),
            PdfObject::Name(PdfName::TYPE),
            PdfObject::Name(PdfName::PAGE),
            PdfObject::Name(PdfName::TYPE),
            PdfObject::Name(PdfName::PAGE),
        ]);

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "[/Type/Page/Type/Page/Type/Page/Type/Page]"
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                [
                    /Type /Page /Type /Page
                    /Type /Page /Type /Page
                ]"
            )
        );
    }

    #[test]
    fn test_pdf_array_write_pdf_indirect_object() {
        let arr = PdfArray::from(vec![
            PdfObject::Reference(ObjectId {
                id: 1,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 2,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 3,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 4,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 5,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 6,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 7,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 8,
                generation: 0,
            }),
        ]);

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "[1 0 R 2 0 R 3 0 R 4 0 R 5 0 R 6 0 R 7 0 R 8 0 R]"
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        arr.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "
                [
                    1 0 R 2 0 R 3 0 R 4 0 R
                    5 0 R 6 0 R 7 0 R 8 0 R
                ]"
            )
        );
    }
}
