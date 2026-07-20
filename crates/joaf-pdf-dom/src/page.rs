use joaf_pdf_core::{PdfError, PdfName, PdfObject};

pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub enum PageLayout {
    SinglePage,
    OneColumn,
    TwoColumnLeft,
    TwoPageLeft,
    TwoPageRight,
}

pub enum PageMode {
    UseNone,
    UseOutlines,
    UseThumbs,
    FullScreen,
    UseOC,
    UseAttachment,
}

pub enum PageTreeKid<'a> {
    Tree(PageTree<'a>),
    Page(Page<'a>),
}

pub struct PageTree<'a> {
    pub parent: Option<&'a PageTree<'a>>,
    pub kids: Vec<PageTreeKid<'a>>,
    pub count: usize,
}

pub struct Page<'a> {
    pub parent: &'a PageTree<'a>,
    pub last_modified: Option<String>,
    pub media_box: Rect,
    pub crop_box: Option<Rect>,
    pub bleed_box: Option<Rect>,
    pub trim_box: Option<Rect>,
    pub art_box: Option<Rect>,
    // box_color_info
    // pub contents: Option<PdfObject>,
    pub rotation: Option<u32>,
    // group
    // thumb
    // B
    // Dur
    // Trans
    // AA
    // Metadata
    // PieceInfo
    // StructParents
    // ID
    // PZ
    // SeparationInfo
    // Tabs
    // TemplateInstantiated
    // PresSteps
    // UserUnit
    // VP
    // pub page_labels: Option<PdfObject>,
    // pub resources: Option<PdfObject>,
    pub page_layout: Option<PageLayout>,
    pub page_mode: Option<PageMode>,
}

impl Rect {
    pub fn from_obj(obj: &PdfObject) -> Result<Self, PdfError> {
        let arr = obj.as_array()?;

        if arr.items.len() != 4 {
            return Err(PdfError::from("MediaBox is not an array of 4 elements."));
        }

        Ok(Self {
            x: arr.items[0].as_integer()? as u32,
            y: arr.items[1].as_integer()? as u32,
            width: arr.items[2].as_integer()? as u32,
            height: arr.items[3].as_integer()? as u32,
        })
    }
}

impl<'a> From<PdfName<'a>> for PageLayout {
    fn from(value: PdfName<'a>) -> Self {
        match value {
            PdfName::SinglePage => Self::SinglePage,
            PdfName::OneColumn => Self::OneColumn,
            PdfName::TwoColumnLeft => Self::TwoColumnLeft,
            PdfName::TwoPageLeft => Self::TwoPageLeft,
            PdfName::TwoPageRight => Self::TwoPageRight,
            _ => Self::SinglePage,
        }
    }
}

impl<'a> From<PdfName<'a>> for PageMode {
    fn from(value: PdfName<'a>) -> Self {
        match value {
            PdfName::UseNone => Self::UseNone,
            PdfName::UseOutlines => Self::UseOutlines,
            PdfName::UseThumbs => Self::UseThumbs,
            PdfName::FullScreen => Self::FullScreen,
            PdfName::UseOC => Self::UseOC,
            PdfName::UseAttachment => Self::UseAttachment,
            _ => Self::UseNone,
        }
    }
}

pub const EMPTY_PAGE_TREE: &'static PageTree = &PageTree {
    parent: None,
    count: 0,
    kids: Vec::new(),
};

impl<'a> PageTree<'a> {
    pub fn new() -> Self {
        Self {
            parent: None,
            count: 0,
            kids: Vec::new(),
        }
    }
    pub fn empty() -> Self {
        Self {
            parent: None,
            count: 0,
            kids: Vec::new(),
        }
    }
}

impl<'a> Page<'a> {
    pub fn empty() -> Self {
        Self {
            parent: EMPTY_PAGE_TREE,
            last_modified: None,
            media_box: Rect {
                x: 0,
                y: 0,
                width: 612,
                height: 792,
            },
            crop_box: None,
            bleed_box: None,
            trim_box: None,
            art_box: None,
            rotation: None,
            page_layout: None,
            page_mode: None,
        }
    }

    pub fn new(parent: &'a PageTree<'a>, media_box: Rect) -> Self {
        Self {
            parent,
            last_modified: None,
            media_box,
            crop_box: None,
            bleed_box: None,
            trim_box: None,
            art_box: None,
            rotation: None,
            page_layout: None,
            page_mode: None,
        }
    }
    // pub fn from_dict(dict: &PdfDictionary<'a>) -> Result<Self, PdfError> {
    //     if dict.get_required(PdfName::Type)?.as_name()? != PdfName::Page {
    //         return Err(PdfError::from("Type is not a Page."));
    //     }
    // }
}
