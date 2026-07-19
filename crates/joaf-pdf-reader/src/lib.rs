use joaf_pdf_core::{ObjectId, PdfError, PdfObject, PdfObjectsMap};
use joaf_pdf_dom::{Catalog, Document, Page, Trailer};
use joaf_pdf_parser::PdfParser;

pub struct PdfMemoryReader<'a> {
    parser: PdfParser<'a>,
    objects: PdfObjectsMap,
}

impl<'a> PdfMemoryReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            parser: PdfParser::new(bytes),
            objects: PdfObjectsMap::new(),
        }
    }

    pub fn read(&'a mut self) -> Result<Document<'a>, PdfError> {
        let version = self.read_version()?;
        let (xref_table, trailer_dict) = self.parser.parse_trailer()?;

        let trailer = Trailer::from(&trailer_dict)?;

        for (id, entry) in xref_table.iter() {
            if let PdfObject::IndirectObject(object_id, object) =
                self.parser.parse_object_at(entry.byte_offset)?
            {
                if *id == 0 && entry.generation == 65535 {
                    continue;
                }

                if object_id.id != *id || object_id.generation != entry.generation {
                    return Err(PdfError::invalid_reference(&format!(
                        "Invalid object id or generation: {}:{} != {}:{}",
                        object_id.id, object_id.generation, id, entry.generation
                    )));
                }

                self.objects.insert(object_id, *object);
            }
        }

        let mut catalog = Catalog::new();

        let root_dict = self.objects.get(&trailer.root).to_dict()?;

        if root_dict.get("Type")?.to_name()? != "Catalog" {
            return Err(PdfError::new("Root dictionary is not a catalog."));
        }

        let pages_dict = root_dict.get("Pages")?.deref(&self.objects).to_dict()?;
        if pages_dict.get("Type")?.to_name()? != "Pages" {
            return Err(PdfError::new("Pages dictionary is not a Pages."));
        }

        let page_count = pages_dict.get("Count")?.to_integer()? as usize;
        let page_ids = pages_dict.get("Kids")?.to_array()?;
        if page_count != page_ids.items.len() {
            return Err(PdfError::new(
                "Page count does not match the number of kids.",
            ));
        }

        for page_id_obj in page_ids.items.iter() {
            let page_dict = page_id_obj.deref(&self.objects).to_dict()?;
            let page = Page::from_dict(&page_dict)?;
            catalog.pages.push(page);
        }

        if let Ok(outlines_id) = root_dict.get("Outlines") {
            let outline_dict = outlines_id.deref(&self.objects).to_dict()?;
            println!("{:#?}", outline_dict);
        }

        Ok(Document {
            version,
            catalog,
            trailer,
            xref_table,
        })
    }

    fn read_version(&mut self) -> Result<String, PdfError> {
        if !self.parser.lexer.input.starts_with(b"%PDF-") {
            return Err(PdfError::new("No %PDF- header found."));
        }

        let mut version = String::new();
        for b in self.parser.lexer.input[5..].iter() {
            match b {
                b'0'..=b'9' => version.push(*b as char),
                b'.' => version.push(*b as char),
                _ => break,
            }
        }

        Ok(version)
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
        let mut pdf_reader = PdfMemoryReader::new(bytes);
        let document = pdf_reader.read()?;

        println!("PDF version: {}", document.version);
        for (key, value) in document.xref_table.iter() {
            println!("Xref entry: {} = {:#?}", key, value);
        }

        assert!(document.catalog.pages.len() == 1);

        Ok(())
    }

    #[test]
    fn load_sample_pdf14_test() -> Result<(), PdfError> {
        let mut file =
            File::open("../../tests/data/minimal_pdf_1_4.pdf").map_err(PdfError::from_io_error)?;
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(PdfError::from_io_error)?;
        let mut pdf_reader = PdfMemoryReader::new(&buf);
        let document = pdf_reader.read()?;

        println!("PDF version: {}", document.version);
        for (key, value) in document.xref_table.iter() {
            println!("Xref entry: {} = {:#?}", key, value);
        }

        assert!(document.catalog.pages.len() == 1);

        Ok(())
    }
}
