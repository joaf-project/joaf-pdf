use joaf_pdf_core::PdfError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    EOF,

    Keyword(String),
    Integer(i64),
    Real(f64),
    Name(String),
    LiteralString(Vec<u8>),
    HexString(Vec<u8>),
    BracketOpen,  // [
    BracketClose, // ]
    DictOpen,     // <<
    DictClose,    // >>
}

impl Token {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Token::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_real(&self) -> Option<f64> {
        match self {
            Token::Real(r) => Some(*r),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Token::Name(s) => Some(s),
            _ => None,
        }
    }
}

pub struct Lexer<'a> {
    pub input: &'a [u8],
    pub pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Lexer { input, pos: 0 }
    }

    pub fn peek_token(&mut self) -> Result<Token, PdfError> {
        let pos = self.pos;
        let token = self.next_token()?;
        self.pos = pos;
        Ok(token)
    }

    pub fn next_token(&mut self) -> Result<Token, PdfError> {
        self.skip_whitespace();
        self.require_token()
    }

    pub fn require_token(&mut self) -> Result<Token, PdfError> {
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

    pub fn consume_bytes(&mut self, amount: usize) -> Result<Vec<u8>, PdfError> {
        if self.pos + amount > self.input.len() {
            return Err(PdfError::unexpected_eof(self.pos));
        }

        let bytes = self.input[self.pos..self.pos + amount].to_vec();
        self.pos += amount;
        Ok(bytes)
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

    fn read_keyword(&mut self) -> Result<Token, PdfError> {
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
        Ok(Token::Keyword(s.to_string()))
    }

    fn read_number(&mut self) -> Result<Token, PdfError> {
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

    fn read_name(&mut self) -> Result<Token, PdfError> {
        self.pos += 1; // Skip '/'
        let mut name = String::new();
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if is_delimiter(b) || is_whitespace(b) {
                break;
            }
            if b == b'#' && self.pos + 2 < self.input.len() {
                let hex_str = std::str::from_utf8(&self.input[self.pos + 1..self.pos + 3])
                    .map_err(|_| {
                        PdfError::parser("Invalid escaped character sequence", self.pos)
                    })?;
                let val = u8::from_str_radix(hex_str, 16).map_err(|_| {
                    PdfError::parser("Invalid escaped character sequence", self.pos)
                })?;
                name.push(val as char);
                self.pos += 3;
            } else {
                name.push(b as char);
                self.pos += 1;
            }
        }

        Ok(Token::Name(name))
    }

    fn read_literal_string(&mut self) -> Result<Token, PdfError> {
        self.pos += 1; // Skip '('

        let mut buf = Vec::new();
        let mut paren_count = 1;

        while self.pos < self.input.len() {
            let b = self.input[self.pos];

            match b {
                b'\\' => {
                    if self.pos >= self.input.len() {
                        return Err(PdfError::parser("Unexpected end of input", self.pos));
                    }

                    self.pos += 1; // skip '\'

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
                            // line continuation
                            self.pos += 1;
                        }
                        b'0'..=b'7' => {
                            // octal
                            let start = self.pos;
                            let mut count = 0;
                            loop {
                                count += 1;
                                self.pos += 1;

                                if count >= 3 {
                                    break;
                                }

                                if self.pos >= self.input.len() {
                                    return Err(PdfError::parser(
                                        "Unexpected end of input",
                                        self.pos,
                                    ));
                                }

                                match self.input[self.pos] {
                                    b'0'..=b'7' => continue,
                                    _ => break,
                                }
                            }

                            let val = u8::from_str_radix(
                                std::str::from_utf8(&self.input[start..self.pos])
                                    .map_err(|_| PdfError::parser("Invalid octal number", start))?,
                                8,
                            )
                            .map_err(|_| PdfError::parser("Invalid octal number", start))?;
                            buf.push(val);
                        }
                        _ => return Err(PdfError::parser("Invalid escaped character", self.pos)),
                    }
                }
                b'(' => {
                    paren_count += 1;

                    buf.push(b);
                    self.pos += 1;
                }
                b')' => {
                    paren_count -= 1;

                    if paren_count == 0 {
                        self.pos += 1; // Skip ')'
                        return Ok(Token::LiteralString(buf));
                    }

                    buf.push(b);
                    self.pos += 1;
                }
                _ => {
                    buf.push(b);
                    self.pos += 1;
                }
            }
        }

        Err(PdfError::parser("Unexpected end of input", self.pos))
    }

    fn read_hex_string(&mut self) -> Result<Token, PdfError> {
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

    fn lex_single_token(input: &[u8]) -> Result<Token, PdfError> {
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
            Token::Keyword("obj".to_string()),
            Token::DictOpen,
            Token::Name("Name".to_string()),
            Token::Name("Value".to_string()),
            Token::Name("Name".to_string()),
            Token::Name("Value".to_string()),
            Token::Name("Integer".to_string()),
            Token::Integer(0),
            Token::Name("Integer".to_string()),
            Token::Integer(-123),
            Token::Name("Real".to_string()),
            Token::Real(0.0),
            Token::Name("Real".to_string()),
            Token::Real(0.123),
            Token::Name("Array".to_string()),
            Token::BracketOpen,
            Token::Name("Name".to_string()),
            Token::Name("Value".to_string()),
            Token::Integer(0),
            Token::Integer(123),
            Token::Real(0.0),
            Token::Real(0.123),
            Token::BracketClose,
            Token::Name("LiteralString".to_string()),
            Token::LiteralString(b"Hello World".to_vec()),
            Token::Name("HexString".to_string()),
            Token::HexString(b"Hello, World!".to_vec()),
            Token::Name("Reference".to_string()),
            Token::Integer(1),
            Token::Integer(0),
            Token::Keyword("R".to_string()),
            Token::Name("Boolean".to_string()),
            Token::Keyword("true".to_string()),
            Token::Name("Boolean".to_string()),
            Token::Keyword("false".to_string()),
            Token::Name("Null".to_string()),
            Token::Keyword("null".to_string()),
            Token::DictClose,
            Token::Keyword("endobj".to_string()),
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
        assert_eq!(token, Token::Name("".to_string()));

        let token = lex_single_token(b"/Name").unwrap();
        assert_eq!(token, Token::Name("Name".to_string()));

        let token = lex_single_token(b"   /Name   ").unwrap();
        assert_eq!(token, Token::Name("Name".to_string()));

        let token = lex_single_token(indoc!(
            b"
                % comments
                /Name
            "
        ))
        .unwrap();
        assert_eq!(token, Token::Name("Name".to_string()));
    }

    #[test]
    fn test_read_name_with_escapes() {
        let token = lex_single_token(b"/Name#20Name").unwrap();
        assert_eq!(token, Token::Name("Name Name".to_string()));

        let token = lex_single_token(b"/Name#23Name").unwrap();
        assert_eq!(token, Token::Name("Name#Name".to_string()));

        let token = lex_single_token(b"/Name#2eName").unwrap();
        assert_eq!(token, Token::Name("Name.Name".to_string()));

        let token = lex_single_token(b"/Name#28Name#29").unwrap();
        assert_eq!(token, Token::Name("Name(Name)".to_string()));
    }

    #[test]
    fn test_read_name_invalid_escape() {
        match lex_single_token(b"/Name#2gName") {
            Ok(t) => panic!("Expected error, got: {:?}", t),
            Err(e) => assert_eq!(e, PdfError::parser("Invalid escaped character sequence", 5)),
        }
    }

    #[test]
    fn test_read_keyword() {
        let token = lex_single_token(b"xref").unwrap();
        assert_eq!(token, Token::Keyword("xref".to_string()));

        let token = lex_single_token(b" R").unwrap();
        assert_eq!(token, Token::Keyword("R".to_string()));
    }

    #[test]
    fn test_read_literal_string() {
        let token = lex_single_token(b"()").unwrap();
        assert_eq!(token, Token::LiteralString(b"".to_vec()));

        let token = lex_single_token(b"(Hello, world!)").unwrap();
        assert_eq!(token, Token::LiteralString(b"Hello, world!".to_vec()));

        let token = lex_single_token(b"(% Hello World %)").unwrap();
        assert_eq!(token, Token::LiteralString(b"% Hello World %".to_vec()));

        let token = lex_single_token(br"(\n\r\t\b\f\(\)\040\\)").unwrap();
        assert_eq!(token, Token::LiteralString(b"\n\r\t\x08\x0c() \\".to_vec()));

        let token = lex_single_token(b"(Hello (New) World)").unwrap();
        assert_eq!(token, Token::LiteralString(b"Hello (New) World".to_vec()));

        let token = lex_single_token(br"(Hello \(New\) World)").unwrap();
        assert_eq!(token, Token::LiteralString(b"Hello (New) World".to_vec()));

        let token = lex_single_token(indoc!(
            b"
            (Hello \
            World)
            "
        ))
        .unwrap();
        assert_eq!(token, Token::LiteralString(b"Hello World".to_vec()));
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
