
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{Error, ErrorKind};
use crate::parser::SrcInfo;
use crate::syntax::Symbol;
use super::term::{Term, *};

#[derive(Debug)]
pub struct Context {
    current_env: Env,
    src: Rc<SrcInfo>
}

impl Context {
    pub fn new(src: Rc<SrcInfo>) -> Self {
        Self { current_env: Env::new(), src }
    }

    pub fn reduce_leaf(&mut self, term: &mut Term) {
        let name: String;
        match (term as &mut dyn TermAccess<Symbol>).try_access() {
            Ok(symbol) => name = symbol.to_string(),
            Err(_) => return (),
        }
        match self.current_env.lookup(&name) {
            Some(ref mut term_ref) => {
                // TODO
            },
            None => Error::new(ErrorKind::FreeIdentifier)
                .with_message(format!("Failed to resolve '{name}'."))
                .report_error(&self.src, (0, 0, 0).into(), 
                    "".to_string()),
        }
    }

    pub fn reduce_branch(&mut self, term: &mut Term) {
        if term.is_branch() {

        }
    }
}

#[derive(Debug)]
pub struct Env {
    bindings: HashMap<String, Term>
}

// TODO: Implement linked environments.
impl Env {
    pub fn new() -> Self {
        Self { bindings: HashMap::new() }
    }

    pub fn lookup(&mut self, name: &String) -> Option<&mut Term> {
        self.bindings.get_mut(name)
    }

    pub fn insert(&mut self, name: &String, term: Term) -> Option<Term> {
        self.bindings.insert(name.to_string(), term)
    }
}
