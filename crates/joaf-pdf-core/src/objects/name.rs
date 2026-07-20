use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
    io::Write,
};

use crate::{Formatter, PdfWriter, WritePdf};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PdfName<'a> {
    pub str: Cow<'a, str>,
}

impl<'a> Borrow<str> for PdfName<'a> {
    fn borrow(&self) -> &str {
        &self.str
    }
}

impl<'a> Display for PdfName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}", self.str)
    }
}

impl<'a> From<&'a str> for PdfName<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(Cow::Borrowed(s))
    }
}

impl<'a> WritePdf for PdfName<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        w.write_token(&format!("/{}", self.str).as_bytes())
    }
}

macro_rules! define_pdf_names {
    ($( ($const_name:ident, $str_val:expr) ),* $(,)?) => {
        impl<'a> PdfName<'a> {
            $(
                pub const $const_name: PdfName<'static> = PdfName {
                    str: Cow::Borrowed($str_val),
                };
            )*
        }
        impl<'a> From<Cow<'a, str>> for PdfName<'a> {
            fn from(s: Cow<'a, str>) -> Self {
                match s.as_ref() {
                    $(
                        $str_val => PdfName::$const_name,
                    )*
                    _ => {
                        println!("Constructing new PdfName: {}", s); // TODO: Remove this
                        PdfName { str: s }
                    }
                }
            }
        }
    };
}

