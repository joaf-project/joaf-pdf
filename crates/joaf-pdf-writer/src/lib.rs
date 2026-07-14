use std::io::Write;

use indoc::formatdoc;
use indoc::indoc;

pub struct Writer {
    stream: Box<dyn Write>,
    pos: usize,
}

impl Writer {
    pub fn new(stream: Box<dyn Write>) -> Self {
        Self { stream, pos: 0 }
    }

    pub fn write_sample_pdf14(&mut self) -> Result<bool, std::io::Error> {
        let mut xref_table: Vec<usize> = Vec::new();

        self.write(b"%PDF-1.4\n")?;
        self.write(b"%\xE2\xE3\xCF\xD3\n")?;

        self.write(b"\n\n")?;
        self.write(
            indoc!(
                r#"
                % This is a minimal valid PDF 1.4 document.
                % It contains only one page and a simple text string.
                % It also has a lot of writespace to make easier for humans to read.
                % And, of course, JOAF PDF Rocks!
                "#
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        xref_table.push(self.pos);
        self.write(
            indoc!(
                r#"
                % Document Catalog
                1 0 obj
                    <<
                        /Type     /Catalog
                        /Outlines 2 0 R
                        /Pages    3 0 R
                    >>
                endobj
                "#
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        xref_table.push(self.pos);
        self.write(
            indoc!(
                r#"
                % Outline Dictionary
                2 0 obj
                    <<
                        /Type  /Outlines
                        /Count 0
                    >>
                endobj
                "#
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        xref_table.push(self.pos);
        self.write(
            indoc!(
                r#"
                % Page Tree Node
                3 0 obj
                    <<
                        /Type  /Pages
                        /Kids  [4 0 R]
                        /Count 1
                    >>
                endobj
                "#
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        xref_table.push(self.pos);
        self.write(
            indoc!(
                r#"
                % Page Object
                4 0 obj
                    <<
                        /Type       /Page
                        /MediaBox   [0 0 595 842]
                        /Contents   5 0 R
                        /Resources
                            <<
                                /ProcSet 6 0 R
                                /Font
                                    <<
                                        /F1 7 0 R
                                    >>
                            >>
                    >>
                endobj
                "#
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        xref_table.push(self.pos);
        let stream_contents = indoc!(
            r#"
                BT
                /F1 24 Tf
                100 700 Td
                (JOAF PDF Rocks) Tj
                ET
            "#
        );
        self.write(
            formatdoc!(
                r#"
                % Page Content Stream
                5 0 obj
                    <<
                        /Length  {}
                    >>
                stream
                {}
                endstream
                endobj
                "#,
                stream_contents.len(),
                stream_contents
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        xref_table.push(self.pos);
        self.write(
            indoc!(
                r#"
                % Procedure Set Array
                6 0 obj
                    [/PDF /Text]
                endobj
                "#
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        xref_table.push(self.pos);
        self.write(
            indoc!(
                r#"
                % Type 1 Font Object (Not Embedded)
                7 0 obj
                    <<
                        /Type     /Font
                        /Subtype  /Type1
                        /Name     /F1
                        /BaseFont /Helvetica
                        /Encoding /MacRomanEncoding
                    >>
                endobj
                "#
            )
            .as_bytes(),
        )?;

        self.write(b"\n\n")?;
        let startxref = self.pos;

        self.write(b"xref\n")?;
        self.write(format!("0 {}\n", xref_table.len() + 1).as_bytes())?;
        self.write(b"0000000000 65535 f\n")?;
        for (_, pos) in xref_table.iter().enumerate() {
            self.write(format!("{:010} {:05} n\n", pos, 0).as_bytes())?;
        }

        self.write(b"\n\n")?;
        self.write(
            formatdoc!(
                r#"
                trailer
                    <<
                        /Size {}
                        /Root 1 0 R
                    >>
                "#,
                xref_table.len() + 1
            )
            .as_bytes(),
        )?;
        self.write(b"\n\n")?;
        self.write(b"startxref\n")?;
        self.write(format!("{}\n", startxref).as_bytes())?;
        self.write(b"%%EOF\n")?;

        Ok(true)
    }

    fn write(&mut self, data: &[u8]) -> Result<bool, std::io::Error> {
        self.stream.write_all(data)?;
        self.pos += data.len();

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn write_sample_pdf14_test() -> Result<(), std::io::Error> {
        let buffer = File::create("minimal_pdf_1_4.pdf")?;
        let mut writer = Writer::new(Box::new(buffer));
        writer.write_sample_pdf14()?;
        Ok(())
    }
}
