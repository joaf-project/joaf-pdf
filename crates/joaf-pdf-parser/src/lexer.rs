use std::borrow::Cow;

use joaf_pdf_core::PdfError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    EOF,

    Keyword(&'a str),
    Integer(i64),
    Real(f64),
    Name(Cow<'a, str>),
    LiteralString(Cow<'a, [u8]>),
    HexString(Vec<u8>),
    BracketOpen,  // [
    BracketClose, // ]
    DictOpen,     // <<
    DictClose,    // >>
}

impl<'a> Token<'a> {}

pub struct Lexer<'a> {
    pub input: &'a [u8],
    pub pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Lexer { input, pos: 0 }
    }

    pub fn peek_token(&mut self) -> Result<Token<'a>, PdfError> {
        let pos = self.pos;
        let token = self.next_token()?;
        self.pos = pos;
        Ok(token)
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, PdfError> {
        self.skip_whitespace();
        self.require_token()
    }

    pub fn require_token(&mut self) -> Result<Token<'a>, PdfError> {
        if self.pos >= self.input.len() {
            return Ok(Token::EOF);
        }

        let b = self.input[self.pos];
        match b {
            b'0'..=b'9' | b'+' | b'-' | b'.' => Ok(self.read_number()?),
            b'/' => Ok(self.read_name()?),
            b'(' => Ok(self.read_literal_string()?),
            b'[' => {
                self.pos += 1;
                Ok(Token::BracketOpen)
            }
            b']' => {
                self.pos += 1;
                Ok(Token::BracketClose)
            }
            b'<' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == b'<' {
                    self.pos += 2;
                    Ok(Token::DictOpen)
                } else {
                    Ok(self.read_hex_string()?)
                }
            }
            b'>' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == b'>' {
                    self.pos += 2;
                    Ok(Token::DictClose)
                } else {
                    Err(PdfError::parser("Unexpected character: >", self.pos))
                }
            }
            _ => Ok(self.read_keyword()?),
        }
    }

    pub fn skip_optional_newline(&mut self) -> Result<(), PdfError> {
        if self.pos + 2 > self.input.len() {
            return Err(PdfError::unexpected_eof(self.pos));
        }

        if self.input[self.pos] == b'\n' {
            self.pos += 1;
            return Ok(());
        }

        if self.input[self.pos] == b'\r' && self.input[self.pos + 1] == b'\n' {
            self.pos += 2;
            return Ok(());
        }

        return Ok(());
    }

    pub fn consume_bytes(&mut self, amount: usize) -> Result<&'a [u8], PdfError> {
        if self.pos + amount > self.input.len() {
            return Err(PdfError::unexpected_eof(self.pos));
        }

        let start = self.pos;
        self.pos += amount;
        Ok(&self.input[start..self.pos])
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if is_whitespace(b) {
                self.pos += 1;
            } else if b == b'%' {
                // For PDF, comments are just whitespace
                self.pos += 1;
                while self.pos < self.input.len() {
                    let cb = self.input[self.pos];
                    self.pos += 1;
                    if cb == b'\n' || cb == b'\r' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn read_keyword(&mut self) -> Result<Token<'a>, PdfError> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if is_delimiter(b) || is_whitespace(b) {
                break;
            }
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| PdfError::parser("Invalid keyword", start))?;
        Ok(Token::Keyword(s))
    }

    fn read_number(&mut self) -> Result<Token<'a>, PdfError> {
        let start = self.pos;

        let mut is_real = false;
        let mut has_numbers = false;

        let b = self.input[self.pos];
        if b == b'+' || b == b'-' {
            self.pos += 1;
        }

        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            match b {
                b'0'..=b'9' => {
                    has_numbers = true;
                    self.pos += 1;
                }
                b'.' => {
                    if is_real {
                        break;
                    }
                    has_numbers = true;
                    is_real = true;
                    self.pos += 1;
                }
                b'+' | b'-' => {
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        let s = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| PdfError::parser("Invalid number", start))?;

        if !has_numbers {
            return Err(PdfError::parser("Invalid number", start));
        }

        if is_real {
            Ok(Token::Real(
                s.parse()
                    .map_err(|_| PdfError::parser("Invalid number", start))?,
            ))
        } else {
            Ok(Token::Integer(
                s.parse()
                    .map_err(|_| PdfError::parser("Invalid number", start))?,
            ))
        }
    }

    fn read_name(&mut self) -> Result<Token<'a>, PdfError> {
        self.pos += 1; // Skip '/'
        let start = self.pos;
        let mut has_escapes = false;

        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if is_delimiter(b) || is_whitespace(b) {
                break;
            }
            if b == b'#' {
                has_escapes = true;
            }
            self.pos += 1;
        }

        let end = self.pos;

        let raw_bytes = &self.input[start..end];
        let raw_str = std::str::from_utf8(raw_bytes).map_err(PdfError::from)?;

        if !has_escapes {
            Ok(Token::Name(Cow::Borrowed(raw_str)))
        } else {
            let mut name = String::with_capacity(raw_bytes.len());
            let mut i = start;

            while i < end {
                let b = self.input[i];
                if b == b'#' && i + 2 < end {
                    let hex_str =
                        std::str::from_utf8(&self.input[i + 1..i + 3]).map_err(PdfError::from)?;
                    let val = u8::from_str_radix(hex_str, 16)
                        .map_err(|_| PdfError::parser("Invalid escaped sequence", i))?;
                    name.push(val as char);
                    i += 3;
                } else {
                    name.push(b as char);
                    i += 1;
                }
            }
            Ok(Token::Name(Cow::Owned(name)))
        }
    }

    fn read_literal_string(&mut self) -> Result<Token<'a>, PdfError> {
        self.pos += 1; // Skip '('
        let start = self.pos;

        let mut paren_count = 1;
        let mut has_escapes = false;

        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            match b {
                b'\\' => {
                    has_escapes = true;
                    self.pos += 2;
                }
                b'(' => {
                    paren_count += 1;
                    self.pos += 1;
                }
                b')' => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        let end = self.pos;
                        self.pos += 1; // Skip the final ')'

                        if !has_escapes {
                            return Ok(Token::LiteralString(Cow::Borrowed(
                                &self.input[start..end],
                            )));
                        } else {
                            break;
                        }
                    }
                    self.pos += 1;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        if paren_count != 0 {
            return Err(PdfError::parser("Unexpected end of input", self.pos));
        }

        let end_pos = self.pos - 1; // The final ')' position
        self.pos = start;

        let mut buf = Vec::with_capacity(end_pos - start);

        while self.pos < end_pos {
            let b = self.input[self.pos];
            match b {
                b'\\' => {
                    self.pos += 1; // skip '\'
                    if self.pos >= end_pos {
                        return Err(PdfError::parser("Unexpected end of input", self.pos));
                    }

                    let b2 = self.input[self.pos];
                    match b2 {
                        b'n' => {
                            buf.push(b'\n');
                            self.pos += 1;
                        }
                        b'r' => {
                            buf.push(b'\r');
                            self.pos += 1;
                        }
                        b't' => {
                            buf.push(b'\t');
                            self.pos += 1;
                        }
                        b'b' => {
                            buf.push(0x08);
                            self.pos += 1;
                        }
                        b'f' => {
                            buf.push(0x0C);
                            self.pos += 1;
                        }
                        b'(' => {
                            buf.push(b'(');
                            self.pos += 1;
                        }
                        b')' => {
                            buf.push(b')');
                            self.pos += 1;
                        }
                        b'\\' => {
                            buf.push(b'\\');
                            self.pos += 1;
                        }
                        b'\n' | b'\r' => {
                            // Line continuation: ignore the newline completely
                            self.pos += 1;
                            let b2 = self.input[self.pos];
                            if b2 == b'\n' || b2 == b'\r' {
                                self.pos += 1;
                            }
                        }
                        b'0'..=b'7' => {
                            // Parse Octal up to 3 digits
                            let octal_start = self.pos;
                            let mut count = 0;
                            while count < 3 && self.pos < end_pos {
                                if matches!(self.input[self.pos], b'0'..=b'7') {
                                    count += 1;
                                    self.pos += 1;
                                } else {
                                    break;
                                }
                            }

                            let octal_str = std::str::from_utf8(&self.input[octal_start..self.pos])
                                .map_err(|_| {
                                    PdfError::parser("Invalid octal number", octal_start)
                                })?;
                            let val = u8::from_str_radix(octal_str, 8).map_err(|_| {
                                PdfError::parser("Invalid octal number", octal_start)
                            })?;
                            buf.push(val);
                        }
                        _ => return Err(PdfError::parser("Invalid escaped character", self.pos)),
                    }
                }
                _ => {
                    buf.push(b);
                    self.pos += 1;
                }
            }
        }

        self.pos = end_pos + 1; // Ensure the cursor finishes right after the closing ')'
        Ok(Token::LiteralString(Cow::Owned(buf)))
    }

    fn read_hex_string(&mut self) -> Result<Token<'a>, PdfError> {
        self.pos += 1; // Skip '<'

        let mut hex_chars = Vec::new();

        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            match b {
                b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                    hex_chars.push(b);
                    self.pos += 1;
                }
                b'>' => {
                    self.pos += 1; // Skip '>'

                    if hex_chars.len() % 2 != 0 {
                        hex_chars.push(b'0');
                    }

                    let bytes = hex::decode(hex_chars)
                        .map_err(|_| PdfError::parser("Invalid hex string", self.pos))?;

                    return Ok(Token::HexString(bytes));
                }
                _ => {
                    if is_whitespace(b) {
                        self.pos += 1;
                        continue;
                    }

                    return Err(PdfError::parser(
                        "Invalid character in hex string",
                        self.pos,
                    ));
                }
            }
        }

        Err(PdfError::parser("Unexpected end of input", self.pos))
    }
}

