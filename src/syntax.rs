use core::fmt::Display;

use crate::error::{Error, ErrorKind};
use crate::evaluation::Term;
use crate::parser::Token;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(String);

impl Symbol {
    /// Warning: The constructor does not include validation.
    /// For parsing, use `Symbol::try_form` instead.
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self(value.into())
    }

    pub fn validate_token(token: &Token) -> bool {
        for ch in token.as_ref().chars() {
            if "()[]{}\x0b".contains(ch) || ch.is_ascii_whitespace() { return false; }
        }
        true
    }
}

impl AsRef<String> for Symbol {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self { Self(value.to_string()) }
}

impl From<String> for Symbol {
    fn from(value: String) -> Self { Self(value) }
}

impl TryFrom<Token> for Symbol {
    type Error = Error;

    /// Warning: The `try_from` is actually used to convert from
    /// a symbol token safely.
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        if Symbol::validate_token(&token) { Ok(Self(token.into())) }
        else {
            // TODO: Adjust into builder form.
            Err(Error::from((
                ErrorKind::InvalidSyntax,
                format!("Unsupported literal appeared in symbol '{}'.", token.as_ref())
            )))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    List(Vec<Node>),
    Symbol(Symbol)
}

impl Node {
    pub fn push(&mut self, node: Node) -> &mut Node{
        self.as_mut().push(node);
        match self {
            Node::List(list) => list.last_mut().unwrap(),
            _ => panic!()
        }
    }

}

impl AsMut<Vec<Node>> for Node {
    fn as_mut(&mut self) -> &mut Vec<Node> {
        match self {
            Node::List(list) => list,
            _ => panic!()
        }
    }
}


impl AsRef<Vec<Node>> for Node {
    fn as_ref(&self) -> &Vec<Node> {
        match self {
            Self::List(vec) => vec,
            _ => panic!()
        }
    }
}

impl Display for Node {
    // TODO: Ensure the safety of nested call to print lists of arbitrary depth.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::List(nodes) => {
                if nodes.is_empty() { return write!(f, "()"); }

                if nodes.len() == 1 { return write!(f, "({})", nodes[0]) }

                write!(f, "({} ", nodes[0])?;
                for node in nodes[1..nodes.len() - 1].iter() {
                    write!(f, "{} ", node)?
                }
                write!(f, "{})", nodes.last().unwrap())
            }
            Node::Symbol(symbol) => write!(f, "{}", symbol)
        }
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Self::Symbol(value.into())
    }
}

impl Into<Term> for Node {
    fn into(self) -> Term {
        match self {
            Node::List(mut list) => {
                let mut term = Term::new();
                term.sub_terms = {
                    let taken_vec = std::mem::take(&mut list);
                    taken_vec.into_iter().map(|node| node.into()).collect()
                };
                term
            },
            Node::Symbol(symbol) => {
                Term::from(symbol)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Token;
    use super::{Node, Symbol};

    #[test]
    fn node_to_string() {
        use Node::*;
        assert_eq!(List(vec![Symbol("apply".into()), Symbol("+".into())]).to_string(), "(apply +)");
    }

    #[test]
    fn symbol_from_str(){
        assert_eq!(Symbol::from("symbol"), Symbol("symbol".to_string()));
    }

    #[test]
    fn symbol_from_string() {
        assert_eq!(Symbol::from("symbol"), Symbol("symbol".to_string()));

        let literal = "test-move".to_string();
        assert_eq!(Symbol::from(literal), Symbol::from("test-move"));
    }

    #[test]
    fn symbol_to_string() {
        assert_eq!(Symbol::new("symbol").to_string(), "symbol");
    }

    #[test]
    fn symbol_try_from_token() {
        assert!(Symbol::try_from(Token::from("valid-token")).is_ok());
        assert!(Symbol::try_from(Token::from("valid_token")).is_ok());
        assert!(Symbol::try_from(Token::from("(invalid-token)")).is_err());
        assert!(Symbol::try_from(Token::from("[invalid_token]")).is_err());
        assert!(Symbol::try_from(Token::from("{invalid token}")).is_err());
    }
}
