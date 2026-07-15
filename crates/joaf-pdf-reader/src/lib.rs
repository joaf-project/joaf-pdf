use std::collections::BTreeMap;

use joaf_pdf_core::{PdfDictionary, PdfError, XrefEntry};
use joaf_pdf_dom::PdfDocument;

pub struct PdfReader<'a> {
    pub document: PdfDocument,
    bytes: &'a [u8],
    pos: usize,
    len: usize,
}

impl<'a> PdfReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            document: PdfDocument::new(),
            bytes,
            pos: 0,
            len: bytes.len(),
        }
    }

    pub fn read(&mut self) -> Result<(), PdfError> {
        self.read_version()?;
        self.read_trailer()?;

        Ok(())
    }

    fn discard_line(&mut self) -> Result<(), PdfError> {
        loop {
            if self.pos >= self.len {
                return Err(PdfError::new("Unexpected EOF"));
            }
            let byte = self.bytes[self.pos];
            self.pos += 1;

            match byte {
                b'\r' | b'\n' => break,
                _ => continue,
            }
        }

        Ok(())
    }

    fn discard_whitespace(&mut self) -> Result<(), PdfError> {
        loop {
            if self.pos >= self.len {
                return Err(PdfError::new("Unexpected EOF"));
            }
            let byte = self.bytes[self.pos];
            self.pos += 1;

            match byte {
                b' ' | b'\t' | b'\r' | b'\n' => continue,
                b'%' => self.discard_line()?,
                _ => {
                    self.pos -= 1;
                    break;
                }
            }
        }

        Ok(())
    }

    fn read_line(&mut self, buffer: &mut String) -> Result<(), PdfError> {
        self.discard_whitespace()?;

        buffer.clear();

        loop {
            if self.pos >= self.len {
                return Err(PdfError::new("Unexpected EOF"));
            }
            let byte = self.bytes[self.pos];
            self.pos += 1;

            match byte {
                b'\r' | b'\n' => break,
                _ => buffer.push(byte as char),
            }
        }

        Ok(())
    }

    fn read_version(&mut self) -> Result<(), PdfError> {
        if !self.bytes.starts_with(b"%PDF-") {
            return Err(PdfError::new("No %PDF- header found."));
        }

        let mut version = String::new();
        for b in self.bytes[5..].iter() {
            match b {
                b'0'..=b'9' => version.push(*b as char),
                b'.' => version.push(*b as char),
                _ => break,
            }
        }

        self.document.version = version;
        Ok(())
    }

    fn read_trailer(&mut self) -> Result<(), PdfError> {
        const EOF_MARKER: &[u8] = b"%%EOF";
        const START_XREF: &[u8] = b"startxref";
        let mut line_buffer = String::new();

        let position = self
            .bytes
            .windows(EOF_MARKER.len())
            .rposition(|window| window == EOF_MARKER)
            .ok_or(PdfError::new("No %%EOF marker found."))?;

        let position = self.bytes[..position]
            .windows(START_XREF.len())
            .rposition(|window| window == START_XREF)
            .ok_or(PdfError::new("No startxref found."))?;

        self.pos = position + START_XREF.len();
        self.read_line(&mut line_buffer)?;

        let trailer_pos = line_buffer
            .trim()
            .parse::<usize>()
            .map_err(|_| PdfError::new("Invalid trailer position."))?;

        self.pos = trailer_pos;

        let mut line_buffer = String::new();

        self.discard_whitespace()?;
        self.read_line(&mut line_buffer)?;
        if line_buffer.trim() != "xref" {
            return Err(PdfError::new("xref section missing."));
        }

        self.read_line(&mut line_buffer)?;

        let xref_parts = line_buffer
            .split_whitespace()
            .map(|x| x.parse::<u32>())
            .collect::<Result<Vec<u32>, _>>()
            .map_err(|_| PdfError::new("Invalid xref section."))?;

        if xref_parts.len() != 2 {
            return Err(PdfError::new("Invalid xref section header."));
        }

        let xref_start_obj = xref_parts[0];
        let xref_section_count = xref_parts[1];

        let mut xref_table: BTreeMap<u32, XrefEntry> = BTreeMap::new();

        for xref_section_index in 0..xref_section_count {
            self.read_line(&mut line_buffer)?;

            let xref_section = line_buffer.split_whitespace().collect::<Vec<&str>>();

            if xref_section.len() != 3 {
                return Err(PdfError::new("Invalid xref section: invalid format."));
            }

            let position = xref_section[0]
                .parse::<usize>()
                .map_err(|_| PdfError::new("Invalid xref section: Invalid position."))?;
            let generation = xref_section[1]
                .parse::<usize>()
                .map_err(|_| PdfError::new("Invalid xref section: Invalid generation."))?;
            let is_in_use = xref_section[2] == "n";

            if xref_start_obj == 0 && position == 0 {
                if generation != 65535 || is_in_use {
                    return Err(PdfError::new(
                        "Invalid xref section: First entry must be free object.",
                    ));
                }
                continue;
            }

            xref_table.insert(
                xref_start_obj + xref_section_index,
                XrefEntry {
                    byte_offset: position as u64,
                    generation: generation as u16,
                    in_use: is_in_use,
                },
            );
        }

        self.read_line(&mut line_buffer)?;
        if line_buffer.trim() != "trailer" {
            return Err(PdfError::new("trailer section missing."));
        }

        self.document.xref_table = xref_table;
        Ok(())
    }

    fn read_dictionary(&mut self, buffer: &str) -> Result<PdfDictionary, PdfError> {
        let mut dictionary = PdfDictionary::new();
        Ok(dictionary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mmap_io::MemoryMappedFile;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn load_sample_pdf14_mmap_test() -> Result<(), PdfError> {
        let mmap = MemoryMappedFile::open_ro("../../tests/data/minimal_pdf_1_4.pdf")
            .map_err(|err| PdfError::new(&format!("Failed to open memory mapped file: {}", err)))?;
        let bytes = &mmap
            .as_slice(0, mmap.len())
            .map_err(|err| PdfError::new(&format!("Failed to open memory mapped file: {}", err)))?;
        let mut pdf_reader = PdfReader::new(bytes);
        pdf_reader.read()?;

        println!("PDF version: {}", pdf_reader.document.version);
        for (key, value) in pdf_reader.document.xref_table.iter() {
            println!("Xref entry: {} = {:#?}", key, value);
        }

        Ok(())
    }

    #[test]
    fn load_sample_pdf14_test() -> Result<(), PdfError> {
        let mut file =
            File::open("../../tests/data/minimal_pdf_1_4.pdf").map_err(PdfError::from_io_error)?;
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(PdfError::from_io_error)?;
        let mut pdf_reader = PdfReader::new(&buf);
        pdf_reader.read()?;

        println!("PDF version: {}", pdf_reader.document.version);
        for (key, value) in pdf_reader.document.xref_table.iter() {
            println!("Xref entry: {} = {:#?}", key, value);
        }

        Ok(())
    }
}
