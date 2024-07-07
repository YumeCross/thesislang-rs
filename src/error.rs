#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidSyntax
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, message: "".to_string() }
    }

    pub fn kind(&self) -> ErrorKind { self.kind }

    pub fn message(&self) -> &String { &self.message }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl core::error::Error for Error {}

impl<S: Into<String>> From<(ErrorKind, S)> for Error {
    fn from(value: (ErrorKind, S)) -> Self {
        Self { kind: value.0, message: value.1.into() }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::Error;

    #[test]
    fn error_to_string() {
        use super::ErrorKind::*;
        assert_eq!(Error::new(InvalidSyntax).to_string(), "InvalidSyntax: ");
    }
}
