mod cli;
mod shared;
mod generator;
mod parser;
mod validator;

fn main() -> std::io::Result<()> {
    cli::main()
}
