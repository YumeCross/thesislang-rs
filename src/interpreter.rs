use std::cell::RefCell;
use std::rc::Rc;

use crate::parser::*;
use crate::evaluation::Context;

#[derive(Debug)]
pub struct Interpreter {
    root_ctx: Context,
    src: Rc<RefCell<SrcInfo>>
}

impl Interpreter {
    pub fn new() -> Self {
        let src_info = SrcInfo::new("", "");
        let rc = Rc::new(RefCell::new(src_info));
        Self { root_ctx: Context::new(rc.clone()), src: rc.clone() }
    }

    pub fn read(&mut self, unit: &mut String) {
        let mut parser = SyntacticParser::new(self.src.clone());
        self.src.borrow_mut().text = core::mem::take(unit);
        parser.parse();
        self.root_ctx.eval(parser.reset().into())
    }

    pub fn run_interactive(&mut self) -> ! {
        use std::io::{*, Write};
        self.src.borrow_mut().id = "<stdin>".to_string();
        loop {
            let mut line = String::new();
            print!("> "); // Print prompt
            stdout().flush().unwrap();
            stdin().read_line(&mut line).unwrap();
            self.read(&mut line);
        }
    }
}
