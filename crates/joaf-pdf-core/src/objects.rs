use std::{borrow::Cow, collections::BTreeMap};

use crate::PdfError;

pub const PDF_NULL: PdfObject = PdfObject::Null;
pub const PDF_TRUE: PdfObject = PdfObject::Boolean(true);
pub const PDF_FALSE: PdfObject = PdfObject::Boolean(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId {
    pub id: u32,
    pub generation: u16,
}

impl ObjectId {
    pub fn as_str(&self) -> String {
        format!("<id: {} gen: {}>", self.id, self.generation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XrefEntry {
    pub byte_offset: usize,
    pub generation: u16,
    pub in_use: bool,
}

pub type XrefTable = BTreeMap<u32, XrefEntry>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfArray<'a> {
    pub items: Vec<PdfObject<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfDictionary<'a> {
    dict: BTreeMap<String, PdfObject<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PdfName<'a> {
    pub str: Cow<'a, str>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfString<'a> {
    pub bytes: Cow<'a, [u8]>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfStream<'a> {
    pub entries: PdfDictionary<'a>,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject<'a> {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(PdfString<'a>),
    Name(PdfName<'a>),
    Array(PdfArray<'a>),
    Dictionary(PdfDictionary<'a>),
    Stream(PdfStream<'a>),
    Reference(ObjectId),
    IndirectObject(ObjectId, Box<PdfObject<'a>>),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfObjectsMap<'a> {
    map: BTreeMap<ObjectId, PdfObject<'a>>,
}

impl<'a> PdfObject<'a> {
    pub fn deref<'b>(&'b self, objects: &'b PdfObjectsMap<'a>) -> &'b PdfObject<'a> {
        match self {
            PdfObject::IndirectObject(_, obj) => obj.as_ref(),
            PdfObject::Reference(id) => objects.get(id),
            _ => self,
        }
    }

    pub fn to_bool(&self) -> Result<bool, PdfError> {
        match self {
            PdfObject::Boolean(b) => Ok(*b),
            _ => Err(PdfError::type_mismatch("Boolean", self.str_type())),
        }
    }

    pub fn to_integer(&self) -> Result<i64, PdfError> {
        match self {
            PdfObject::Integer(i) => Ok(*i),
            _ => Err(PdfError::type_mismatch("Integer", self.str_type())),
        }
    }

    pub fn to_real(&self) -> Result<f64, PdfError> {
        match self {
            PdfObject::Real(r) => Ok(*r),
            _ => Err(PdfError::type_mismatch("Real", self.str_type())),
        }
    }

    pub fn to_name(&self) -> Result<&PdfName<'a>, PdfError> {
        match self {
            PdfObject::Name(n) => Ok(n),
            _ => Err(PdfError::type_mismatch("Name", self.str_type())),
        }
    }

    pub fn to_array(&self) -> Result<&PdfArray<'a>, PdfError> {
        match self {
            PdfObject::Array(a) => Ok(a),
            _ => Err(PdfError::type_mismatch("Array", self.str_type())),
        }
    }

    pub fn to_dict(&self) -> Result<&PdfDictionary<'a>, PdfError> {
        match self {
            PdfObject::Dictionary(dict) => Ok(dict),
            _ => Err(PdfError::type_mismatch("Dictionary", self.str_type())),
        }
    }

    pub fn to_stream(&self) -> Result<&PdfStream<'a>, PdfError> {
        match self {
            PdfObject::Stream(stream) => Ok(stream),
            _ => Err(PdfError::type_mismatch("Stream", self.str_type())),
        }
    }

    pub fn to_ref(&self) -> Result<&ObjectId, PdfError> {
        match self {
            PdfObject::Reference(obj) => Ok(obj),
            _ => Err(PdfError::type_mismatch("Reference", self.str_type())),
        }
    }

    pub fn to_indirect(&self) -> Result<(&ObjectId, &PdfObject<'a>), PdfError> {
        match self {
            PdfObject::IndirectObject(id, obj) => Ok((id, obj.as_ref())),
            _ => Err(PdfError::type_mismatch("IndirectObject", self.str_type())),
        }
    }

    pub fn str_type(&self) -> &'static str {
        match self {
            PdfObject::Null => "Null",
            PdfObject::Boolean(_) => "Boolean",
            PdfObject::Integer(_) => "Integer",
            PdfObject::Real(_) => "Real",
            PdfObject::String(_) => "String",
            PdfObject::Name(_) => "Name",
            PdfObject::Array(_) => "Array",
            PdfObject::Dictionary(_) => "Dictionary",
            PdfObject::Stream(_) => "Stream",
            PdfObject::Reference(_) => "Reference",
            PdfObject::IndirectObject(_, _) => "IndirectObject",
        }
    }
}

impl<'a> PdfArray<'a> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<PdfObject<'a>>) -> Self {
        Self { items }
    }
}

impl<'a> PdfDictionary<'a> {
    pub fn new() -> Self {
        Self {
            dict: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: PdfObject<'a>) {
        self.dict.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&PdfObject<'a>> {
        self.dict.get(key)
    }

    pub fn get_required(&self, key: &str) -> Result<&PdfObject<'a>, PdfError> {
        match self.dict.get(key) {
            Some(obj) => Ok(obj),
            None => Err(PdfError::missing_required_key(key)),
        }
    }
}

impl<'a> From<&'a str> for PdfName<'a> {
    fn from(s: &'a str) -> Self {
        Self {
            str: Cow::Borrowed(s),
        }
    }
}

impl<'a> From<Cow<'a, str>> for PdfName<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        Self { str: s }
    }
}

impl<'a> PdfName<'a> {
    pub const PAGE: PdfName<'static> = PdfName {
        str: Cow::Borrowed("Page"),
    };
    pub const TYPE: PdfName<'static> = PdfName {
        str: Cow::Borrowed("Type"),
    };
}

impl<'a> PdfStream<'a> {
    pub fn new(entries: PdfDictionary<'a>, offset: usize) -> Self {
        Self { entries, offset }
    }
}

impl<'a> PdfObjectsMap<'a> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, object_id: ObjectId, obj_ref: PdfObject<'a>) {
        self.map.insert(object_id, obj_ref);
    }

    pub fn get(&self, object_id: &ObjectId) -> &PdfObject<'a> {
        match self.map.get(object_id) {
            Some(obj_ref) => obj_ref,
            None => &PDF_NULL,
        }
    }
}
