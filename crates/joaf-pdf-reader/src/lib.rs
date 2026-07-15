use std::{
    collections::HashMap,
    io::{BufRead, Seek, SeekFrom},
};

use joaf_pdf_core::{PdfDictionary, PdfError, XrefEntry};
use joaf_pdf_dom::{PdfCatalog, PdfDocument};

pub struct PdfReader {}

impl PdfReader {
    pub fn read<T: BufRead + Seek>(reader: &mut T) -> Result<PdfDocument, PdfError> {
        reader
            .seek(SeekFrom::Start(0))
            .map_err(PdfError::from_io_error)?;

        let mut line_buffer = String::new();
        reader
            .read_line(&mut line_buffer)
            .map_err(PdfError::from_io_error)?;

        if !line_buffer.starts_with("%PDF-") {
            return Err(PdfError::new("No %PDF- header found."));
        }

        let mut version = String::new();
        for c in line_buffer[5..].chars() {
            match c {
                '0'..='9' => version.push(c),
                '.' => version.push(c),
                _ => break,
            }
        }

        let file_len = reader
            .seek(SeekFrom::End(0))
            .map_err(PdfError::from_io_error)?;

        let chunk_size = std::cmp::min(1024, file_len as usize);

        reader
            .seek(SeekFrom::End(-(chunk_size as i64)))
            .map_err(PdfError::from_io_error)?;

        let mut buffer: Vec<u8> = vec![0; chunk_size as usize];
        reader
            .read_exact(&mut buffer)
            .map_err(PdfError::from_io_error)?;

        let lines = buffer
            .trim_ascii_end()
            .rsplit(|x| *x == b'\n' || *x == b'\r')
            .map(|x| {
                str::from_utf8(x)
                    .map(|s| s.trim())
                    .map_err(PdfError::from_utf8_error)
            })
            .take_while(|x| match x {
                Ok(s) => *s != "startxref",
                Err(_) => true,
            })
            .collect::<Result<Vec<&str>, _>>()?;

        if lines.len() != 2 || lines[0] != "%%EOF" {
            return Err(PdfError::new("No %%EOF at end of file."));
        }

        let trailer_pos = lines[1]
            .parse::<usize>()
            .map_err(|_| PdfError::new("Invalid trailer position."))?;

        reader
            .seek(SeekFrom::Start(trailer_pos as u64))
            .map_err(PdfError::from_io_error)?;

        let mut line_buffer = String::new();

        let mut read_next_line = |buf: &mut String| -> Result<bool, PdfError> {
            loop {
                buf.clear();
                let bytes_read = reader.read_line(buf).map_err(PdfError::from_io_error)?;

                if bytes_read == 0 {
                    return Ok(false);
                }

                if !buf.trim().is_empty() {
                    return Ok(true);
                }
            }
        };

        if !read_next_line(&mut line_buffer)? || line_buffer.trim() != "xref" {
            return Err(PdfError::new("xref section missing."));
        }

        if !read_next_line(&mut line_buffer)? {
            return Err(PdfError::new("xref section header missing."));
        }

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

        let mut xref_table: HashMap<u32, XrefEntry> = HashMap::new();

        for xref_section_index in 0..xref_section_count {
            if !read_next_line(&mut line_buffer)? {
                return Err(PdfError::new("xref table extends beyond EOF."));
            }

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

        let trailer = PdfDictionary::new();
        let catalog = PdfCatalog { pages: Vec::new() };

        Ok(PdfDocument {
            version,
            catalog,
            trailer,
            xref_table,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn load_sample_pdf14_test() -> Result<(), PdfError> {
        let file = File::open("../../tests/data/minimal_pdf_1_4.pdf")
            .map_err(|err| PdfError::from_io_error(err))?;
        let mut reader = BufReader::new(file);
        let pdf = PdfReader::read(&mut reader)?;

        println!("PDF version: {}", pdf.version);
        for (key, value) in pdf.xref_table.iter() {
            println!("Xref entry: {} = {:#?}", key, value);
        }

        Ok(())
    }
}
