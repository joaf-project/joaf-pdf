use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
    io::Write,
};

use crate::{Formatter, PdfWriter, WritePdf};

macro_rules! define_pdf_names {
    ($( ($variant_name:ident, $str_val:expr) ),* $(,)?) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub enum PdfName<'a> {
            $(
                $variant_name,
            )*
            Custom(Cow<'a, str>),
        }

        impl<'a> PdfName<'a> {
            pub fn as_str(&self) -> &str {
                match self {
                    $(
                        Self::$variant_name => $str_val,
                    )*
                    Self::Custom(s) => s.as_ref(),
                }
            }
        }

        impl<'a> From<Cow<'a, str>> for PdfName<'a> {
            fn from(s: Cow<'a, str>) -> Self {
                match s.as_ref() {
                    $(
                        $str_val => Self::$variant_name,
                    )*
                    _ => Self::Custom(s),
                }
            }
        }
    };
}

impl<'a> Default for PdfName<'a> {
    fn default() -> Self {
        Self::Custom(Cow::Borrowed(""))
    }
}

impl<'a> Borrow<str> for PdfName<'a> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Display for PdfName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}", self.as_str())
    }
}

impl<'a> From<&'a str> for PdfName<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(Cow::Borrowed(s))
    }
}

impl<'a> From<String> for PdfName<'a> {
    fn from(s: String) -> Self {
        Self::from(Cow::Owned(s))
    }
}

impl<'a> WritePdf for PdfName<'a> {
    fn write_pdf<F: Formatter, W: Write>(
        &self,
        w: &mut PdfWriter<'_, W, F>,
    ) -> std::io::Result<()> {
        w.write_token(format!("/{}", self.as_str()).as_bytes())
    }
}

impl<'a> PartialEq for PdfName<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<'a> Eq for PdfName<'a> {}

impl<'a> PartialOrd for PdfName<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for PdfName<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'a> std::hash::Hash for PdfName<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<'a> PartialEq<str> for PdfName<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a, 'b> PartialEq<&'b str> for PdfName<'a> {
    fn eq(&self, other: &&'b str) -> bool {
        self.as_str() == *other
    }
}

