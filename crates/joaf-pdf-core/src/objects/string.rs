use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;

use crate::PdfError;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PdfString<'a> {
    pub bytes: Cow<'a, [u8]>,
}

impl<'a> PdfString<'a> {
    pub fn new(bytes: Cow<'a, [u8]>) -> Self {
        Self { bytes }
    }
}

impl<'a> Deref for PdfString<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl<'a> Display for PdfString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.bytes)
    }
}

impl<'a> TryFrom<&PdfString<'a>> for String {
    type Error = PdfError;

    fn try_from(value: &PdfString<'a>) -> Result<Self, Self::Error> {
        String::from_utf8(value.bytes.to_vec()).map_err(PdfError::from)
    }
}

impl<'a> TryInto<String> for PdfString<'a> {
    type Error = PdfError;

    fn try_into(self) -> Result<String, Self::Error> {
        String::from_utf8(self.bytes.into_owned()).map_err(PdfError::from)
    }
}
