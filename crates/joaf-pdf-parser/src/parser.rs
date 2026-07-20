use joaf_pdf_core::*;

use crate::lexer::{Lexer, Token};

pub struct PdfParser<'a> {
    pub lexer: Lexer<'a>,
    pub version: String,
    pub xref_table: XrefTable,
    pub trailer_dict: PdfDictionary<'a>,
}

impl<'a> PdfParser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        PdfParser {
            lexer: Lexer::new(input),
            version: String::new(),
            xref_table: XrefTable::new(),
            trailer_dict: PdfDictionary::new(),
        }
    }

    pub fn set_position(&mut self, pos: usize) {
        self.lexer.pos = pos;
    }

    pub fn parse_structure(&mut self) -> Result<(), PdfError> {
        self.version = self.parse_version()?;
        let (xref_table, trailer_dict) = self.parse_trailer()?;
        self.xref_table = xref_table;
        self.trailer_dict = trailer_dict;
        Ok(())
    }

    pub fn parse_indirect_object(
        &mut self,
        id: u32,
        generation: u16,
    ) -> Result<PdfObject<'a>, PdfError> {
        let entry = self
            .xref_table
            .get(&id)
            .ok_or_else(|| PdfError::invalid_reference(id, generation))?;

        if let PdfObject::IndirectObject { object_id, object } =
            self.parse_object_at(entry.byte_offset)?
        {
            if object_id.id != id || object_id.generation != generation {
                return Err(PdfError::invalid_reference(
                    object_id.id,
                    object_id.generation,
                ));
            }

            Ok(*object)
        } else {
            Err(PdfError::invalid_reference(id, generation))
        }
    }

    pub fn parse_object_at(&mut self, position: usize) -> Result<PdfObject<'a>, PdfError> {
        let saved_pos = self.lexer.get_position();
        self.lexer.set_position(position);
        let obj = self.parse_object();
        self.lexer.set_position(saved_pos);
        return obj;
    }

    pub fn parse_object(&mut self) -> Result<PdfObject<'a>, PdfError> {
        let token = self.lexer.next_token()?;

        match token {
            Token::Name(name) => Ok(PdfObject::Name(PdfName::from(name))),
            Token::Integer(i) => {
                let pos = self.lexer.pos;

                // Check for a indirect reference (0 0 R) or object (0 0 obj)
                if let Token::Integer(generation) = self.lexer.next_token()? {
                    if let Token::Keyword(kw) = self.lexer.next_token()? {
                        match kw {
                            "R" => {
                                return Ok(PdfObject::Reference(ObjectId {
                                    id: i as u32,
                                    generation: generation as u16,
                                }));
                            }
                            "obj" => {
                                let obj = self.parse_object()?;
                                return Ok(PdfObject::IndirectObject {
                                    object_id: ObjectId {
                                        id: i as u32,
                                        generation: generation as u16,
                                    },
                                    object: Box::new(obj),
                                });
                            }
                            _ => {}
                        }
                    }
                }

                self.lexer.pos = pos;
                Ok(PdfObject::Integer(i))
            }
            Token::Real(r) => Ok(PdfObject::Real(r)),
            Token::LiteralString(s) => Ok(PdfObject::String(PdfString { bytes: s })),
            Token::HexString(s) => Ok(PdfObject::String(PdfString { bytes: s })),
            Token::Keyword(kw) => match kw {
                "true" => Ok(PdfObject::TRUE),
                "false" => Ok(PdfObject::FALSE),
                "null" => Ok(PdfObject::NULL),
                _ => Err(PdfError::from("Invalid token.")),
            },
            Token::BracketOpen => Ok(self.parse_array()?),
            Token::DictOpen => Ok(self.parse_dict_or_stream()?),
            _ => Err(PdfError::from("Invalid token.")),
        }
    }

    pub fn parse_trailer(&mut self) -> Result<(XrefTable, PdfDictionary<'a>), PdfError> {
        const EOF_MARKER: &[u8] = b"%%EOF";
        const START_XREF: &[u8] = b"startxref";

        let position = self
            .lexer
            .input
            .windows(EOF_MARKER.len())
            .rposition(|window| window == EOF_MARKER)
            .ok_or(PdfError::from("No %%EOF marker found."))?;

        let position = self.lexer.input[..position]
            .windows(START_XREF.len())
            .rposition(|window| window == START_XREF)
            .ok_or(PdfError::from("No startxref found."))?;

        self.lexer.pos = position + START_XREF.len();
        if let Token::Integer(position) = self.lexer.next_token()? {
            self.lexer.pos = position as usize;
        } else {
            return Err(PdfError::from("Invalid trailer position."));
        }

        let xref_table = self.parse_xref_table()?;

        if let Token::Keyword(kw) = self.lexer.next_token()? {
            if kw != "trailer" {
                return Err(PdfError::from("Invalid trailer."));
            }
        } else {
            return Err(PdfError::from("Invalid trailer."));
        }

        if let PdfObject::Dictionary(trailer) = self.parse_object()? {
            if let Ok(size_obj) = trailer.get_required(&PdfName::SIZE) {
                if let PdfObject::Integer(size) = size_obj {
                    if xref_table.len() != (*size) as usize {
                        return Err(PdfError::from("Invalid trailer."));
                    }
                } else {
                    return Err(PdfError::from("Invalid trailer."));
                }
            } else {
                return Err(PdfError::from("Invalid trailer."));
            }

            Ok((xref_table, trailer))
        } else {
            Err(PdfError::from("Invalid trailer."))
        }
    }

    fn parse_version(&mut self) -> Result<String, PdfError> {
        if !self.lexer.input.starts_with(b"%PDF-") {
            return Err(PdfError::from("No %PDF- header found."));
        }

        let mut version = String::new();
        for b in self.lexer.input[5..].iter() {
            match b {
                b'0'..=b'9' => version.push(*b as char),
                b'.' => version.push(*b as char),
                _ => break,
            }
        }

        Ok(version)
    }

    fn parse_xref_table(&mut self) -> Result<XrefTable, PdfError> {
        if let Token::Keyword(kw) = self.lexer.next_token()? {
            if kw != "xref" {
                return Err(PdfError::from("Invalid xref table."));
            }
        } else {
            return Err(PdfError::from("Invalid xref table."));
        }

        let section_start: u32;
        let section_count: u32;

        if let Token::Integer(value) = self.lexer.next_token()? {
            section_start = value as u32;
        } else {
            return Err(PdfError::from("Invalid xref table."));
        }

        if let Token::Integer(value) = self.lexer.next_token()? {
            section_count = value as u32;
        } else {
            return Err(PdfError::from("Invalid xref table."));
        }

        let mut xref_table = XrefTable::new();

        for section_index in 0..section_count {
            let position: usize;
            let generation: u16;
            let keyword: &str;

            if let Token::Integer(value) = self.lexer.next_token()? {
                position = value as usize;
            } else {
                return Err(PdfError::from("Invalid xref table."));
            }

            if let Token::Integer(value) = self.lexer.next_token()? {
                generation = value as u16;
            } else {
                return Err(PdfError::from("Invalid xref table."));
            }

            if let Token::Keyword(value) = self.lexer.next_token()? {
                keyword = value;
            } else {
                return Err(PdfError::from("Invalid xref table."));
            }

            xref_table.insert(
                section_start + section_index,
                XrefEntry {
                    byte_offset: position as usize,
                    generation: generation as u16,
                    in_use: keyword == "n",
                },
            );
        }

        Ok(xref_table)
    }

    fn parse_array(&mut self) -> Result<PdfObject<'a>, PdfError> {
        let mut array: Vec<PdfObject<'a>> = Vec::new();
        loop {
            let token = self.lexer.peek_token()?;
            match token {
                Token::BracketClose => {
                    self.lexer.next_token()?;
                    return Ok(PdfObject::Array(PdfArray::from(array)));
                }
                _ => {
                    array.push(self.parse_object()?);
                }
            }
        }
    }

    fn parse_dict_or_stream(&mut self) -> Result<PdfObject<'a>, PdfError> {
        let mut dict = PdfDictionary::new();
        loop {
            let token = self.lexer.next_token()?;
            match token {
                Token::Name(name) => {
                    dict.insert(name.into(), self.parse_object()?);
                }
                Token::DictClose => {
                    // Check for a stream
                    let token = self.lexer.peek_token()?;
                    match token {
                        Token::Keyword(kw) => {
                            match kw {
                                "stream" => {
                                    self.lexer.next_token()?; // discard "stream" token
                                    self.lexer.skip_optional_newline()?;
                                    return Ok(PdfObject::Stream(PdfStream::new(
                                        dict,
                                        self.lexer.pos,
                                    )));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    return Ok(PdfObject::Dictionary(dict));
                }
                _ => return Err(PdfError::from("Invalid token.")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use joaf_pdf_core::{PdfArray, PdfName};

    use super::*;

    fn parse_single_object(input: &[u8]) -> Result<PdfObject<'_>, PdfError> {
        let mut parser = PdfParser::new(input);
        parser.parse_object()
    }

    #[test]
    fn test_parse_bool() {
        let token = parse_single_object(b"true").unwrap();
        assert_eq!(token, PdfObject::Boolean(true));

        let token = parse_single_object(indoc!(
            b"
            % a true value
            true
            "
        ))
        .unwrap();
        assert_eq!(token, PdfObject::Boolean(true));

        let token = parse_single_object(b"false").unwrap();
        assert_eq!(token, PdfObject::Boolean(false));

        let token = parse_single_object(indoc!(
            b"
            % a false value
            false
            "
        ))
        .unwrap();
        assert_eq!(token, PdfObject::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let token = parse_single_object(b"null").unwrap();
        assert_eq!(token, PdfObject::Null);

        let token = parse_single_object(indoc!(
            b"
            % a null value
            null
            "
        ))
        .unwrap();
        assert_eq!(token, PdfObject::Null);
    }

    #[test]
    fn test_parse_reference() {
        let token = parse_single_object(b"1 0 R").unwrap();
        assert_eq!(
            token,
            PdfObject::Reference(ObjectId {
                id: 1,
                generation: 0
            })
        );

        let token = parse_single_object(indoc!(
            b"
            % a reference
            1 0 R
            "
        ))
        .unwrap();
        assert_eq!(
            token,
            PdfObject::Reference(ObjectId {
                id: 1,
                generation: 0
            })
        );
    }

    #[test]
    fn test_empty_array() {
        let token = parse_single_object(b"[]").unwrap();
        assert_eq!(token, PdfObject::Array(PdfArray::new()));

        let token = parse_single_object(indoc!(
            b"
            % a empty array
            []
            "
        ))
        .unwrap();
        assert_eq!(token, PdfObject::Array(PdfArray::new()));

        let token = parse_single_object(indoc!(
            b"
            % a empty array
            [
            % with a comment inside
            ]
            "
        ))
        .unwrap();
        assert_eq!(token, PdfObject::Array(PdfArray::new()));
    }

    #[test]
    fn test_simple_array() {
        let token = parse_single_object(indoc!(b"[1 2 3]")).unwrap();
        let expected = PdfObject::Array(PdfArray::from(vec![
            PdfObject::Integer(1),
            PdfObject::Integer(2),
            PdfObject::Integer(3),
        ]));
        assert_eq!(token, expected);
    }

    #[test]
    fn test_array_with_references() {
        let token = parse_single_object(indoc!(b"[1 0 R 2 0 R 3 0 R]")).unwrap();
        let expected = PdfObject::Array(PdfArray::from(vec![
            PdfObject::Reference(ObjectId {
                id: 1,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 2,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 3,
                generation: 0,
            }),
        ]));
        assert_eq!(token, expected);

        let token = parse_single_object(indoc!(
            b"
                % an array with references
                [
                    1 0 R % reference 1
                    2 0 R % reference 2
                    3 0 R % reference 3
                ]
            "
        ))
        .unwrap();
        let expected = PdfObject::Array(PdfArray::from(vec![
            PdfObject::Reference(ObjectId {
                id: 1,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 2,
                generation: 0,
            }),
            PdfObject::Reference(ObjectId {
                id: 3,
                generation: 0,
            }),
        ]));
        assert_eq!(token, expected);
    }

    #[test]
    fn test_empty_dictionary() {
        let token = parse_single_object(b"<<>>").unwrap();
        assert_eq!(token, PdfObject::Dictionary(PdfDictionary::new()));

        let token = parse_single_object(indoc!(
            b"
            % a empty dictionary
            <<>>
            "
        ))
        .unwrap();
        assert_eq!(token, PdfObject::Dictionary(PdfDictionary::new()));

        let token = parse_single_object(indoc!(
            b"
            % a empty dictionary
            <<
            % with a comment inside
            >>
            "
        ))
        .unwrap();
        assert_eq!(token, PdfObject::Dictionary(PdfDictionary::new()));
    }

    #[test]
    fn test_simple_dictionary() {
        let token = parse_single_object(indoc!(
            b"
            % a simple dictionary
            <<
                /Type  /Page
            >>
            "
        ))
        .unwrap();

        let mut expected = PdfDictionary::new();
        expected.insert(PdfName::TYPE, PdfObject::Name(PdfName::PAGE));

        assert_eq!(token, PdfObject::Dictionary(expected));

        let token = parse_single_object(indoc!(
            b"
            % a simple dictionary
            <<
                /Count  1
            >>
            "
        ))
        .unwrap();

        let mut expected = PdfDictionary::new();
        expected.insert(PdfName::COUNT, PdfObject::Integer(1));

        assert_eq!(token, PdfObject::Dictionary(expected));
    }

    #[test]
    fn test_parse_object() {
        let token = parse_single_object(indoc!(
            b"
            % a simple object
            1 0 obj
            <<
                /Type  /Page
            >>
            endobj
            "
        ))
        .unwrap();

        let mut expected_obj = PdfDictionary::new();
        expected_obj.insert(PdfName::TYPE, PdfObject::Name(PdfName::PAGE));

        let expected = PdfObject::IndirectObject {
            object_id: ObjectId {
                id: 1,
                generation: 0,
            },
            object: Box::new(PdfObject::Dictionary(expected_obj)),
        };

        assert_eq!(token, expected);
    }

    #[test]
    fn test_parse_empty_stream() {
        let token = parse_single_object(indoc!(
            b"
            % an empty stream
            1 0 obj
            <<
                /Length 0
            >>
            stream
            endstream
            endobj
            "
        ))
        .unwrap();

        let mut expected_dict = PdfDictionary::new();
        expected_dict.insert(PdfName::LENGTH, PdfObject::Integer(0));

        let expected_obj = PdfStream::new(expected_dict, 53);

        let expected = PdfObject::IndirectObject {
            object_id: ObjectId {
                id: 1,
                generation: 0,
            },
            object: Box::new(PdfObject::Stream(expected_obj)),
        };

        assert_eq!(token, expected);
    }

    #[test]
    fn test_parse_simple_stream() {
        let token = parse_single_object(indoc!(
            b"
            % a simple stream
            1 0 obj
            <<
                /Length 13
            >>
            stream
            simple stream
            endstream
            endobj
            "
        ))
        .unwrap();

        let mut expected_dict = PdfDictionary::new();
        expected_dict.insert(PdfName::LENGTH, PdfObject::Integer(13));

        let expected_obj = PdfStream::new(expected_dict, 54);

        let expected = PdfObject::IndirectObject {
            object_id: ObjectId {
                id: 1,
                generation: 0,
            },
            object: Box::new(PdfObject::Stream(expected_obj)),
        };

        assert_eq!(token, expected);
    }

    #[test]
    fn test_parse_stardard_example_stream() {
        let token = parse_single_object(indoc!(
            br##"
                1 0 obj
                    << /Length 534
                       /Filter [/ASCII85Decode /LZWDecode]
                    >>
                stream
                J..)6T`?p&<!J9%_[umg"B7/Z7KNXbN'S+,*Q/&"OLT'F
                LIDK#!n`$"<Atdi`\Vn%b%)&'cA*VnK\CJY(sF>c!Jnl@
                RM]WM;jjH6Gnc75idkL5]+cPZKEBPWdR>FF(kj1_R%W_d
                &/jS!;iuad7h?[L-F$+]]0A3Ck*$I0KZ?;<)CJtqi65Xb
                Vc3\n5ua:Q/=0$W<#N3U;H,MQKqfg1?:lUpR;6oN[C2E4
                ZNr8Udn.'p+?#X+1>0Kuk$bCDF/(3fL5]Oq)^kJZ!C2H1
                'TO]Rl?Q:&'<5&iP!$Rq;BXRecDN[IJB`,)o8XJOSJ9sD
                S]hQ;Rj@!ND)bD_q&C\g:inYC%)&u#:u,M6Bm%IY!Kb1+
                ":aAa'S`ViJglLb8<W9k6Yl\\0McJQkDeLWdPN?9A'jX*
                al>iG1p&i;eVoK&juJHs9%;Xomop"5KatWRT"JQ#qYuL,
                JD?M$0QP)lKn06l1apKDC@\qJ4B!!(5m+j.7F790m(Vj8
                8l8Q:_CZ(Gm1%X\N1&u!FKHMB~>
                endstream
                endobj
            "##
        ))
        .unwrap();

        let mut expected_dict = PdfDictionary::new();
        expected_dict.insert(PdfName::LENGTH, PdfObject::Integer(534));
        expected_dict.insert(
            PdfName::FILTER,
            PdfObject::Array(PdfArray::from(vec![
                PdfObject::Name(PdfName::ASCII85_DECODE),
                PdfObject::Name(PdfName::LZW_DECODE),
            ])),
        );

        let expected = PdfObject::IndirectObject {
            object_id: ObjectId {
                id: 1,
                generation: 0,
            },
            object: Box::new(PdfObject::Stream(PdfStream::new(expected_dict, 84))),
        };

        assert_eq!(token, expected);
    }

    #[test]
    fn test_parse_trailer() {
        const PDF_DOC: &[u8] = indoc!(
            br##"
            % ... contents of the PDF ...
            xref
            0 5
            0000000000 65535 f
            0000000010 00000 n
            0000000070 00000 n
            0000000134 00000 n
            0000000204 00000 n

            trailer
                <<
                    /Size 5
                    /Root 1 0 R
                >>
            startxref
            0
            %%EOF
            "##
        );

        let mut parser = PdfParser::new(PDF_DOC);
        let (xref_table, trailer) = parser.parse_trailer().unwrap();

        let expected_xref_table = XrefTable::from([
            (
                0,
                XrefEntry {
                    byte_offset: 0,
                    generation: 65535,
                    in_use: false,
                },
            ),
            (
                1,
                XrefEntry {
                    byte_offset: 10,
                    generation: 0,
                    in_use: true,
                },
            ),
            (
                2,
                XrefEntry {
                    byte_offset: 70,
                    generation: 0,
                    in_use: true,
                },
            ),
            (
                3,
                XrefEntry {
                    byte_offset: 134,
                    generation: 0,
                    in_use: true,
                },
            ),
            (
                4,
                XrefEntry {
                    byte_offset: 204,
                    generation: 0,
                    in_use: true,
                },
            ),
        ]);

        assert_eq!(xref_table, expected_xref_table);

        let mut expected_trailer = PdfDictionary::new();
        expected_trailer.insert(PdfName::SIZE, PdfObject::Integer(5));
        expected_trailer.insert(
            PdfName::ROOT,
            PdfObject::Reference(ObjectId {
                id: 1,
                generation: 0,
            }),
        );

        assert_eq!(trailer, expected_trailer);
    }
}
