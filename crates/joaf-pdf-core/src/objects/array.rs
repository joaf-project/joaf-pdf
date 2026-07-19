use super::PdfObject;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfArray<'a> {
    pub items: Vec<PdfObject<'a>>,
}

impl<'a> PdfArray<'a> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<PdfObject<'a>>) -> Self {
        Self { items }
    }
}
