mod cli;
mod error;
mod macros;
mod parser;
mod syntax;

fn main() {
    use cli::*;
    let mut app = Command::new("thesis");
    app.add_arg(
        Arg::new("--help", true)
            .short_id('h')
            .description("Print the help message."));
    app.add_arg(
        Arg::new("--version", true)
            .short_id('v')
            .description("Print the version information."));

    let args: Vec<String> = std::env::args().into_iter().collect();
    let map = app.match_with(args[1..].to_vec());
    
    for (key, _val) in map {
        match key.as_str() {
            "help" => app.print_help(),
            "version" => seq!(println!(env!("CARGO_PKG_VERSION")), break),
            _ => {}
        }
    }
}
