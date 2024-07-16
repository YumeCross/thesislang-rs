use std::rc::Rc;

use crate::parser::*;
use crate::evaluation::Context;

#[derive(Debug)]
pub struct Interpreter {
    root_ctx: Context,
    src: Rc<SrcInfo>
}

impl Interpreter {
    pub fn new() -> Self {
        let src_info = SrcInfo::new("", "");
        let rc = Rc::new(src_info);
        Self { root_ctx: Context::new(rc.clone()), src: rc.clone() }
    }
}
