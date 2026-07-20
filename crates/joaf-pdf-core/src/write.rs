use std::io::Write;

pub trait Formatter {
    fn write_indent<W: Write>(
        &self,
        w: &mut W,
        depth: usize,
        state: &mut WriterState,
    ) -> std::io::Result<()>;
    fn write_newline<W: Write>(&self, w: &mut W, state: &mut WriterState) -> std::io::Result<()>;
    fn write_newline_or_separator<W: Write>(
        &self,
        w: &mut W,
        state: &mut WriterState,
    ) -> std::io::Result<()>;
    fn write_separator<W: Write>(&self, w: &mut W, state: &mut WriterState) -> std::io::Result<()>;
}

pub trait WritePdf {
    fn write_pdf<F: Formatter, W: Write>(&self, w: &mut PdfWriter<'_, W, F>)
    -> std::io::Result<()>;
}

pub struct CompactFormatter;

pub struct PrettyFormatter;

impl Formatter for CompactFormatter {
    #[inline(always)]
    fn write_indent<W: Write>(
        &self,
        _w: &mut W,
        _depth: usize,
        _state: &mut WriterState,
    ) -> std::io::Result<()> {
        Ok(()) // Monomorphized into a no-op: compiler deletes the call
    }

    #[inline(always)]
    fn write_newline<W: Write>(&self, _w: &mut W, _state: &mut WriterState) -> std::io::Result<()> {
        Ok(()) // Monomorphized into a no-op: compiler deletes the call
    }

    #[inline]
    fn write_newline_or_separator<W: Write>(
        &self,
        w: &mut W,
        state: &mut WriterState,
    ) -> std::io::Result<()> {
        *state = WriterState::Whitespace;
        w.write_all(b" ")
    }

    #[inline(always)]
    fn write_separator<W: Write>(
        &self,
        _w: &mut W,
        _state: &mut WriterState,
    ) -> std::io::Result<()> {
        Ok(())
    }
}

impl Formatter for PrettyFormatter {
    #[inline]
    fn write_indent<W: Write>(
        &self,
        w: &mut W,
        depth: usize,
        state: &mut WriterState,
    ) -> std::io::Result<()> {
        for _ in 0..depth {
            w.write_all(b"    ")?;
        }
        *state = WriterState::Whitespace;
        Ok(())
    }

    #[inline]
    fn write_newline<W: Write>(&self, w: &mut W, state: &mut WriterState) -> std::io::Result<()> {
        *state = WriterState::Whitespace;
        w.write_all(b"\n")
    }

    #[inline]
    fn write_newline_or_separator<W: Write>(
        &self,
        w: &mut W,
        state: &mut WriterState,
    ) -> std::io::Result<()> {
        *state = WriterState::Whitespace;
        w.write_all(b"\n")
    }

    #[inline]
    fn write_separator<W: Write>(&self, w: &mut W, state: &mut WriterState) -> std::io::Result<()> {
        *state = WriterState::Whitespace;
        w.write_all(b" ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriterState {
    Start,
    Regular,
    Delimiter,
    Whitespace,
}

pub struct PdfWriter<'a, W: Write, F: Formatter> {
    writer: &'a mut W,
    pub formatter: F,
    pub pure_ascii: bool,
    indent_level: usize,
    state: WriterState,
}
impl<'a, W: Write, F: Formatter> PdfWriter<'a, W, F> {
    pub fn new(writer: &'a mut W, formatter: F, pure_ascii: bool) -> Self {
        Self {
            writer,
            formatter,
            pure_ascii,
            indent_level: 0,
            state: WriterState::Start,
        }
    }

    #[inline(always)]
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    #[inline(always)]
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)
    }

    pub fn write_token(&mut self, data: &[u8]) -> std::io::Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        let first_byte = data[0];
        let start_with_whitespace = is_pdf_whitespace(first_byte);
        let starts_with_regular = !is_pdf_delimiter(first_byte) && !start_with_whitespace;

        // Auto-inject space if both the previous character and current token are "regular"
        if self.state == WriterState::Regular && starts_with_regular {
            self.writer.write_all(b" ")?;
        } else if !start_with_whitespace
            && self.state != WriterState::Start
            && self.state != WriterState::Whitespace
        {
            self.formatter
                .write_separator(self.writer, &mut self.state)?;
        }

        self.writer.write_all(data)?;

        // Update the state based on the last byte of this token
        let last_byte = data[data.len() - 1];
        self.state = if is_pdf_whitespace(last_byte) {
            WriterState::Whitespace
        } else if is_pdf_delimiter(last_byte) {
            WriterState::Delimiter
        } else {
            WriterState::Regular
        };
        Ok(())
    }

    pub fn write_whitespace(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)?;
        self.state = WriterState::Whitespace;
        Ok(())
    }

    pub fn write_integer(&mut self, i: i64) -> std::io::Result<()> {
        if self.state == WriterState::Regular {
            self.writer.write_all(b" ")?;
        }
        write!(self.writer, "{}", i)?;
        self.state = WriterState::Regular;
        Ok(())
    }

    pub fn write_real(&mut self, f: f64) -> std::io::Result<()> {
        if self.state == WriterState::Regular {
            self.writer.write_all(b" ")?;
        }

        if f.fract() == 0.0 {
            write!(self.writer, "{:.1}", f)?; // Force .0 for whole numbers
        } else {
            write!(self.writer, "{}", f)?; // Keep full precision for others
        };

        self.state = WriterState::Regular;
        Ok(())
    }

    #[inline(always)]
    pub fn write_indent(&mut self) -> std::io::Result<()> {
        self.formatter
            .write_indent(self.writer, self.indent_level, &mut self.state)
    }

    #[inline(always)]
    pub fn write_newline(&mut self) -> std::io::Result<()> {
        self.formatter.write_newline(self.writer, &mut self.state)
    }

    #[inline(always)]
    pub fn write_newline_or_separator(&mut self) -> std::io::Result<()> {
        self.formatter
            .write_newline_or_separator(self.writer, &mut self.state)
    }
}

#[inline]
fn is_pdf_delimiter(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

#[inline]
fn is_pdf_whitespace(b: u8) -> bool {
    matches!(b, b'\0' | b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
}

#[cfg(test)]
mod tests {
    use crate::{ObjectId, PdfName, PdfObject};

    use super::*;

    #[test]
    fn test_write_token_with_formatter() {
        let items = vec![
            PdfObject::Null,
            PdfObject::TRUE,
            PdfObject::FALSE,
            PdfObject::Name(PdfName::TYPE),
            PdfObject::Name(PdfName::PAGE),
            PdfObject::Name(PdfName::LENGTH),
            PdfObject::Integer(12345),
            PdfObject::Integer(-12345),
            PdfObject::Real(12345.0),
            PdfObject::Real(-12345.0),
            PdfObject::Real(12345.123),
            PdfObject::Real(12345.123456),
            PdfObject::Reference(ObjectId::new(1, 0)),
            PdfObject::Reference(ObjectId::new(2, 1)),
        ];

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);

        for item in items.clone() {
            item.write_pdf(&mut pdf_writer).unwrap();
        }
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "null true false/Type/Page/Length 12345 -12345 12345.0 -12345.0 12345.123 12345.123456 1 0 R 2 1 R"
        );

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);

        for item in items.clone() {
            item.write_pdf(&mut pdf_writer).unwrap();
        }

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "null true false /Type /Page /Length 12345 -12345 12345.0 -12345.0 12345.123 12345.123456 1 0 R 2 1 R"
        );
    }
}
