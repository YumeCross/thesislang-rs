use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use ariadne::{Color, Fmt, Label};

use crate::error::{Error, ErrorKind};
use crate::{if_or, seq};
use crate::syntax::{Node, Symbol};

#[derive(Debug)]
pub struct SrcInfo {
    pub(crate) id: String,
    pub(crate) text: String
}

impl SrcInfo {
    pub fn new<S: Into<String>>(id: S, text: S) -> Self {
        Self { id: id.into(), text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token(String);

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str { self.0.as_str() }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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

impl Token {
    pub fn as_left_parentheses(&self) -> &str {
        match self.0.as_str() {
            ")" => "(",
            "]" => "[",
            "}" => "{",
            _ => panic!("Error: Not a parentheses.")
        }
    }

    pub fn as_right_parentheses(&self) -> &str {
        match self.0.as_str() {
            "(" => ")",
            "[" => "]",
            "{" => "}",
            _ => panic!("Error: Not a parentheses.")
        }
    }

    pub fn match_left_parentheses(&self, other: &String) -> bool {
        match self.0.as_str() {
            ")" | "]" | "}" => if_or!(other == self.as_left_parentheses(), true, false),
            _ => panic!("Error: Not a parentheses.")
        }
    }
}

/// Representing the current position as (line, column, index) of a source parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePos(usize, usize, usize);

impl SourcePos {
    pub fn ln(&self) -> usize { self.0 }

    pub fn col(&self) -> usize { self.1 }

    pub fn i(&self) -> usize { self.2 }

    pub fn next_ln(&mut self) { seq!(self.0 += 1, self.1 += 1, self.2 += 1) }

    pub fn next_col(&mut self) { seq!(self.1 += 1, self.2 += 1) }
}

impl From<(usize, usize, usize)> for SourcePos {
    fn from(value: (usize, usize, usize)) -> Self { Self(value.0, value.1, value.2) }
}

#[derive(Debug)]
pub struct LexicalParser {
    buf: String,
    pos: SourcePos,
    results: Vec<(SourcePos, Token)>,
    // 0 indicates initial state
    // 1 indicates parsing string literal
    // 2 indicates to unescape characters
    parsing_context: usize
}

impl LexicalParser {
    pub fn new() -> Self {
        Self { buf: "".to_string(), pos: (1, 1, 1).into(), results: vec![], parsing_context: 0 }
    }

    pub fn results(self) -> Vec<(SourcePos, Token)> {
        self.results
    }

    pub fn tokens(&self) -> Vec<Token> {
        self.results.iter().map(|pair| pair.1.clone()).collect()
    }

    pub fn parse_c(&mut self, ch: char) {
        match ch {
            ch if self.parsing_context == 1 => {
                self.buf.push(ch);
                if ch == '\\' {
                    self.parsing_context = 2;
                }

                if ch == '"' || ch == '\'' {
                    self.parsing_context = 0;
                }
            },
            '(' | '[' | '{' => {
                self.push_token(String::from(ch).into());
            }
            ')' | ']' | '}'=> {
                self.try_collect_buf();
                self.push_token(String::from(ch).into())
            }
            ',' | ';' => self.push_token(String::from(ch).into()),
            '\'' | '"'=> {
                self.buf.push(ch);
                if self.parsing_context == 0 {
                    self.parsing_context = 1;
                } else if self.parsing_context == 2 {
                    self.parsing_context = 1;
                }
            },
            ch if ch.is_ascii_whitespace() || ch == '\x0B' => self.try_collect_buf(),
            ch => self.buf.push(ch)
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
    src: Rc<RefCell<SrcInfo>>,
    tree: Node,
}

impl SyntacticParser {
    pub fn new(src: Rc<RefCell<SrcInfo>>) -> Self {
        Self { src, tree: Node::List(vec![]) }
    }

    fn first_quoted(s: &str) -> bool {
        match s.chars().nth(0).unwrap() {
            '\'' | '"' => true,
            _ => false
        }
    }

    pub fn parse(&mut self) {
        let mut nest: (i32, Vec<(SourcePos, String)>) = (0, vec![]); // (Nesting Depth, Parentheses Kind)
        let mut current = &mut self.tree;

        let src = self.src.borrow();

        let tokens = {
            let mut lexer = LexicalParser::new();
            lexer.parse_str(&src.text);
            lexer.results()
        };

        for (pos, token) in tokens {
            match token.0.as_str() {
                "(" | "[" | "{" => {
                    nest.0 += 1;
                    nest.1.push((pos, token.0.to_string()));
                    current = current.push(Node::List(vec![]));
                }
                ")" | "]" | "}" => {
                    nest.0 -= 1;
                    let last = nest.1.last().unwrap_or_else(|| {
                        Error::new(ErrorKind::InvalidSyntax)
                            .with_message(
                                format!("No corresponding '{}' can be found for '{token}'.",
                                token.as_left_parentheses()))
                            .with_span((pos.i()-1)..pos.i())
                            .report_error(&src, pos, format!("Invalid '{token}' here."));
                    });
                    if !token.match_left_parentheses(&last.1) {
                        use Color::*;
                        Error::new(ErrorKind::InvalidSyntax)
                            .with_message(
                        format!(
                    "'{}' is required, but only to found '{token}'", Token(last.1.clone()).as_right_parentheses()
                                )
                            )
                            .with_span(pos.i()-1..pos.i())
                            .with_label(
                                Label::new((src.id.clone(), (last.0.2-1)..last.0.2))
                                    .with_color(Fixed(86))
                                    .with_message(
                                        format!("Opening delimiter '{}{}", 
                                            last.1.clone().fg(Red), "' occurred here.".fg(Cyan)).fg(Cyan))
                                    .with_order(1)
                            )
                            .report_error(&src, pos,
                            format!("Invalid closing '{}{}.", token.fg(Fixed(81)), "' here".fg(Red)).fg(Red).to_string())
                    }
                    nest.1.pop();
                    current = &mut self.tree;
                    for _ in 0..nest.0 {
                        if let Node::List(ref mut list) = current {
                            current = list.last_mut().unwrap();
                        }
                    }
                }
                _ => {
                    let symbol = Symbol::try_from(token.0);
                    current.push(Node::Symbol(symbol.unwrap_or_else(|err| panic!("{err}"))));
                }
            }
        }

        if nest.0 != 0 {
            let last = nest.1.last().unwrap();
            Error::new(ErrorKind::InvalidSyntax)
                .with_message(
                    format!("No corresponding '{}' for '{}' was found.", Token(last.1.clone()).as_right_parentheses(), last.1))
                .with_span((last.0.i()-1)..last.0.i())
                .report_error(&src, last.0,
                    format!("Single '{}' found here.", last.1.clone().fg(Color::Red)));
        }
    }

    pub fn try_parse(&mut self) -> Result<(), Error> {
        let mut nest: (i32, Vec<(SourcePos, String)>) = (0, vec![]); // (Nesting Depth, Parentheses Kind)
        let mut current = &mut self.tree;

        let src = self.src.borrow();

        let tokens = {
            let mut lexer = LexicalParser::new();
            lexer.parse_str(&src.text);
            lexer.results()
        };

        for (pos, token) in tokens {
            match token.0.as_str() {
                "(" | "[" | "{" => {
                    nest.0 += 1;
                    nest.1.push((pos, token.0.to_string()));
                    current = current.push(Node::List(vec![]));
                }
                ")" | "]" | "}" => {
                    nest.0 -= 1;
                    let wrapped_last = nest.1.last();
                    let last: &(SourcePos, String);
                    match wrapped_last {
                        Some(val) => last = val,
                        None => return Err(Error::new(ErrorKind::InvalidSyntax)
                        .with_message(
                            format!("No corresponding '{}' can be found for '{token}'.",
                            token.as_left_parentheses()))
                        .with_span((pos.i()-1)..pos.i())
                        .return_error(&src, pos, format!("Invalid '{token}' here.")))
                    }
                    if !token.match_left_parentheses(&last.1) {
                        use Color::*;
                        return Err(Error::new(ErrorKind::InvalidSyntax)
                            .with_message(
                        format!(
                    "'{}' is required, but only to found '{token}'", Token(last.1.clone()).as_right_parentheses()
                                )
                            )
                            .with_span(pos.i()-1..pos.i())
                            .with_label(
                                Label::new((src.id.clone(), (last.0.2-1)..last.0.2))
                                    .with_color(Fixed(86))
                                    .with_message(
                                        format!("Opening delimiter '{}{}", 
                                            last.1.clone().fg(Red), "' occurred here.".fg(Cyan)).fg(Cyan))
                                    .with_order(1)
                            )
                            .return_error(&src, pos,
                            format!("Invalid closing '{}{}.", token.fg(Fixed(81)), "' here".fg(Red)).fg(Red).to_string()))
                    }
                    nest.1.pop();
                    current = &mut self.tree;
                    for _ in 0..nest.0 {
                        if let Node::List(ref mut list) = current {
                            current = list.last_mut().unwrap();
                        }
                    }
                },
                s if Self::first_quoted(s) => {
                    match Self::try_unquote(s) {
                        Ok(unquoted) => current.push(Node::String(unquoted)),
                        Err(err) => return Err(err)
                    };
                }
                _ => {
                    let symbol = Symbol::try_from(token.0);
                    current.push(Node::Symbol(symbol.unwrap_or_else(|err| panic!("{err}"))));
                }
            }
        }

        Ok(if nest.0 != 0 {
            let last = nest.1.last().unwrap();
            return Err(Error::new(ErrorKind::InvalidSyntax)
                .with_message(
                    format!("No corresponding '{}' for '{}' was found.", Token(last.1.clone()).as_right_parentheses(), last.1))
                .with_span((last.0.i()-1)..last.0.i())
                .return_error(&src, last.0,
                    format!("Single '{}' found here.", last.1.clone().fg(Color::Red))));
        })
    }

    pub fn try_unquote(s: &str) -> Result<String, Error> {
        let first = s.chars().nth(0).unwrap();
        let end = s.chars().last().unwrap();
        if first == end {
            Ok(s[1..s.len()-1].to_string())
        } else { Err(Error::new(ErrorKind::InvalidSyntax)) }
    }

    pub fn parse_untraced(&mut self, tokens: Vec<Token>) {
        let mut nest: (i32, Vec<String>) = (0, vec![]); // (Nesting Depth, Parentheses Kind)
        let mut current = &mut self.tree;

        for token in tokens {
            match token.0.as_str() {
                "(" | "[" | "{" => {
                    nest.0 += 1;
                    nest.1.push(token.0.to_string());
                    current = current.push(Node::List(vec![]));
                }
                ")" | "]" | "}" => {
                    nest.0 -= 1;
                    let _last = nest.1.last().unwrap_or_else(|| {
                        panic!("{}", 
                        Error::new(ErrorKind::InvalidSyntax)
                                .with_message(format!("No corresponding '{token}' can be found.")))
                    });
                    // TODO: Adjust implementation.
                    // let _ = token.match_left_parentheses(last).is_err_and(|err| panic!("{err}"));
                    nest.1.pop();
                    current = &mut self.tree;
                    for _ in 0..nest.0 {
                        if let Node::List(ref mut list) = current {
                            current = list.last_mut().unwrap();
                        }
                    }
                }
                _ => {
                    let symbol = Symbol::try_from(token);
                    current.push(Node::Symbol(symbol.unwrap_or_else(|err| panic!("{err}"))));
                }
            }
        }
    }

    pub fn reset(mut self) -> Node {
        core::mem::replace(&mut self.tree, Node::List(vec![]))
    }
    
    pub fn tree(self) -> Node {
        self.tree
    }

}

#[allow(unused)]
pub struct InfixTransformer {}

impl InfixTransformer {}

#[cfg(test)]
mod tests {
    use crate::{share, syntax::Node};
    use super::{SrcInfo, LexicalParser, SyntacticParser, Token};

    fn to_tokens(vector: Vec<&str>) -> Vec<Token> {
        vector.into_iter().map(|string| string.into()).collect()
    }

    #[test]
    fn lexical_parse_str() {
        let mut lexer;
        lexer = LexicalParser::new();
        lexer.parse_str("($if #t #t #f)");
        assert_eq!(*lexer.tokens(), to_tokens(vec!["(", "$if", "#t", "#t", "#f", ")"]));
        lexer = LexicalParser::new();
        lexer.parse_str("(eval     ())\n(display)");
        assert_eq!(*lexer.tokens(), to_tokens(vec!["(", "eval", "(", ")", ")", "(", "display", ")"]));
    }

    #[test]
    fn lexical_parse_literal() {
        let mut lexer: LexicalParser;
        lexer = LexicalParser::new();
        lexer.parse_str(r#"($if "test=parsing" #t)"#);
        println!("{:?}", lexer.results())
    }

    #[test]
    fn syntactic_parse_tokens_untraced() {
        use Node::*;
        let mut parser: SyntacticParser;
        
        parser = SyntacticParser::new(share!(SrcInfo::new("test-1", "apply display +".into())));
        parser.parse();
        assert_eq!(parser.tree(), 
            List(vec![Symbol("apply".into()), Symbol("display".into()), Symbol("+".into())]));
        
        parser = SyntacticParser::new(
            share!(SrcInfo::new(
                "test-2",
                "apply display (cons (list $if #t) [cons (list* #t #f) ()])".into()
            ))
        );
        parser.parse();
        assert_eq!(parser.tree(),
            List(vec!["apply".into(), "display".into(), 
                List(vec!["cons".into(), 
                    List(vec!["list".into(), "$if".into(), "#t".into()]),
                    List(vec!["cons".into(), 
                        List(vec!["list*".into(), "#t".into(), "#f".into()]),
                        List(vec![])]
                    )
                ])        
            ])
        )
    }

    #[test]
    fn syntactic_parse_parentheses_match() {
        use Node::*;
        let mut parser = SyntacticParser::new(share!(SrcInfo::new("test-1", "([{}])")));
        parser.parse();
        assert_eq!(parser.tree(), List(vec![List(vec![List(vec![List(vec![])])])]));
    }
}
