use std::{fmt::Display, io::Write};

use crate::{Formatter, PdfWriter, WritePdf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId {
    pub id: u32,
    pub generation: u16,
}

impl Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} R", self.id, self.generation)
    }
}

impl WritePdf for ObjectId {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        w.write_token(&format!("{} {} R", self.id, self.generation).as_bytes())
    }
}

impl ObjectId {
    pub fn new(id: u32, generation: u16) -> Self {
        Self { id, generation }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompactFormatter, PrettyFormatter};

    #[test]
    fn test_object_id_write_pdf() {
        let id = ObjectId {
            id: 1,
            generation: 2,
        };

        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, CompactFormatter, false);
        id.write_pdf(&mut writer).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "1 2 R");

        let mut buffer = Vec::new();
        let mut writer = PdfWriter::new(&mut buffer, PrettyFormatter, false);
        id.write_pdf(&mut writer).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "1 2 R");
    }
}
