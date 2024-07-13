use crate::error::Error;
use crate::seq;
use crate::syntax::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token(String);

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str { self.0.as_str() }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self { Self(value) }
}

impl Into<String> for Token {
    fn into(self) -> String { self.0 }
}

/// Representing the current position as (line, column) of a source parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePos(usize, usize);

impl SourcePos {
    pub fn ln(&self) -> usize { self.0 }

    pub fn col(&self) -> usize { self.1 }

    pub fn next_ln(&mut self) { seq!(self.0 += 1, self.1 += 1) }

    pub fn next_col(&mut self) { self.1 += 1 }
}

impl From<(usize, usize)> for SourcePos {
    fn from(value: (usize, usize)) -> Self { Self(value.0, value.1) }
}

#[derive(Debug)]
pub struct LexicalParser {
    buf: String,
    pos: SourcePos,
    results: Vec<(SourcePos, Token)>
}

impl LexicalParser {
    pub fn new() -> Self {
        Self { buf: "".to_string(), pos: (1, 1).into(), results: vec![] }
    }

    pub fn results(&self) -> &Vec<(SourcePos, Token)> {
        &self.results
    }

    pub fn tokens(&self) -> Vec<Token> {
        self.results.iter().map(|pair| pair.1.clone()).collect()
    }

    pub fn parse_c(&mut self, ch: char) {
        match ch {
            '(' | '[' | '{' => {
                self.push_token("(".into());
            }
            ')' | ']' | '}'=> {
                self.try_collect_buf();
                self.push_token(")".into())
            }
            ',' | ';' => todo!("Add delimiter parsing."),
            '\'' | '"' => todo!("Add string literal parsing."),
            ch => {
                if ch.is_ascii_whitespace() || ch == '\x0B' {
                    self.try_collect_buf()
                } else {
                    self.buf.push(ch)
                }
            }
        }

        if ch != '\n' { self.pos.next_col() } else { self.pos.next_ln() }
    }

    pub fn parse_str(&mut self, source: &str) {
        for ch in source.chars() { self.parse_c(ch) }
        self.try_collect_buf();
    }

    #[inline]
    fn push_token(&mut self, token: Token) {
        self.results.push((self.pos, token))
    }

    #[inline]
    fn push_buf_as_token(&mut self) {
        self.results.push((self.pos, core::mem::take(&mut self.buf).into()))
    }

    /// Try to collect the buffer
    #[inline]
    fn try_collect_buf(&mut self) {
        if !self.buf.is_empty() { self.push_buf_as_token() }
    }
}

pub struct SyntacticParser {
    tree: Node,
}

impl SyntacticParser {
    pub fn new() -> Self {
        Self { tree: Node::List(vec![]) }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) {
        // let mut nest: usize = 0;
        let mut current: &mut Node = &mut self.tree;
        use Node::*;
        for token in tokens {
            match token.0.as_str() {
                "(" => seq!(/* nest += 1, */ current = self.tree.push(List(vec![]))),
                ")" => seq!(/* nest -= 1 */), // TODO: Set `current` to the previous one.
                _ => {
                    current.push(Symbol(token.try_into().unwrap_or_else(|err: Error| {
                        // TODO: Diagnostics message output.
                        panic!("{}", err.message());
                    })));
                }
            }
        }
    }

    pub fn tree(self) -> Node {
        self.tree
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::Node;
    use super::{LexicalParser, SyntacticParser, Token};

    #[allow(unused)]
    fn strs_to_tokens(strs: Vec<&str>) -> Vec<Token> {
        strs.into_iter().map(|string| string.into()).collect()
    }

    #[test]
    fn lexical_parse_str() {
        let mut lexer;
        lexer = LexicalParser::new();
        lexer.parse_str("($if #t #t #f)");
        assert_eq!(*lexer.tokens(), strs_to_tokens(vec!["(", "$if", "#t", "#t", "#f", ")"]));
        lexer = LexicalParser::new();
        lexer.parse_str("(eval     ())\n(display)");
        assert_eq!(*lexer.tokens(), strs_to_tokens(vec!["(", "eval", "(", ")", ")", "(", "display", ")"]));
    }

    #[test]
    fn syntactic_parse_tokens() {
        use Node::*;
        let mut lexer = LexicalParser::new();
        lexer.parse_str("apply display +");
        let mut parser = SyntacticParser::new();
        parser.parse(lexer.tokens());
        assert_eq!(parser.tree(), 
            List(vec![Symbol("apply".into()), Symbol("display".into()), Symbol("+".into())]));
    }
}
