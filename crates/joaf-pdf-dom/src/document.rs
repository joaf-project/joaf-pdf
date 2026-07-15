use std::collections::BTreeMap;

use joaf_pdf_core::{PdfDictionary, XrefEntry};

use crate::PdfCatalog;

pub struct PdfDocument {
    pub version: String,
    pub catalog: PdfCatalog,
    pub trailer: PdfDictionary,
    pub xref_table: BTreeMap<u32, XrefEntry>,
}

impl PdfDocument {
    pub fn new() -> Self {
        Self {
            version: String::new(),
            catalog: PdfCatalog::new(),
            trailer: PdfDictionary::new(),
            xref_table: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {}
