use std::collections::HashMap;

use crate::{if_or, seq};
use crate::error::{Error, ErrorKind};


#[derive(Debug, Clone, Copy)]
pub struct Arg {
    id: (&'static str, char), // (Id, ShortId)
    optional: bool,
    /// Determine whether to stop parsing the rest args.
    interrupt: bool,
    parameterized: Parameter,
    prefix: char,
    info: (&'static str, &'static str), // (Description, Details)
}

impl Arg {
    pub fn new(id: &'static str) -> Self {
        let mut this = Self {
            id: (id, '\0'),
            optional: false,
            interrupt: false,
            parameterized: Parameter::No,
            prefix: '\0',
            info: ("", ""),
        };
        if id.starts_with("--") {
            if id.len() < 4 {
                panic!("Error: The id must has more than 4 characters.")
            }
            this.prefix = '-';
            this.optional = true;
        } else {
            this.parameterized = Parameter::Required
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

    pub fn interrupt(mut self) -> Self {
        seq!(self.interrupt = true, self)
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
        let id = self.id.0;
        let short_id = {
            if self.id.1 != '\0' {
                format!(", -{}", self.id.1)
            } else {
                "".into()
            }
        };
        let info_0 = self.info.0;
        let info_1 = if self.info.1 == "" { "".into() } else {
            "\n      ".to_owned() + self.info.1
        };
        format!(
            "{id}{short_id} {}\n      {info_0} {info_1}",
            {
                match self.parameterized {
                    Parameter::Optional(default) => format!("[default: {default}]"),
                    _ => "".to_string(),
                }
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parameter {
    No,
    Optional(&'static str),
    Required,
}

impl From<Parameter> for u8 {
    fn from(value: Parameter) -> Self {
        match value {
            Parameter::No => 0u8,
            Parameter::Optional(_) => 1u8,
            Parameter::Required => 2u8,
        }
    }
}

pub struct Command {
    exec_name: &'static str,
    help_content: &'static str,
    args: HashMap<String, Arg>,
    added_arg_names: Vec<String>,
    pos_args: Vec<Arg>,
}

impl Command {
    pub fn new(exec_name: &'static str, help_content: &'static str,) -> Self {
        Self {
            exec_name,
            help_content,
            args: HashMap::new(),
            added_arg_names: vec![],
            pos_args: vec![],
        }
    }

    pub fn add_arg(&mut self, arg: Arg) {
        if arg.prefix != '\0' {
            self.args.insert(arg.id.0.into(), arg);
            self.added_arg_names.push(arg.id.0.into());
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

    pub fn match_with(&self, args: Vec<String>) -> Result<HashMap<String, String>, Error> {
        let mut expect_flag: u8 = 0;
        let mut pos_parameters: Vec<String> = vec![];
        let mut results: HashMap<String, String> = HashMap::new();
        for (i, val) in args.iter().enumerate() {
            if expect_flag == 1 || expect_flag == 2 {
                seq!(expect_flag = 0, break)
            };
            match self.args.get(val) {
                Some(arg) => {
                    if !results.contains_key(&arg.id.0[1..]) {
                        // Note: The key for insertion has no "--".
                        results.insert(
                            arg.id.0[2..].to_string(),
                            arg.try_get_parameter(args.get(i + 1)),
                        );
                        if_or!(arg.interrupt, return Ok(results));
                        expect_flag = arg.parameterized.into();
                        continue;
                    } else {
                        panic!("Error: Duplicate parameter of '{}' was found.", arg.id.0)
                    }
                }
                None => pos_parameters.push(val.clone()),
            }
        }
        if expect_flag == 2 {
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
            return Err(Error::new(ErrorKind::CommandFailed).with_message(format!(
                "Error: Required argument '{}' was not found.",
                required_arg_id
            )));
        }
        Ok(results)
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
            // Don't use direct iteration to make sure
            // the output order follows the the argument order
            for arg_id in &self.added_arg_names {
                let arg = self.args[arg_id];
                string += format!("\n   {}", arg.help()).as_str();
            }
            string
        };
        let exec_name = self.exec_name;
        let help_content = self.help_content;
        println!(
            r#"Usage: {exec_name} [options]{pos_args}
      {help_content}

Options:{arg_helps}"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Arg, Command, Parameter::*};

    #[test]
    fn command_match_with_1() {
        use std::collections::HashMap;

        let mut command = Command::new("cli-test", "");
        command.add_arg(
            Arg::new("--help")
                .short_id('h')
                .parameterize(Optional("\"\""))
                .interrupt(),
        );
        command.add_arg(Arg::new("--version").short_id('v').interrupt());
        let mut map: HashMap<String, String>;
        map = command.match_with(vec!["--help".into(), "test".into()]).unwrap();
        assert_eq!(map, HashMap::from([("help".into(), "test".into())]));
        map = command.match_with(vec!["--help".into()]).unwrap();
        assert_eq!(map, HashMap::from([("help".into(), "\"\"".into())]));
        map = command.match_with(vec!["--version".into(), "--help".into()]).unwrap();
        assert_eq!(map, HashMap::from([("version".into(), "".into())]));
    }

    #[test]
    fn command_match_with_2() {
        let mut command = Command::new("cli-test", "");
        command.add_arg(Arg::new("--version").short_id('v').interrupt());
        command.add_arg(Arg::new("script"));
        assert_eq!(command.match_with(vec![]).unwrap_err().message(), "Error: Required argument 'script' was not found.");
    }
}
