use std::collections::BTreeMap;

use super::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfObjectsMap<'a> {
    map: BTreeMap<ObjectId, PdfObject<'a>>,
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
            None => &PdfObject::NULL,
        }
    }
}
