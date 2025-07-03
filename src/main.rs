mod cli;
mod generator;
mod parser;

fn main() -> std::io::Result<()> {
    cli::main()
}
