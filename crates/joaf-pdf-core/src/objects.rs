use std::collections::BTreeMap;

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
pub struct PdfArray {
    pub items: Vec<PdfObject>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfDictionary {
    dict: BTreeMap<String, PdfObject>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfString {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfStream {
    pub entries: PdfDictionary,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(PdfString),
    Name(String),
    Array(PdfArray),
    Dictionary(PdfDictionary),
    Stream(PdfStream),
    Reference(ObjectId),
    IndirectObject(ObjectId, Box<PdfObject>),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfObjectsMap {
    map: BTreeMap<ObjectId, PdfObject>,
}

impl PdfObject {
    pub fn deref<'a: 'b, 'b>(&'a self, objects: &'b PdfObjectsMap) -> &'b PdfObject {
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

    pub fn to_name(&self) -> Result<&str, PdfError> {
        match self {
            PdfObject::Name(s) => Ok(s.as_str()),
            _ => Err(PdfError::type_mismatch("Name", self.str_type())),
        }
    }

    pub fn to_array(&self) -> Result<&PdfArray, PdfError> {
        match self {
            PdfObject::Array(a) => Ok(a),
            _ => Err(PdfError::type_mismatch("Array", self.str_type())),
        }
    }

    pub fn to_dict(&self) -> Result<&PdfDictionary, PdfError> {
        match self {
            PdfObject::Dictionary(dict) => Ok(dict),
            _ => Err(PdfError::type_mismatch("Dictionary", self.str_type())),
        }
    }

    pub fn to_stream(&self) -> Result<&PdfStream, PdfError> {
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

    pub fn to_indirect(&self) -> Result<(&ObjectId, &PdfObject), PdfError> {
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

impl PdfArray {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<PdfObject>) -> Self {
        Self { items }
    }
}

impl PdfDictionary {
    pub fn new() -> Self {
        Self {
            dict: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: PdfObject) {
        self.dict.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.dict.get(key)
    }

    pub fn get_required(&self, key: &str) -> Result<&PdfObject, PdfError> {
        match self.dict.get(key) {
            Some(obj) => Ok(obj),
            None => Err(PdfError::missing_required_key(key)),
        }
    }
}

impl PdfStream {
    pub fn new(entries: PdfDictionary, offset: usize) -> Self {
        Self { entries, offset }
    }
}

impl PdfObjectsMap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, object_id: ObjectId, obj_ref: PdfObject) {
        self.map.insert(object_id, obj_ref);
    }

    pub fn get(&self, object_id: &ObjectId) -> &PdfObject {
        match self.map.get(object_id) {
            Some(obj_ref) => obj_ref,
            None => &PDF_NULL,
        }
    }
}
