use std::cell::RefCell;
use std::rc::Rc;

use ariadne::Source;

use crate::parser::*;
use crate::evaluation::Context;

#[derive(Debug)]
pub struct Interpreter {
    interactive: bool,
    root_ctx: Context,
    src: Rc<RefCell<SrcInfo>>
}

impl Interpreter {
    pub fn new() -> Self {
        let src_info = SrcInfo::new("", "");
        let rc = Rc::new(RefCell::new(src_info));
        Self { interactive: true, root_ctx: Context::new(rc.clone()), src: rc.clone() }
    }

    pub fn read(&mut self, unit: &mut String) {
        let mut parser = SyntacticParser::new(self.src.clone());
        self.src.borrow_mut().text = core::mem::take(unit);
        let _ = parser.try_parse().is_err_and(|err| {
            err.report
                .unwrap()
                .finish()
                .print((self.src.borrow().id.clone(), Source::from(&self.src.borrow().text)))
                .unwrap();
            true
        });
        let _ = self.root_ctx.eval(parser.reset().into())
            .is_err_and(|err| {
                err.report
                    .unwrap()
                    .finish()
                    .print((self.src.borrow().id.clone(), Source::from(&self.src.borrow().text)))
                    .unwrap();
            if !self.interactive { std::process::exit(1); }
            false
        });
    }

    // TODO: Add history
    pub fn run_interactive(&mut self) -> ! {
        use std::io::{*, Write};
        self.interactive = true;
        self.src.borrow_mut().id = "<stdin>".to_string();
        loop {
            let mut line = String::new();
            print!("> "); // Print prompt
            stdout().flush().unwrap();
            stdin().read_line(&mut line).unwrap();
            line = line.trim().into();

            if line == "exit" { std::process::exit(0) }

            self.read(&mut line);
        }
    }
}
