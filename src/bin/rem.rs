use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use rempower::cli::{Cli, Commands};
use rempower::subcommands::dns::perform;
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dns(args) => {
            perform(args)?;
        }
        Commands::Completions { shell } => {
            generate_completions(shell);
        }

    }

    Ok(())
}

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "rem", &mut io::stdout());
}