#[inline]
fn is_whitespace(b: u8) -> bool {
    // whitespace in PDF:
    // 0x00 - null
    // 0x09 - tab
    // 0x0A - newline
    // 0x0C - form feed
    // 0x0D - carriage return
    // 0x20 - space
    matches!(b, b'\0' | b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
}

#[inline]
fn is_delimiter(b: u8) -> bool {
    // delimiter in PDF:
    // ( )   - strings
    // < >   - hex strings
    // << >> - dictionaries
    // [ ]   - arrays
    // { }   - dictionaries
    // /     - names
    // %     - comments
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    fn lex_single_token(input: &[u8]) -> Result<Token<'_>, PdfError> {
        let mut lexer = Lexer::new(input);
        lexer.next_token()
    }

    #[test]
    fn test_lexer() {
        let input = indoc!(
            b"
            1 0 obj
            <<
            % This is a comment
            /Name/Value
            % Another comment
            /Name          /Value  % One more comment
            /Integer       0
            /Integer       -123
            /Real          0.
            /Real          .123
            /Array         [ /Name /Value 0 123 0. .123 ]
            /LiteralString (Hello World)
            /HexString     <48656c6c6f2c20576f726c6421>
            /Reference     1 0 R
            /Boolean       true
            /Boolean       false
            /Null          null
            >>
            endobj
            "
        );

        let tokens = [
            Token::Integer(1),
            Token::Integer(0),
            Token::Keyword("obj"),
            Token::DictOpen,
            Token::Name(Cow::Borrowed("Name")),
            Token::Name(Cow::Borrowed("Value")),
            Token::Name(Cow::Borrowed("Name")),
            Token::Name(Cow::Borrowed("Value")),
            Token::Name(Cow::Borrowed("Integer")),
            Token::Integer(0),
            Token::Name(Cow::Borrowed("Integer")),
            Token::Integer(-123),
            Token::Name(Cow::Borrowed("Real")),
            Token::Real(0.),
            Token::Name(Cow::Borrowed("Real")),
            Token::Real(0.123),
            Token::Name(Cow::Borrowed("Array")),
            Token::BracketOpen,
            Token::Name(Cow::Borrowed("Name")),
            Token::Name(Cow::Borrowed("Value")),
            Token::Integer(0),
            Token::Integer(123),
            Token::Real(0.),
            Token::Real(0.123),
            Token::BracketClose,
            Token::Name(Cow::Borrowed("LiteralString")),
            Token::LiteralString(Cow::Borrowed(b"Hello World")),
            Token::Name(Cow::Borrowed("HexString")),
            Token::HexString(b"Hello, World!".to_vec()),
            Token::Name(Cow::Borrowed("Reference")),
            Token::Integer(1),
            Token::Integer(0),
            Token::Keyword("R"),
            Token::Name(Cow::Borrowed("Boolean")),
            Token::Keyword("true"),
            Token::Name(Cow::Borrowed("Boolean")),
            Token::Keyword("false"),
            Token::Name(Cow::Borrowed("Null")),
            Token::Keyword("null"),
            Token::DictClose,
            Token::Keyword("endobj"),
        ];

        let mut lexer = Lexer::new(input);
        let mut actual_tokens = Vec::new();
        loop {
            let token = lexer.next_token().unwrap();
            if token == Token::EOF {
                break;
            }
            actual_tokens.push(token);
        }

        assert_eq!(actual_tokens, tokens);
    }

    #[test]
    fn test_read_integer_numbers() {
        let token = lex_single_token(b"0").unwrap();
        assert_eq!(token, Token::Integer(0));

        let token = lex_single_token(b"123").unwrap();
        assert_eq!(token, Token::Integer(123));

        let token = lex_single_token(b"+123").unwrap();
        assert_eq!(token, Token::Integer(123));

        let token = lex_single_token(b"-123").unwrap();
        assert_eq!(token, Token::Integer(-123));

        let token = lex_single_token(b"   123   ").unwrap();
        assert_eq!(token, Token::Integer(123));

        let token = lex_single_token(indoc!(
            b"
                % comments
                123%coment
            "
        ))
        .unwrap();
        assert_eq!(token, Token::Integer(123));
    }

    #[test]
    fn test_read_real_numbers() {
        let token = lex_single_token(b"123.").unwrap();
        assert_eq!(token, Token::Real(123.0));

        let token = lex_single_token(b"+123.").unwrap();
        assert_eq!(token, Token::Real(123.0));

        let token = lex_single_token(b"-123.").unwrap();
        assert_eq!(token, Token::Real(-123.0));

        let token = lex_single_token(b".456").unwrap();
        assert_eq!(token, Token::Real(0.456));

        let token = lex_single_token(b"+.456").unwrap();
        assert_eq!(token, Token::Real(0.456));

        let token = lex_single_token(b"-.456").unwrap();
        assert_eq!(token, Token::Real(-0.456));

        let token = lex_single_token(b"123.456").unwrap();
        assert_eq!(token, Token::Real(123.456));

        let token = lex_single_token(b"+123.456").unwrap();
        assert_eq!(token, Token::Real(123.456));

        let token = lex_single_token(b"-123.456").unwrap();
        assert_eq!(token, Token::Real(-123.456));

        let token = lex_single_token(b"   123.456   ").unwrap();
        assert_eq!(token, Token::Real(123.456));

        let token = lex_single_token(indoc!(
            b"
                % comments
                123.456
            "
        ))
        .unwrap();
        assert_eq!(token, Token::Real(123.456));
    }

    #[test]
    fn test_read_invalid_numbers() {
        match lex_single_token(b"+") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid number", 0)),
        }

        match lex_single_token(b"-") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid number", 0)),
        }

        match lex_single_token(b".") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid number", 0)),
        }

        match lex_single_token(b"+.") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid number", 0)),
        }

        match lex_single_token(b"+.") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid number", 0)),
        }
    }

    #[test]
    fn test_read_name() {
        let token = lex_single_token(b"/").unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("")));

        let token = lex_single_token(b"/Name").unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("Name")));

        let token = lex_single_token(b"   /Name   ").unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("Name")));

        let token = lex_single_token(indoc!(
            b"
                % comments
                /Name
            "
        ))
        .unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("Name")));
    }

    #[test]
    fn test_read_name_with_escapes() {
        let token = lex_single_token(b"/Name#20Name").unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("Name Name")));

        let token = lex_single_token(b"/Name#23Name").unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("Name#Name")));

        let token = lex_single_token(b"/Name#2eName").unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("Name.Name")));

        let token = lex_single_token(b"/Name#28Name#29").unwrap();
        assert_eq!(token, Token::Name(Cow::Borrowed("Name(Name)")));
    }

    #[test]
    fn test_read_name_invalid_escape() {
        match lex_single_token(b"/Name#2gName") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid escaped sequence", 5)),
        }
    }

    #[test]
    fn test_read_keyword() {
        let token = lex_single_token(b"xref").unwrap();
        assert_eq!(token, Token::Keyword("xref"));

        let token = lex_single_token(b" R").unwrap();
        assert_eq!(token, Token::Keyword("R"));
    }

    #[test]
    fn test_read_literal_string() {
        let token = lex_single_token(b"()").unwrap();
        assert_eq!(token, Token::LiteralString(Cow::Borrowed(b"")));

        let token = lex_single_token(b"(Hello, world!)").unwrap();
        assert_eq!(token, Token::LiteralString(Cow::Borrowed(b"Hello, world!")));

        let token = lex_single_token(b"(% Hello World %)").unwrap();
        assert_eq!(
            token,
            Token::LiteralString(Cow::Borrowed(b"% Hello World %"))
        );

        let token = lex_single_token(br"(\n\r\t\b\f\(\)\040\\)").unwrap();
        assert_eq!(
            token,
            Token::LiteralString(Cow::Borrowed(b"\n\r\t\x08\x0c() \\"))
        );

        let token = lex_single_token(b"(Hello (New) World)").unwrap();
        assert_eq!(
            token,
            Token::LiteralString(Cow::Borrowed(b"Hello (New) World"))
        );

        let token = lex_single_token(br"(Hello \(New\) World)").unwrap();
        assert_eq!(
            token,
            Token::LiteralString(Cow::Borrowed(b"Hello (New) World"))
        );

        let token = lex_single_token(indoc!(
            b"
            (Hello \
            World)
            "
        ))
        .unwrap();

        assert_eq!(token, Token::LiteralString(Cow::Borrowed(b"Hello World")));
    }

    #[test]
    fn test_read_hex_string() {
        let token = lex_single_token(b"<>").unwrap();
        assert_eq!(token, Token::HexString(b"".to_vec()));

        let token = lex_single_token(b"<48656c6c6f2c20576f726c6421>").unwrap();
        assert_eq!(token, Token::HexString(b"Hello, World!".to_vec()));

        let token = lex_single_token(b"<48656c6c6f2c2>").unwrap();
        assert_eq!(token, Token::HexString(b"Hello, ".to_vec()));

        let token = lex_single_token(indoc!(
            b"
                <
                    48 65 6c 6c
                    6f 2c 20 57
                    6f 72 6c 64
                    21
                >   "
        ))
        .unwrap();
        assert_eq!(token, Token::HexString(b"Hello, World!".to_vec()));
    }

    #[test]
    fn test_read_hex_string_invalid_escape() {
        match lex_single_token(b"<48656c6c6f2c20576f726c6421g>") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid character in hex string", 27)),
        }
    }
}
