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
    Null(PdfNull),
    Boolean(PdfBoolean),
    Integer(PdfInteger),
    Real(PdfReal),
    String(PdfString),
    Name(PdfName),
    Array(PdfArray),
    Dictionary(PdfDictionary),
    Stream(PdfStream),
    Reference(ObjectId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfNull {
    pub value: (),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfBoolean {
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfInteger {
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfReal {
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfString {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfName {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfArray {
    pub entries: Vec<PdfObject>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfDictionary {
    pub entries: BTreeMap<String, PdfObject>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream {
    pub entries: BTreeMap<String, PdfObject>,
    pub data: Vec<u8>,
}

impl PdfNull {
    pub fn new() -> Self {
        Self { value: () }
    }
}

impl PdfBoolean {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

impl PdfInteger {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

impl PdfReal {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl PdfString {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl PdfName {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl PdfArray {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, value: PdfObject) {
        self.entries.push(value);
    }
}

impl PdfDictionary {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: &str, value: PdfObject) {
        self.entries.insert(key.to_string(), value);
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PdfObject)> {
        self.entries.iter()
    }

    pub fn get_name(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(PdfObject::Name(name)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_reference(&self, key: &str) -> Option<ObjectId> {
        match self.get(key) {
            Some(PdfObject::Reference(id)) => Some(*id),
            _ => None,
        }
    }
}

impl PdfStream {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            data: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: &str, value: PdfObject) {
        self.entries.insert(key.to_string(), value);
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PdfObject)> {
        self.entries.iter()
    }
}
