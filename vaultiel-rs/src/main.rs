//! Vaultiel CLI entry point.

use clap::Parser;
use std::process::ExitCode;
use vaultiel::cli::args::{Cli, Commands, CacheCommands};
use vaultiel::cli::output::Output;
use vaultiel::cli::{blocks, cache, content, create, delete, frontmatter, headings, info, links, lint, list, rename, resolve, search, tags, tasks};
use vaultiel::cli::lint::LintFormat;
use vaultiel::types::Priority;
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

        // Phase 3 commands
        Commands::GetTasks(args) => {
            let filter = tasks::TaskFilter {
                symbols: args.symbol.clone(),
                due_before: args.due_before.clone(),
                due_after: args.due_after.clone(),
                due_on: args.due_on.clone(),
                scheduled_before: args.scheduled_before.clone(),
                scheduled_after: args.scheduled_after.clone(),
                scheduled_on: args.scheduled_on.clone(),
                done_before: args.done_before.clone(),
                done_after: args.done_after.clone(),
                done_on: args.done_on.clone(),
                priority: args.priority.as_ref().and_then(|p| p.parse().ok()),
                contains: args.contains.clone(),
                has_metadata: args.has_metadata.clone(),
                links_to: args.links_to.clone(),
                tag: args.tag.clone(),
                has_block_ref: args.has_block_ref,
                block_ref: args.block_ref.clone(),
            };
            tasks::get_tasks(
                &vault,
                args.note.as_deref(),
                args.glob.as_deref(),
                filter,
                args.flat,
                &output,
            )
        }
        Commands::FormatTask(args) => {
            let priority: Option<Priority> = args.priority.as_ref().and_then(|p| p.parse().ok());
            let custom: std::collections::HashMap<String, String> = args
                .custom_metadata
                .iter()
                .filter_map(|s| {
                    let parts: Vec<&str> = s.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect();
            tasks::format_task_command(
                &args.desc,
                &args.symbol,
                args.scheduled.as_deref(),
                args.due.as_deref(),
                args.done.as_deref(),
                priority,
                &custom,
                &vault,
                &output,
            )
        }

        // Phase 4 commands
        Commands::Info(args) => {
            info::info(&vault, args.detailed, &output)
        }
        Commands::Lint(args) => {
            let format = LintFormat::from_str(&args.format).unwrap_or(LintFormat::Json);
            lint::lint(
                &vault,
                &args.only,
                &args.ignore,
                args.glob.as_deref(),
                &args.fail_on,
                format,
                &output,
            )
        }
        Commands::FindOrphans(args) => {
            lint::find_orphans(&vault, &args.exclude, &output)
        }
        Commands::FindBrokenLinks(args) => {
            lint::find_broken_links(&vault, args.note.as_deref(), &output)
        }

        // Phase 5 commands
        Commands::Cache(args) => {
            match &args.command {
                CacheCommands::Status => cache::status(&vault, &output),
                CacheCommands::Rebuild(rebuild_args) => {
                    cache::rebuild(&vault, rebuild_args.progress, &output)
                }
                CacheCommands::Clear => cache::clear(&vault, &output),
            }
        }
    }
}
