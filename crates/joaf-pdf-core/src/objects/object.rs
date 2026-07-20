use std::io::Write;

use crate::{Formatter, PdfError, PdfWriter, WritePdf};

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
    IndirectObject {
        object_id: ObjectId,
        object: Box<PdfObject<'a>>,
    },
}

impl<'a> TryFrom<&'a PdfObject<'a>> for bool {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Boolean(b) => Ok(*b),
            _ => Err(PdfError::type_mismatch("Boolean", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for i64 {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Integer(i) => Ok(*i),
            _ => Err(PdfError::type_mismatch("Integer", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for f64 {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Real(r) => Ok(*r),
            _ => Err(PdfError::type_mismatch("Real", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for &'a PdfName<'a> {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Name(n) => Ok(n),
            _ => Err(PdfError::type_mismatch("Name", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for &'a PdfString<'a> {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::String(s) => Ok(s),
            _ => Err(PdfError::type_mismatch("String", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for &'a PdfArray<'a> {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Array(arr) => Ok(arr),
            _ => Err(PdfError::type_mismatch("Array", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for &'a PdfDictionary<'a> {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Dictionary(dict) => Ok(dict),
            _ => Err(PdfError::type_mismatch("Dictionary", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for &'a PdfStream<'a> {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Stream(s) => Ok(s),
            _ => Err(PdfError::type_mismatch("Stream", value.as_type_str())),
        }
    }
}

impl<'a> TryFrom<&'a PdfObject<'a>> for ObjectId {
    type Error = PdfError;
    fn try_from(value: &'a PdfObject<'a>) -> Result<Self, Self::Error> {
        match value {
            PdfObject::Reference(id) => Ok(*id),
            PdfObject::IndirectObject { object_id, .. } => Ok(*object_id),
            _ => Err(PdfError::type_mismatch("Reference", value.as_type_str())),
        }
    }
}

impl<'a> WritePdf for PdfObject<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        match self {
            PdfObject::Null => w.write_token(b"null"),
            PdfObject::Boolean(b) => {
                if *b {
                    w.write_token(b"true")
                } else {
                    w.write_token(b"false")
                }
            }
            PdfObject::Integer(i) => w.write_token(i.to_string().as_bytes()),
            PdfObject::Real(r) => {
                let s = if r.fract() == 0.0 {
                    format!("{:.1}", r) // Force .0 for whole numbers
                } else {
                    format!("{}", r) // Keep full precision for others
                };
                w.write_token(s.as_bytes())
            }
            PdfObject::Name(name) => name.write_pdf(w),
            PdfObject::Array(arr) => arr.write_pdf(w),
            PdfObject::Reference(id) => id.write_pdf(w),
            PdfObject::String(pdf_str) => pdf_str.write_pdf(w),
            PdfObject::Dictionary(dict) => dict.write_pdf(w),
            PdfObject::Stream(stream) => stream.write_pdf(w),
            PdfObject::IndirectObject { object_id, object } => {
                w.write_token(
                    &format!("\n{} {} obj\n", object_id.id, object_id.generation).as_bytes(),
                )?;
                w.indent();
                w.write_indent()?;
                object.write_pdf(w)?;
                w.dedent();
                w.write_token(b"\nendobj\n")
            }
        }
    }
}

impl<'a> PdfObject<'a> {
    pub const NULL: PdfObject<'static> = PdfObject::Null;
    pub const TRUE: PdfObject<'static> = PdfObject::Boolean(true);
    pub const FALSE: PdfObject<'static> = PdfObject::Boolean(false);

    pub fn deref<'b>(&'b self, objects: &'b PdfObjectsMap<'a>) -> &'b PdfObject<'a> {
        match self {
            PdfObject::IndirectObject { object_id, .. } => objects.get(object_id),
            PdfObject::Reference(id) => objects.get(id),
            _ => self,
        }
    }

    pub fn as_bool(&self) -> Result<bool, PdfError> {
        match self {
            PdfObject::Boolean(b) => Ok(*b),
            _ => Err(PdfError::type_mismatch("Boolean", self.as_type_str())),
        }
    }

    pub fn as_integer(&self) -> Result<i64, PdfError> {
        match self {
            PdfObject::Integer(i) => Ok(*i),
            _ => Err(PdfError::type_mismatch("Integer", self.as_type_str())),
        }
    }

    pub fn as_real(&self) -> Result<f64, PdfError> {
        match self {
            PdfObject::Real(r) => Ok(*r),
            _ => Err(PdfError::type_mismatch("Real", self.as_type_str())),
        }
    }

    pub fn as_name(&self) -> Result<&PdfName<'a>, PdfError> {
        match self {
            PdfObject::Name(n) => Ok(n),
            _ => Err(PdfError::type_mismatch("Name", self.as_type_str())),
        }
    }

    pub fn as_array(&self) -> Result<&PdfArray<'a>, PdfError> {
        match self {
            PdfObject::Array(a) => Ok(a),
            _ => Err(PdfError::type_mismatch("Array", self.as_type_str())),
        }
    }

    pub fn as_dict(&self) -> Result<&PdfDictionary<'a>, PdfError> {
        match self {
            PdfObject::Dictionary(dict) => Ok(dict),
            _ => Err(PdfError::type_mismatch("Dictionary", self.as_type_str())),
        }
    }

    pub fn as_stream(&self) -> Result<&PdfStream<'a>, PdfError> {
        match self {
            PdfObject::Stream(stream) => Ok(stream),
            _ => Err(PdfError::type_mismatch("Stream", self.as_type_str())),
        }
    }

    pub fn as_ref_id(&self) -> Result<&ObjectId, PdfError> {
        match self {
            PdfObject::Reference(obj) => Ok(obj),
            _ => Err(PdfError::type_mismatch("Reference", self.as_type_str())),
        }
    }

    pub fn as_indirect(&self) -> Result<(&ObjectId, &PdfObject<'a>), PdfError> {
        match self {
            PdfObject::IndirectObject {
                object_id: id,
                object,
            } => Ok((id, object.as_ref())),
            _ => Err(PdfError::type_mismatch(
                "IndirectObject",
                self.as_type_str(),
            )),
        }
    }

    pub fn as_type_str(&self) -> &'static str {
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
            PdfObject::IndirectObject { .. } => "IndirectObject",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompactFormatter, PrettyFormatter};
    use indoc::indoc;

    #[test]
    fn test_pdf_object_write_indirect_object() {
        let kids = PdfArray::from(vec![
            PdfObject::Reference(ObjectId {
                id: 3,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 4,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 5,
                generation: 0,
            }),
        ]);

        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::TYPE, PdfObject::Name(PdfName::PAGES));
        dict.insert(
            PdfName::NAME,
            PdfObject::String(PdfString::from(b"Joaf PDF".as_ref())),
        );
        dict.insert(PdfName::COUNT, PdfObject::Integer(3));
        dict.insert(PdfName::KIDS, PdfObject::Array(kids));

        let obj = PdfObject::IndirectObject {
            object_id: ObjectId {
                id: 2,
                generation: 100,
            },
            object: PdfObject::Dictionary(dict).into(),
        };

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        obj.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "

                2 100 obj
                <</Count 3/Kids[3 0 R 4 0 R 5 0 R]/Name(Joaf PDF)/Type/Pages>>
                endobj
                "
            )
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        obj.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "

                2 100 obj
                    <<
                        /Count 3
                        /Kids [
                            3 0 R 4 0 R 5 0 R
                        ]
                        /Name (Joaf PDF)
                        /Type /Pages
                    >>
                endobj
                "
            )
        );
    }

    #[test]
    fn test_pdf_object_write_indirect_stream() {
        let mut dict = PdfDictionary::new();
        dict.insert(PdfName::LENGTH, PdfObject::Integer(0));

        let obj = PdfObject::IndirectObject {
            object_id: ObjectId {
                id: 2,
                generation: 100,
            },
            object: PdfObject::Stream(PdfStream::new(dict, 0)).into(),
        };

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        obj.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "

                2 100 obj
                <</Length 0>>stream
                endstream
                endobj
                "
            )
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        obj.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            indoc!(
                "

                2 100 obj
                    <<
                        /Length 0
                    >>
                stream
                endstream
                endobj
                "
            )
        );
    }
}
