#[derive(Debug)]
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
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl core::error::Error for Error {}

mod tests {
    use super::Error;

    #[test]
    fn error_to_string() {
        use super::ErrorKind::*;
        assert_eq!(Error::new(InvalidSyntax).to_string(), "InvalidSyntax: ");
    }
}
