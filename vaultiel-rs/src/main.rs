//! Vaultiel CLI entry point.

use clap::Parser;
use std::process::ExitCode;
use vaultiel::cli::args::{Cli, Commands};
use vaultiel::cli::output::Output;
use vaultiel::cli::{blocks, content, create, delete, frontmatter, headings, links, list, rename, resolve, search, tags};
use vaultiel::config::Config;
use vaultiel::error::{ExitCode as VaultExitCode, VaultError};
use vaultiel::vault::Vault;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli) {
        Ok(code) => ExitCode::from(code.code() as u8),
        Err(e) => {
            if !cli.quiet {
                eprintln!("Error: {}", e);
            }
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

fn run(cli: &Cli) -> Result<VaultExitCode, VaultError> {
    // Load config
    let config = Config::load()?;

    // Resolve vault path
    let vault_path = config.resolve_vault_path(cli.vault.as_deref())?;
    let vault = Vault::new(vault_path, config)?;

    // Create output helper
    let output = Output::new(cli.output_format(), cli.quiet);

    // Dispatch command
    match &cli.command {
        // Phase 1 commands
        Commands::List(args) => {
            list::run(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::Create(args) => {
            create::run(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::Delete(args) => {
            delete::run(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::Search(args) => {
            search::run(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::Resolve(args) => {
            resolve::run(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::GetContent(args) => {
            content::get_content(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::SetContent(args) => {
            content::set_content(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::AppendContent(args) => {
            content::append_content(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::PrependContent(args) => {
            content::prepend_content(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::ReplaceContent(args) => {
            content::replace_content(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::GetFrontmatter(args) => {
            frontmatter::get_frontmatter(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::ModifyFrontmatter(args) => {
            frontmatter::modify_frontmatter(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }
        Commands::RemoveFrontmatter(args) => {
            frontmatter::remove_frontmatter(&vault, args, &output)?;
            Ok(VaultExitCode::Success)
        }

        // Phase 2 commands
        Commands::GetLinks(args) => {
            let filter = links::LinkFilter {
                context: args.context.clone(),
                embeds_only: args.embeds_only,
                no_embeds: args.no_embeds,
                media_only: args.media_only,
                notes_only: false,
            };
            links::get_links(&vault, &args.path, filter, &output)
        }
        Commands::GetInLinks(args) => {
            let filter = links::LinkFilter {
                context: args.context.clone(),
                embeds_only: args.embeds_only,
                no_embeds: args.no_embeds,
                media_only: args.media_only,
                notes_only: false,
            };
            links::get_in_links(&vault, &args.path, filter, &output)
        }
        Commands::GetOutLinks(args) => {
            let filter = links::LinkFilter {
                context: args.context.clone(),
                embeds_only: args.embeds_only,
                no_embeds: args.no_embeds,
                media_only: args.media_only,
                notes_only: false,
            };
            links::get_out_links(&vault, &args.path, filter, &output)
        }
        Commands::GetEmbeds(args) => {
            links::get_embeds(&vault, &args.path, args.media_only, args.notes_only, &output)
        }
        Commands::GetTags(args) => {
            if let Some(ref path) = args.path {
                tags::get_tags_from_note(&vault, path, &output)
            } else {
                tags::get_tags_vault(&vault, args.with_counts, args.nested, args.glob.as_deref(), &output)
            }
        }
        Commands::GetBlocks(args) => {
            blocks::get_blocks(&vault, &args.path, &output)
        }
        Commands::GetBlockRefs(args) => {
            blocks::get_block_refs(&vault, &args.path, &output)
        }
        Commands::GetHeadings(args) => {
            headings::get_headings(&vault, &args.path, args.min_level, args.max_level, args.nested, &output)
        }
        Commands::GetSection(args) => {
            let include_subheadings = args.include_subheadings && !args.exclude_subheadings;
            headings::get_section(&vault, &args.path, &args.heading, args.by_slug, include_subheadings, args.content_only, &output)
        }
        Commands::Rename(args) => {
            rename::rename(&vault, &args.from, &args.to, args.no_propagate, args.dry_run, &output)
        }
    }
}
