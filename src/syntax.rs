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

#[derive(Debug, Clone)]
pub enum Node {
    List(Vec<Node>),
    Symbol(Symbol)
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::List(nodes) => {
                if nodes.is_empty() { return write!(f, "()"); }

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

mod tests {
    use super::{Node, Symbol};

    #[test]
    fn node_to_string() {
        use Node::*;
        assert_eq!(List(vec![Symbol("apply".into()), Symbol("+".into())]).to_string(), "(apply +)");
    }

    #[test]
    fn symbol_to_string() {
        assert_eq!(Symbol::from("").to_string(), "");
    }
}
