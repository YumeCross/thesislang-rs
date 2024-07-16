use std::fmt::Debug;

use crate::error::Error;
use super::term::Term;
use super::context::Context;

pub trait Combiner {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeFn {
    func: Box<fn(Term, Context) -> Result<Term, Error>>
}

impl Combiner for NativeFn {}
