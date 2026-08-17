mod api;
mod cache;
mod cli;
mod cmd;
mod utils;

use crate::cache::Cache;
use crate::cli::{Cli, Commands};

use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;

use std::io;

use cmd::{diff, genome, search, taxon};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // check for updates
    if cli.check_update {
        utils::check_update(cli.verbose)?;
        return Ok(());
    }

    if cli.cache_info {
        let cache = Cache::open()?;
        let info = cache.info()?;
        eprintln!("Cache: {}", cache::cache_path().display());
        eprintln!(
            "  Entries:  {} ({} expired)",
            info.entry_count, info.expired_count
        );
        eprintln!("  Size:     {:.1} MB", info.size_bytes as f64 / 1_048_576.0);
        return Ok(());
    }

    if cli.clear_cache {
        let cache = Cache::open()?;
        let n = cache.clear()?;
        eprintln!("Cache cleared ({} entries removed).", n);
        return Ok(());
    }

    // Determine effective cache setting, opt-out by default
    let use_cache = !cli.no_cache;

    // Check GTDB db status
    if cli.verbose {
        if utils::is_gtdb_db_online(false)? {
            eprintln!("GTDB status: online");
        } else {
            eprintln!("GTDB status: offline. Please try again later.");
            std::process::exit(0);
        }

        // Log API Version
        let api_version = utils::get_api_version(false)?;
        eprintln!("GTDB API Version: {}", api_version);
    }

    let command = match cli.command {
        Some(Commands::Search(args)) => {
            search::search(&args, use_cache)?;
        }
        Some(Commands::Genome(args)) => {
            if args.history {
                genome::get_genome_taxon_history(&args, use_cache)?;
            } else if args.metadata {
                genome::get_genome_metadata(&args, use_cache)?;
            } else {
                genome::get_genome_card(&args, use_cache)?
            }
        }
        Some(Commands::Taxon(args)) => {
            if args.search || args.all {
                taxon::search_taxon(&args, use_cache)?;
            } else if args.genomes {
                taxon::get_taxon_genomes(&args, use_cache)?;
            } else {
                taxon::get_taxon_name(&args, use_cache)?;
            }
        }
        Some(Commands::Diff(args)) => {
            diff::diff(&args, use_cache)?;
        }
        Some(Commands::Completions(args)) => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(args.shell, &mut cmd, bin_name, &mut io::stdout());
        }
        None => {
            // no subcommand provided.
            // print help and exit.
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
            std::process::exit(0);
        }
    };

    Ok(())
}
