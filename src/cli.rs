use std::collections::HashMap;

use crate::{if_or, seq};

#[derive(Debug, Clone, Copy)]
pub struct Arg {
    id: (&'static str, char),
    optional: bool,
    parameterized: Parameter,
    prefix: char,
    info: (&'static str, &'static str), // (Description, Details)
}

impl Arg {
    pub fn new(id: &'static str, opt: bool) -> Self {
        let mut this = Self {
            id: (id, '\0'),
            optional: opt,
            parameterized: Parameter::No,
            prefix: '\0',
            info: ("", ""),
        };
        if id.starts_with("--") {
            if id.len() < 4 {
                panic!("Error: The id must has more than 4 characters.")
            }
            this.prefix = '-'
        }
        this
    }

    pub fn parameterize(mut self, parameterized: Parameter) -> Self {
        seq!(self.parameterized = parameterized, self)
    }

    pub fn short_id(mut self, ch: char) -> Self {
        if self.prefix != '\0' {
            seq!(self.id.1 = ch, self)
        } else {
            panic!("Error: A positional arg doesn't support short id.")
        }
    }

    pub fn description(mut self, content: &'static str) -> Self {
        seq!(self.info.0 = content, self)
    }

    pub fn details(mut self, content: &'static str) -> Self {
        seq!(self.info.1 = content, self)
    }

    pub fn try_get_parameter(&self, parameter: Option<&String>) -> String {
        use Parameter::*;
        match self.parameterized {
            No => "".into(),
            Optional(default) => parameter.unwrap_or(&default.to_string()).clone(),
            Required => parameter
                .unwrap_or_else(|| panic!("Error: Parameter not found."))
                .clone(),
        }
    }

    pub fn help(&self) -> String {
        format!("{} {}\n      {} {}", self.id.0, {
            match self.parameterized {
                Parameter::Optional(default) => format!("[default: {}]", default),
                _ => "".to_string()
            }
        }, self.info.0, self.info.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parameter {
    No,
    Optional(&'static str),
    Required,
}

pub struct Command {
    exec_name: &'static str,
    args: HashMap<String, Arg>,
    pos_args: Vec<Arg>,
}

impl Command {
    pub fn new(exec_name: &'static str) -> Self {
        Self {
            exec_name,
            args: HashMap::new(),
            pos_args: vec![],
        }
    }

    pub fn add_arg(&mut self, arg: Arg) {
        if arg.prefix != '\0' {
            self.args.insert(arg.id.0.into(), arg);
            if arg.id.1 != '\0' {
                self.args.insert(format!("-{}", arg.id.1), arg);
            }
        } else {
            if arg.optional == true && self.pos_args.last().is_some_and(|arg| !arg.optional) {
                panic!("Error: Cannot add a optional argument after a required one.")
            }
            self.pos_args.push(arg);
        }
    }

    pub fn match_with(&self, args: Vec<String>) -> HashMap<String, String> {
        let mut expect_parameter: bool = false;
        let mut pos_parameters: Vec<String> = vec![];
        let mut results: HashMap<String, String> = HashMap::new();
        for (i, val) in args.iter().enumerate() {
            if !expect_parameter {
                match self.args.get(val) {
                    Some(arg) => {
                        if !results.contains_key(&arg.id.0[1..]) {
                            // Note: The key for insertion has no "--".
                            results.insert(
                                arg.id.0[2..].to_string(),
                                arg.try_get_parameter(args.get(i + 1)),
                            );
                            continue;
                        } else {
                            panic!("Error: Duplicate parameter of '{}' was found.", arg.id.0)
                        }
                    }
                    None => pos_parameters.push(val.clone()),
                }
            }
            if expect_parameter {
                expect_parameter = false
            };
        }
        if expect_parameter {
            panic!("Error: Required parameter not found.")
        }
        let pos_param_len = pos_parameters.len();
        let mut used_pos_arg = 0usize;
        let mut required_pos_arg = 0usize;
        let mut required_arg_id = "";
        if pos_param_len > self.pos_args.len() {
            panic!("Error: Too many parameters received.")
        }
        for arg in &self.pos_args {
            if_or!(
                !arg.optional,
                seq!(required_pos_arg += 1, required_arg_id = arg.id.0)
            );
            if used_pos_arg >= pos_param_len {
                continue;
            }
            results.insert(
                arg.id.0.into(),
                core::mem::take(&mut pos_parameters[used_pos_arg]),
            );
            used_pos_arg += 1;
        }
        if used_pos_arg < required_pos_arg {
            panic!(
                "Error: Required argument '{}' was not found.",
                required_arg_id
            )
        }
        results
    }

    pub fn print_help(&self) {
        let pos_args = {
            let mut string = String::new();
            string.reserve(self.pos_args.len() * 3);
            for arg in &self.pos_args {
                let bracket = if_or!(arg.optional, ('[', ']'), ('<', '>'));
                string += format!(" {}{}{} ", bracket.0, arg.id.0, bracket.1).as_str();
            }
            string
        };
        let arg_helps = {
            let mut string = String::new();
            let mut added_args: Vec<&str> = vec![];
            added_args.reserve(self.args.len() / 2);
            for arg in self.args.values() {
                if added_args.contains(&arg.id.0) { break }
                string += format!("  {}\n", arg.help()).as_str();
                added_args.push(arg.id.0);
            }
            string
        };
        println!(
r#"Usage: {} [options] {}
{}"#,
            self.exec_name, pos_args, arg_helps
        )
    }
}

// TODO: Add tests.
mod tests {}
