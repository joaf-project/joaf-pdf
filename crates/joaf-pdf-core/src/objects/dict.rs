use std::{borrow::Borrow, collections::BTreeMap, io::Write};

use crate::{Formatter, PdfError, PdfWriter, WritePdf};

use super::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfDictionary<'a> {
    dict: BTreeMap<PdfName<'a>, PdfObject<'a>>,
}

impl<'a> WritePdf for PdfDictionary<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        if self.dict.is_empty() {
            return w.write_token(b"<<>>");
        }

        w.write_token(b"<<")?;
        w.write_newline()?;
        w.indent();
        for (key, value) in self.dict.iter() {
            w.write_indent()?;
            key.write_pdf(w)?;
            value.write_pdf(w)?;
            w.write_newline()?;
        }
        w.dedent();
        w.write_indent()?;
        w.write_token(b">>")?;

        Ok(())
    }
}

impl<'a> PdfDictionary<'a> {
    pub fn new() -> Self {
        Self {
            dict: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: PdfName<'a>, value: PdfObject<'a>) {
        self.dict.insert(key, value);
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&PdfObject<'a>>
    where
        PdfName<'a>: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.dict.get(key)
    }

    pub fn get_required<Q>(&self, key: &Q) -> Result<&PdfObject<'a>, PdfError>
    where
        PdfName<'a>: Borrow<Q>,
        Q: Ord + ?Sized + std::fmt::Display,
    {
        match self.dict.get(key) {
            Some(obj) => Ok(obj),
            None => Err(PdfError::missing_required_key(&key.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompactFormatter, PrettyFormatter};
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_pdf_dict_new() {
        let dict = PdfDictionary::new();
        assert_eq!(dict.dict.len(), 0);
    }

    #[test]
    fn test_pdf_dict_insert() {
        let mut dict = PdfDictionary::new();
        let key = PdfName::Type;
        let value = PdfObject::Name(PdfName::Page);
        dict.insert(key, value);
        assert_eq!(dict.dict.len(), 1);
    }

    #[test]
    fn test_pdf_dict_get() {
        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::Type, PdfObject::Name(PdfName::Page));
        assert_eq!(
            dict.get(&PdfName::Type),
            Some(&PdfObject::Name(PdfName::Page))
        );
    }

    #[test]
    fn test_pdf_dict_get_required() {
        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::Type, PdfObject::Name(PdfName::Page));
        assert_eq!(
            dict.get_required(&PdfName::Type),
            Ok(&PdfObject::Name(PdfName::Page))
        );
    }

    #[test]
    fn test_pdf_dict_get_required_missing() {
        let dict = PdfDictionary::new();
        let key = PdfName::Type;
        assert_eq!(
            dict.get_required(&key),
            Err(PdfError::missing_required_key("/Type"))
        );
    }

    #[test]
    fn test_pdf_dict_write_empty_pdf() -> std::io::Result<()> {
        let dict = PdfDictionary::new();

        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, CompactFormatter, false);
        dict.write_pdf(&mut writer)?;
        assert_eq!(String::from_utf8(buffer).unwrap(), "<<>>");

        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, PrettyFormatter, false);
        dict.write_pdf(&mut writer)?;
        assert_eq!(String::from_utf8(buffer).unwrap(), "<<>>");
        Ok(())
    }

    #[test]
    fn test_pdf_dict_write_pdf() -> std::io::Result<()> {
        let mut dict = PdfDictionary::new();
        let key = PdfName::Type;
        let value = PdfObject::Name(PdfName::Page);
        dict.insert(key, value);

        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, CompactFormatter, false);
        dict.write_pdf(&mut writer)?;
        assert_eq!(String::from_utf8(buffer).unwrap(), "<</Type/Page>>");

        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, PrettyFormatter, false);
        dict.write_pdf(&mut writer)?;
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            indoc!(
                "
                <<
                    /Type /Page
                >>"
            )
        );
        Ok(())
    }
}
