use joaf_pdf_core::XrefTable;

use crate::{Catalog, Trailer};

pub struct Document<'a> {
    pub version: String,
    pub catalog: Catalog<'a>,
    pub trailer: Trailer,
    pub xref_table: XrefTable,
}

impl<'a> Document<'a> {}

#[cfg(test)]
mod tests {}