define_pdf_names! {
    (Annots, "Annots"),
    (ArtBox, "ArtBox"),
    (Ascent, "Ascent"),
    (ASCII85Decode, "ASCII85Decode"),
    (ASCIIHexDecode, "ASCIIHexDecode"),
    (AvgWidth, "AvgWidth"),
    (BaseEncoding, "BaseEncoding"),
    (BaseFont, "BaseFont"),
    (BitsPerComponent, "BitsPerComponent"),
    (BlackIs1, "BlackIs1"),
    (BleedBox, "BleedBox"),
    (CapHeight, "CapHeight"),
    (Catalog, "Catalog"),
    (CCITTFaxDecode, "CCITTFaxDecode"),
    (CharProcs, "CharProcs"),
    (CharSet, "CharSet"),
    (CIDFontType0, "CIDFontType0"),
    (CIDFontType0C, "CIDFontType0C"),
    (CIDFontType2, "CIDFontType2"),
    (CIDSet, "CIDSet"),
    (CIDSystemInfo, "CIDSystemInfo"),
    (CIDToGIDMap, "CIDToGIDMap"),
    (CMap, "CMap"),
    (CMapName, "CMapName"),
    (CMapVersion, "CMapVersion"),
    (Colors, "Colors"),
    (ColorTransform, "ColorTransform"),
    (Columns, "Columns"),
    (Contents, "Contents"),
    (Count, "Count"),
    (Courier, "Courier"),
    (CourierBold, "Courier-Bold"),
    (CourierBoldOblique, "Courier-BoldOblique"),
    (CourierOblique, "Courier-Oblique"),
    (CropBox, "CropBox"),
    (Crypt, "Crypt"),
    (DamagedRowsBeforeError, "DamagedRowsBeforeError"),
    (DCTDecode, "DCTDecode"),
    (DescendantFonts, "DescendantFonts"),
    (Descent, "Descent"),
    (Differences, "Differences"),
    (DW, "DW"),
    (DW2, "DW2"),
    (EarlyChange, "EarlyChange"),
    (EncodedByteAlign, "EncodedByteAlign"),
    (Encoding, "Encoding"),
    (EndOfBlock, "EndOfBlock"),
    (EndOfLine, "EndOfLine"),
    (FD, "FD"),
    (Filter, "Filter"),
    (FirstChar, "FirstChar"),
    (Flags, "Flags"),
    (FlateDecode, "FlateDecode"),
    (Font, "Font"),
    (FontBBox, "FontBBox"),
    (FontDescriptor, "FontDescriptor"),
    (FontFamily, "FontFamily"),
    (FontFile, "FontFile"),
    (FontFile2, "FontFile2"),
    (FontFile3, "FontFile3"),
    (FontMatrix, "FontMatrix"),
    (FontName, "FontName"),
    (FontResources, "FontResources"),
    (FontStretch, "FontStretch"),
    (FontWeight, "FontWeight"),
    (FullScreen, "FullScreen"),
    (Helvetica, "Helvetica"),
    (HelveticaBold, "Helvetica-Bold"),
    (HelveticaBoldOblique, "Helvetica-BoldOblique"),
    (HelveticaOblique, "Helvetica-Oblique"),
    (ID, "ID"),
    (Index, "Index"),
    (Info, "Info"),
    (ItalicAngle, "ItalicAngle"),
    (JBIG2Decode, "JBIG2Decode"),
    (JBIG2Globals, "JBIG2Globals"),
    (JPXDecode, "JPXDecode"),
    (K, "K"),
    (Kids, "Kids"),
    (Lang, "Lang"),
    (LastChar, "LastChar"),
    (Leading, "Leading"),
    (Length, "Length"),
    (Length1, "Length1"),
    (Length2, "Length2"),
    (Length3, "Length3"),
    (LZWDecode, "LZWDecode"),
    (MaxWidth, "MaxWidth"),
    (MediaBox, "MediaBox"),
    (Metadata, "Metadata"),
    (MissingGlyph, "MissingGlyph"),
    (MissingWidth, "MissingWidth"),
    (MMType1, "MMType1"),
    (N, "N"),
    (Name, "Name"),
    (OneColumn, "OneColumn"),
    (OpenType, "OpenType"),
    (Ordering, "Ordering"),
    (Outlines, "Outlines"),
    (Page, "Page"),
    (PageLabels, "PageLabels"),
    (PageLayout, "PageLayout"),
    (PageMode, "PageMode"),
    (Pages, "Pages"),
    (PageTree, "PageTree"),
    (Parent, "Parent"),
    (PDF, "PDF"),
    (Predictor, "Predictor"),
    (Prev, "Prev"),
    (ProcSet, "ProcSet"),
    (Registry, "Registry"),
    (Resources, "Resources"),
    (Root, "Root"),
    (Rotate, "Rotate"),
    (RunLengthDecode, "RunLengthDecode"),
    (SinglePage, "SinglePage"),
    (Size, "Size"),
    (StemH, "StemH"),
    (StemV, "StemV"),
    (Style, "Style"),
    (SubFilter, "SubFilter"),
    (SubType, "SubType"),
    (Supplement, "Supplement"),
    (Symbol, "Symbol"),
    (Text, "Text"),
    (Thumb, "Thumb"),
    (TimesBold, "Times-Bold"),
    (TimesBoldItalic, "Times-BoldItalic"),
    (TimesItalic, "Times-Italic"),
    (TimesRoman, "Times-Roman"),
    (ToUnicode, "ToUnicode"),
    (TrimBox, "TrimBox"),
    (TrueType, "TrueType"),
    (TwoColumnLeft, "TwoColumnLeft"),
    (TwoPageLeft, "TwoPageLeft"),
    (TwoPageRight, "TwoPageRight"),
    (Type, "Type"),
    (Type0, "Type0"),
    (Type1, "Type1"),
    (Type1C, "Type1C"),
    (Type3, "Type3"),
    (UseAttachment, "UseAttachment"),
    (UseCMap, "UseCMap"),
    (UseNone, "UseNone"),
    (UseOC, "UseOC"),
    (UseOutlines, "UseOutlines"),
    (UseThumbs, "UseThumbs"),
    (W, "W"),
    (W2, "W2"),
    (Widths, "Widths"),
    (WMode, "WMode"),
    (XHeight, "XHeight"),
    (XRef, "XRef"),
    (ZapfDingbats, "ZapfDingbats"),
}

#[cfg(test)]
mod tests {
    use crate::{CompactFormatter, PdfObject, PrettyFormatter};

    use super::*;

    #[test]
    fn test_pdf_name_from_str() {
        let name = PdfName::from("Test");
        assert_eq!(name.as_str(), "Test");
        assert_eq!(name, PdfName::Custom(Cow::Borrowed("Test")));

        let type_name = PdfName::from("Type");
        assert_eq!(type_name, PdfName::Type);
    }

    #[test]
    fn test_pdf_name_writet_pdf() {
        let name = PdfObject::Name(PdfName::Type);

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
