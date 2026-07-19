use joaf_pdf_core::{PdfDictionary, PdfError, PdfObject};

pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Page {
    pub media_box: Rect,
    pub contents: PdfObject,
    pub resources: PdfDictionary,
}

impl Rect {
    pub fn from_obj(obj: &PdfObject) -> Result<Self, PdfError> {
        let arr = obj.to_array()?;

        if arr.items.len() != 4 {
            return Err(PdfError::from("MediaBox is not an array of 4 elements."));
        }

        Ok(Self {
            x: arr.items[0].to_integer()? as u32,
            y: arr.items[1].to_integer()? as u32,
            width: arr.items[2].to_integer()? as u32,
            height: arr.items[3].to_integer()? as u32,
        })
    }
}

impl Page {
    pub fn from_dict(dict: &PdfDictionary) -> Result<Self, PdfError> {
        if dict.get_required("Type")?.to_name()? != "Page" {
            return Err(PdfError::from("Type is not a Page."));
        }

        let media_box = Rect::from_obj(dict.get_required("MediaBox")?)?;
        let contents = dict.get_required("Contents")?.clone();
        let resources = dict.get_required("Resources")?.to_dict()?.clone();

        Ok(Self {
            media_box,
            contents,
            resources,
        })
    }
}
