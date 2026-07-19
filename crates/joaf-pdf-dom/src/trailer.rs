use joaf_pdf_core::{ObjectId, PdfDictionary, PdfError, PdfObject};

pub struct Trailer {
    pub size: u32,
    pub root: ObjectId,
}

impl TryFrom<&PdfDictionary> for Trailer {
    type Error = PdfError;

    fn try_from(value: &PdfDictionary) -> Result<Self, Self::Error> {
        let size = match value.get_required("Size") {
            Ok(PdfObject::Integer(size)) => *size as u32,
            _ => return Err(PdfError::from("Size is not an integer.")),
        };

        let root = match value.get_required("Root") {
            Ok(PdfObject::Reference(root)) => *root,
            _ => return Err(PdfError::from("Root is not an indirect object.")),
        };

        Ok(Trailer { size, root })
    }
}

impl Trailer {}
