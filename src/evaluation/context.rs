
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{Error, ErrorKind};
use crate::if_or;
use crate::parser::SrcInfo;
use crate::syntax::Symbol;
use super::term::{Term, *};

#[derive(Debug)]
pub struct Context {
    pub(crate) env: Env,
    src: Rc<RefCell<SrcInfo>>
}

impl Context {
    pub fn new(src: Rc<RefCell<SrcInfo>>) -> Self {
        Self { env: Env::new(), src }
    }

    pub fn eval(&mut self, mut term: Term) -> Result<(), Error> {
        if !term.is_branch() {
            self.reduce_leaf(&mut term);
            Ok(())
        } else {
            self.reduce_branch(&mut term)
        }
    }

    pub fn reduce_leaf(&mut self, term: &mut Term) -> Result<(), Error> {
        let name: String;
        match (term as &mut dyn TermAccess<Symbol>).try_access() {
            Ok(symbol) => name = symbol.to_string(),
            Err(_) => return Ok(()),
        }
        match self.env.lookup(&name) {
            Some(ref mut term_ref) => {
                // TODO
                term.value_ref = term_ref.value_ref.clone();
                Ok(())
            },
            None => Err(Error::new(ErrorKind::FreeIdentifier)
                .with_message(format!("Failed to resolve '{name}'."))
                .return_error(&self.src.borrow(), (0, 0, 0).into(), 
                    "".to_string()))
        }
    }

    pub fn reduce_branch(&mut self, term: &mut Term) -> Result<(), Error> {
        if term.is_branch() {
            let front = term.sub_terms.front_mut().unwrap();
            match self.reduce_leaf(front) {
                Err(err) => return Err(err),
                _ => {}
            }
        }
        Ok(())
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
