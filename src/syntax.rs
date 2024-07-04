use core::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol(String);

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self { Self(value.to_string()) }
}

impl Into<String> for Symbol {
    fn into(self) -> String { self.0 }
}

mod tests {
    #[test]
    fn symbol_to_string() {
        assert_eq!(Symbol::from("").to_string(), "");
    }
}
