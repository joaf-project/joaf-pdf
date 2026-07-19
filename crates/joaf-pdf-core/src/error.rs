use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct PdfError {
    pub message: String,
}

impl PdfError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn from_io_error(error: std::io::Error) -> Self {
        Self {
            message: format!("IO Error: {}", error.to_string()),
        }
    }

    pub fn from_utf8_error(error: std::str::Utf8Error) -> Self {
        Self {
            message: format!("Utf8 Error: {}", error.to_string()),
        }
    }

    pub fn from_from_utf8_error(error: std::string::FromUtf8Error) -> Self {
        Self {
            message: format!("Utf8 Error: {}", error.to_string()),
        }
    }

    pub fn unexpected_eof(pos: usize) -> Self {
        Self {
            message: format!("Unexpected EOF at position {}", pos),
        }
    }

    pub fn invalid_type(expected: &str) -> Self {
        Self {
            message: format!("Invalid Type: {}", expected),
        }
    }

    pub fn invalid_reference(key: &str) -> Self {
        Self {
            message: format!("Invalid Reference: {}", key),
        }
    }

    pub fn missing_required_key(key: &str) -> Self {
        Self {
            message: format!("Missing Required Key: {}", key),
        }
    }

    pub fn parser(message: &str, position: usize) -> Self {
        Self {
            message: format!("Parser error at position {}: {}", position, message),
        }
    }

    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        Self {
            message: format!("Type mismatch: expected {}, got {}", expected, actual),
        }
    }
}

impl Error for PdfError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PDF error: {}", self.message)
    }
}
