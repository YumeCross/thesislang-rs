mod command;
mod error;
mod macros;
mod parser;
mod syntax;
mod evaluation;
mod interpreter;

fn main() {
    use command::*;
    let mut app = Command::new("thesis", 
r#"The prototype of Thesis interpreter."#);
    app.add_arg(
        Arg::new("--help")
            .short_id('h')
            .description("Print the help message.")
            .interrupt());
    app.add_arg(
        Arg::new("--version")
            .short_id('v')
            .description("Print the current version.")
            .interrupt());
    app.add_arg(
        Arg::new("--output")
            .short_id('o')
            .parameterize(Parameter::Required)
            .description("Specify the path of the output file.")
    );
    app.add_arg(
        Arg::new("--target")
            .parameterize(Parameter::Optional("\"ast\""))
            .description("Specify the output target.")
            .details(
r#"The supported output targets are listed here. Note that only a work in progress target is support currently.
      - "ast": Output as a desugared abstract syntax tree (in list form)."#)
    );
    app.add_arg(
        Arg::new("script")
            .parameterize(Parameter::Optional("-")));
    let args: Vec<String> = std::env::args().into_iter().collect();
    let map = match app.match_with(args[1..].to_vec()) {
        Ok(map) => map,
        Err(err) => seq!(println!("{}", err), return)
    };
    
    for (key, val) in &map {
        match key.as_str() {
            "help" => seq!(app.print_help(), break),
            "version" => seq!(println!(env!("CARGO_PKG_VERSION")), break),
            // In the future, the implementation will only
            // evaluate the script without specifying '--output'.
            "script" => {
                if map.get("script").unwrap() == "-" {
                    run_loop()
                } else {
                    execute_script(val, map.get("output")).unwrap()
                }
            },
            "target" => match map.get("target").unwrap().as_str() {
                "ast" => continue,
                _ => panic!()
            },
            _ => {}
        }
    }
}

fn run_loop() -> ! {
    use interpreter::*;
    let mut instance = Interpreter::new();
    instance.run_interactive()
}

fn execute_script(path: &String, out: Option<&String>) -> Result<(), std::io::Error> {
    use std::fs::*;
    use std::io::Write;
    use parser::*;
    let input = std::fs::read(path);
    let content = String::from_utf8(match input {
        Ok(val) => val,
        Err(err) => return Err(err),
    }).unwrap_or_else(|err| {
        panic!("{err}");
    });
    let mut parser = SyntacticParser::new(share!(SrcInfo::new(path, &content)));
        parser.parse();
    match out {
        Some(out_path) => {
            let mut file = File::create(out_path)?;
            return write!(file, "{}", parser.tree())
        },
        None => Ok(())
    }
}
