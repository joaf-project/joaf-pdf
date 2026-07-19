use joaf_pdf_core::{PdfDictionary, PdfError, PdfObjectsMap, XrefTable};

use crate::{Catalog, Trailer};

pub struct Document {
    pub version: String,
    pub catalog: Catalog,
    pub trailer: Trailer,
    pub xref_table: XrefTable,
    pub objects: PdfObjectsMap,
}

impl Document {
    pub fn try_new(
        version: String,
        trailer_dict: &PdfDictionary,
        xref_table: XrefTable,
    ) -> Result<Self, PdfError> {
        Ok(Self {
            version,
            catalog: Catalog::new(),
            trailer: Trailer::try_from(trailer_dict)?,
            xref_table,
            objects: PdfObjectsMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {}
