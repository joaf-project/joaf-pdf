use super::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfStream<'a> {
    pub entries: PdfDictionary<'a>,
    pub offset: usize,
}

impl<'a> PdfStream<'a> {
    pub fn new(entries: PdfDictionary<'a>, offset: usize) -> Self {
        Self { entries, offset }
    }
}
