use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId {
    pub id: u32,
    pub generation: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct XrefEntry {
    pub byte_offset: u64,
    pub generation: u16,
    pub in_use: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Name(String),
    Array(Vec<PdfObject>),
    Dictionary(BTreeMap<String, PdfObject>),
    Stream(PdfStream),
    Reference(ObjectId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream {
    pub entries: BTreeMap<String, PdfObject>,
    pub data: Vec<u8>,
}

impl PdfStream {
    pub fn new(entries: BTreeMap<String, PdfObject>, data: Vec<u8>) -> Self {
        Self { entries, data }
    }
}
