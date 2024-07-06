#[derive(Debug, Clone)]
pub struct Token(String);

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str { self.0.as_str() }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Into<String> for Token {
    fn into(self) -> String { self.0 }
}
