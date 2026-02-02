//! Vaultiel CLI entry point.

use clap::Parser;
use std::process::ExitCode;
use vaultiel::cli::args::{Cli, Commands};
use vaultiel::cli::output::Output;
use vaultiel::cli::{content, create, delete, frontmatter, list, resolve, search};
use vaultiel::config::Config;
use vaultiel::error::{exit_code, VaultError};
use vaultiel::vault::Vault;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli) {
        Ok(()) => ExitCode::from(exit_code::SUCCESS as u8),
        Err(e) => {
            if !cli.quiet {
                eprintln!("Error: {}", e);
            }
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

fn run(cli: &Cli) -> Result<(), VaultError> {
    // Load config
    let config = Config::load()?;

    // Resolve vault path
    let vault_path = config.resolve_vault_path(cli.vault.as_deref())?;
    let vault = Vault::new(vault_path, config)?;

    // Create output helper
    let output = Output::new(cli.output_format(), cli.quiet);

    // Dispatch command
    match &cli.command {
        Commands::List(args) => list::run(&vault, args, &output),
        Commands::Create(args) => create::run(&vault, args, &output),
        Commands::Delete(args) => delete::run(&vault, args, &output),
        Commands::Search(args) => search::run(&vault, args, &output),
        Commands::Resolve(args) => resolve::run(&vault, args, &output),
        Commands::GetContent(args) => content::get_content(&vault, args, &output),
        Commands::SetContent(args) => content::set_content(&vault, args, &output),
        Commands::AppendContent(args) => content::append_content(&vault, args, &output),
        Commands::PrependContent(args) => content::prepend_content(&vault, args, &output),
        Commands::ReplaceContent(args) => content::replace_content(&vault, args, &output),
        Commands::GetFrontmatter(args) => frontmatter::get_frontmatter(&vault, args, &output),
        Commands::ModifyFrontmatter(args) => frontmatter::modify_frontmatter(&vault, args, &output),
        Commands::RemoveFrontmatter(args) => frontmatter::remove_frontmatter(&vault, args, &output),
    }
}
