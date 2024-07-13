mod cli;
mod error;
mod macros;
mod parser;
mod syntax;

fn main() {
    use cli::*;
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
    app.add_arg(Arg::new("script"));
    let args: Vec<String> = std::env::args().into_iter().collect();
    let map = match app.match_with(args[1..].to_vec()) {
        Ok(map) => map,
        Err(err) => seq!(println!("{}", err), return)
    };
    
    for (key, _val) in map {
        match key.as_str() {
            "help" => seq!(app.print_help(), break),
            "version" => seq!(println!(env!("CARGO_PKG_VERSION")), break),
            "script" => todo!("Add script execution."),
            _ => {}
        }
    }
}