define_pdf_names! {
    (ASCII85_DECODE, "ASCII85Decode"),
    (ASCIIHEX_DECODE, "ASCIIHexDecode"),
    (ANNOTS, "Annots"),
    (ART_BOX, "ArtBox"),
    (ASCENT, "Ascent"),
    (AVG_WIDTH, "AvgWidth"),
    (BASE_ENCODING, "BaseEncoding"),
    (BASE_FONT, "BaseFont"),
    (BITS_PER_COMPONENT, "BitsPerComponent"),
    (BLACK_IS_1, "BlackIs1"),
    (BLEED_BOX, "BleedBox"),
    (CAP_HEIGHT, "CapHeight"),
    (CATALOG, "Catalog"),
    (CHAR_PROCS, "CharProcs"),
    (CHAT_SET, "CharSet"),
    (CID_FONT_TYPE0, "CIDFontType0"),
    (CID_FONT_TYPE0C, "CIDFontType0C"),
    (CID_FONT_TYPE2, "CIDFontType2"),
    (CID_SET, "CIDSet"),
    (CID_SYSTEM_INFO, "CIDSystemInfo"),
    (CID_TO_GID_MAP, "CIDToGIDMap"),
    (CCITTFAX_DECODE, "CCITTFaxDecode"),
    (CMAP, "CMap"),
    (CMAP_NAME, "CMapName"),
    (CMAP_VERSION, "CMapVersion"),
    (COLOR_TRANSFORM, "ColorTransform"),
    (COLORS, "Colors"),
    (COLUMNS, "Columns"),
    (CONTENTS, "Contents"),
    (COUNT, "Count"),
    (COURIER, "Courier"),
    (COURIER_BOLD, "Courier-Bold"),
    (COURIER_BOLD_OBLIQUE, "Courier-BoldOblique"),
    (COURIER_OBLIQUE, "Courier-Oblique"),
    (CROP_BOX, "CropBox"),
    (CRYPT, "Crypt"),
    (DAMAGED_ROWS_BEFORE_ERROR, "DamagedRowsBeforeError"),
    (DCT_DECODE, "DCTDecode"),
    (DESCENDANT_FONTS, "DescendantFonts"),
    (DESCENT, "Descent"),
    (DIFFERENCES, "Differences"),
    (DW, "DW"),
    (DW2, "DW2"),
    (EARLY_CHANGE, "EarlyChange"),
    (ENCODED_BYTE_ALIGN, "EncodedByteAlign"),
    (ENCODING, "Encoding"),
    (END_OF_BLOCK, "EndOfBlock"),
    (END_OF_LINE, "EndOfLine"),
    (FD, "FD"),
    (FILTER, "Filter"),
    (FIRST_CHAR, "FirstChar"),
    (FLAGS, "Flags"),
    (FLATE_DECODE, "FlateDecode"),
    (FONT, "Font"),
    (FONT_BBOX, "FontBBox"),
    (FONT_DESCRIPTOR, "FontDescriptor"),
    (FONT_FAMILY, "FontFamily"),
    (FONT_FILE, "FontFile"),
    (FONT_FILE2, "FontFile2"),
    (FONT_FILE3, "FontFile3"),
    (FONT_MATRIX, "FontMatrix"),
    (FONT_NAME, "FontName"),
    (FONT_RESOURCES, "FontResources"),
    (FONT_STRETCH, "FontStretch"),
    (FONT_WEIGHT, "FontWeight"),
    (HELVETICA, "Helvetica"),
    (HELVETICA_BOLD, "Helvetica-Bold"),
    (HELVETICA_BOLD_OBLIQUE, "Helvetica-BoldOblique"),
    (HELVETICA_OBLIQUE, "Helvetica-Oblique"),
    (ID, "ID"),
    (INDEX, "Index"),
    (INFO, "Info"),
    (ITALIC_ANGLE, "ItalicAngle"),
    (JBIG2_DECODE, "JBIG2Decode"),
    (JBIG2_GLOBALS, "JBIG2Globals"),
    (JPX_DECODE, "JPXDecode"),
    (K, "K"),
    (KIDS, "Kids"),
    (LANG, "Lang"),
    (LAST_CHAR, "LastChar"),
    (LEADING, "Leading"),
    (LENGTH, "Length"),
    (LENGTH1, "Length1"),
    (LENGTH2, "Length2"),
    (LENGTH3, "Length3"),
    (LZW_DECODE, "LZWDecode"),
    (MAX_WIDTH, "MaxWidth"),
    (MEDIA_BOX, "MediaBox"),
    (METADATA, "Metadata"),
    (MISSING_GLYPH, "MissingGlyph"),
    (MISSING_WIDTH, "MissingWidth"),
    (MM_TYPE1, "MMType1"),
    (N, "N"),
    (NAME, "Name"),
    (OPEN_TYPE, "OpenType"),
    (ORDERING, "Ordering"),
    (OUTLINES, "Outlines"),
    (PAGE, "Page"),
    (PAGE_LABELS, "PageLabels"),
    (PAGE_LAYOUT, "PageLayout"),
    (PAGE_MODE, "PageMode"),
    (PAGES, "Pages"),
    (PARENT, "Parent"),
    (PDF, "PDF"),
    (PREDICTOR, "Predictor"),
    (PREV, "Prev"),
    (PROC_SET, "ProcSet"),
    (REGISTRY, "Registry"),
    (RESOURCES, "Resources"),
    (ROOT, "Root"),
    (ROTATE, "Rotate"),
    (RUN_LENGTH_DECODE, "RunLengthDecode"),
    (SIZE, "Size"),
    (STEM_H, "StemH"),
    (STEM_V, "StemV"),
    (STYLE, "Style"),
    (SUB_FILTER, "SubFilter"),
    (SUB_TYPE, "SubType"),
    (SUPPLEMENT, "Supplement"),
    (SYMBOL, "Symbol"),
    (TEXT, "Text"),
    (THUMB, "Thumb"),
    (TIMES_BOLD, "Times-Bold"),
    (TIMES_BOLD_ITALIC, "Times-BoldItalic"),
    (TIMES_ITALIC, "Times-Italic"),
    (TIMES_ROMAN, "Times-Roman"),
    (TO_UNICODE, "ToUnicode"),
    (TRIM_BOX, "TrimBox"),
    (TRUE_TYPE, "TrueType"),
    (TYPE, "Type"),
    (TYPE0, "Type0"),
    (TYPE1, "Type1"),
    (TYPE1C, "Type1C"),
    (TYPE3, "Type3"),
    (USE_CMAP, "UseCMap"),
    (W, "W"),
    (W2, "W2"),
    (WIDTHS, "Widths"),
    (WMODE, "WMode"),
    (XHEIGHT, "XHeight"),
    (XREF, "XRef"),
    (ZAPF_DINGBATS, "ZapfDingbats"),
}

#[cfg(test)]
mod tests {
    use crate::{CompactFormatter, PdfObject, PrettyFormatter};

    use super::*;

    #[test]
    fn test_pdf_name_from_str() {
        let name = PdfName::from("Test");
        assert_eq!(name.str, "Test");
    }

    #[test]
    fn test_pdf_name_writet_pdf() {
        let name = PdfObject::Name(PdfName::TYPE);

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, CompactFormatter, false);
        name.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "/Type");

        let mut writer = Vec::new();
        let mut pdf_writer = PdfWriter::new(&mut writer, PrettyFormatter, false);
        name.write_pdf(&mut pdf_writer).unwrap();
        assert_eq!(String::from_utf8(writer).unwrap(), "/Type");
    }
}
