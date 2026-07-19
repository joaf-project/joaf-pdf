use joaf_pdf_core::{PdfError, PdfObject, PdfObjectsMap};
use joaf_pdf_dom::{Document, Page};
use joaf_pdf_parser::PdfParser;

pub struct PdfMemoryReader<'a> {
    parser: PdfParser<'a>,
}

impl<'a> PdfMemoryReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            parser: PdfParser::new(bytes),
        }
    }

    pub fn read(&mut self) -> Result<Document<'a>, PdfError> {
        let mut objects = PdfObjectsMap::new();

        self.parser.parse_structure()?;

        let xref_table = self.parser.xref_table.clone();

        for (id, entry) in xref_table.iter() {
            if let PdfObject::IndirectObject(object_id, object) =
                self.parser.parse_object_at(entry.byte_offset)?
            {
                if *id == 0 && entry.generation == 65535 {
                    continue;
                }

                if object_id.id != *id || object_id.generation != entry.generation {
                    return Err(PdfError::from(format!(
                        "Object and xref reference mismatch: {}:{} != {}:{}",
                        object_id.id, object_id.generation, id, entry.generation
                    )));
                }

                objects.insert(object_id, *object);
            }
        }

        let mut doc = Document::try_new(
            self.parser.version.clone(),
            &self.parser.trailer_dict,
            xref_table,
        )?;
        doc.objects = objects;

        let root_dict = doc.objects.get(&doc.trailer.root).to_dict()?;

        if root_dict.get_required("Type")?.to_name()?.str != "Catalog" {
            return Err(PdfError::from("Root dictionary is not a catalog."));
        }

        let pages_dict = root_dict
            .get_required("Pages")?
            .deref(&doc.objects)
            .to_dict()?;
        if pages_dict.get_required("Type")?.to_name()?.str != "Pages" {
            return Err(PdfError::from("Pages dictionary is not a Pages."));
        }

        let page_count = pages_dict.get_required("Count")?.to_integer()? as usize;
        let page_ids = pages_dict.get_required("Kids")?.to_array()?;
        if page_count != page_ids.items.len() {
            return Err(PdfError::from(
                "Page count does not match the number of kids.",
            ));
        }

        for page_id_obj in page_ids.items.iter() {
            let page_dict = page_id_obj.deref(&doc.objects).to_dict()?;
            let page = Page::from_dict(page_dict)?;
            doc.catalog.pages.push(page);
        }

        if let Ok(outlines_id) = root_dict.get_required("Outlines") {
            let outline_dict = outlines_id.deref(&doc.objects).to_dict()?;
            println!("{:#?}", outline_dict);
        }

        Ok(doc)
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
            .map_err(|err| PdfError::from(format!("Failed to open memory mapped file: {}", err)))?;
        let bytes = &mmap
            .as_slice(0, mmap.len())
            .map_err(|err| PdfError::from(format!("Failed to open memory mapped file: {}", err)))?;
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
            File::open("../../tests/data/minimal_pdf_1_4.pdf").map_err(PdfError::from)?;
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).map_err(PdfError::from)?;
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
