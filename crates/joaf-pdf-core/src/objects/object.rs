use crate::PdfError;

use super::*;

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

impl<'a> PdfObject<'a> {
    pub const NULL: PdfObject<'static> = PdfObject::Null;
    pub const TRUE: PdfObject<'static> = PdfObject::Boolean(true);
    pub const FALSE: PdfObject<'static> = PdfObject::Boolean(false);

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
