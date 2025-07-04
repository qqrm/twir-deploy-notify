mod cli;
mod generator;
mod parser;
mod validator;

fn main() -> std::io::Result<()> {
    cli::main()
}
