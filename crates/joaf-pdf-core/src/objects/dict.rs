use std::{borrow::Borrow, collections::BTreeMap};

use crate::PdfError;

use super::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfDictionary<'a> {
    dict: BTreeMap<PdfName<'a>, PdfObject<'a>>,
}

impl<'a> PdfDictionary<'a> {
    pub fn new() -> Self {
        Self {
            dict: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: PdfName<'a>, value: PdfObject<'a>) {
        self.dict.insert(key, value);
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&PdfObject<'a>>
    where
        PdfName<'a>: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.dict.get(key)
    }

    pub fn get_required<Q>(&self, key: &Q) -> Result<&PdfObject<'a>, PdfError>
    where
        PdfName<'a>: Borrow<Q>,
        Q: Ord + ?Sized + std::fmt::Display,
    {
        match self.dict.get(key) {
            Some(obj) => Ok(obj),
            None => Err(PdfError::missing_required_key(&key.to_string())),
        }
    }
}
