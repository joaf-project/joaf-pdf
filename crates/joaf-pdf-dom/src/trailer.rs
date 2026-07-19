use joaf_pdf_core::{ObjectId, PdfDictionary, PdfError, PdfObject};

pub struct Trailer {
    pub size: u32,
    pub root: ObjectId,
}

impl Trailer {
    pub fn from(obj: &PdfDictionary) -> Result<Trailer, PdfError> {
        let size = match obj.dict.get("Size") {
            Some(PdfObject::Integer(size)) => *size as u32,
            _ => return Err(PdfError::from("Size is not an integer.")),
        };

        let root = match obj.dict.get("Root") {
            Some(PdfObject::Reference(root)) => *root,
            _ => return Err(PdfError::from("Root is not an indirect object.")),
        };

        Ok(Trailer { size, root })
    }
}
