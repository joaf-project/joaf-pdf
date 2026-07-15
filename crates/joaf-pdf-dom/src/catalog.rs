use crate::PdfPage;

pub struct PdfCatalog {
    pub pages: Vec<PdfPage>,
}

impl PdfCatalog {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }
}
