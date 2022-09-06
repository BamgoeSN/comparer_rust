use clap::{Parser, Subcommand};
use comparer_rust::inputgen::generate_input;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate testcases of a given number and prints it out for debugging purpose
    Inputdebug {
        /// The number of testcases to generate
        num: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inputdebug { num } => {
            for tc in 1..=num {
                println!(r#"Testcase #{tc}"#);
                println!("{}", generate_input());
            }
        }
    }
}
