use std::collections::HashMap;

use joaf_pdf_core::{PdfDictionary, XrefEntry};

use crate::PdfCatalog;

pub struct PdfDocument {
    pub version: String,
    pub catalog: PdfCatalog,
    pub trailer: PdfDictionary,
    pub xref_table: HashMap<u32, XrefEntry>,
}

#[cfg(test)]
mod tests {}
