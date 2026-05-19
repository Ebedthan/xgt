mod api;
mod cli;
mod cmd;
mod utils;

use crate::cli::{Cli, Commands};
use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use cmd::{genome, search, taxon};
use std::io;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check GTDB db status
    if cli.verbose {
        if utils::is_gtdb_db_online(true)? {
            eprintln!("GTDB status: online");
        } else {
            eprintln!("GTDB status: offline. Please try again later.");
            std::process::exit(0);
        }

        // Log API Version
        let api_version = utils::get_api_version(true)?;
        eprintln!("GTDB API Version: {}", api_version);
    }

    match cli.command {
        Commands::Search(args) => {
            search::search(&args)?;
        }
        Commands::Genome(args) => {
            if args.history {
                genome::get_genome_taxon_history(&args)?;
            } else if args.metadata {
                genome::get_genome_metadata(&args)?;
            } else {
                genome::get_genome_card(&args)?
            }
        }
        Commands::Taxon(args) => {
            if args.search || args.all {
                taxon::search_taxon(&args)?;
            } else if args.genomes {
                taxon::get_taxon_genomes(&args)?;
            } else {
                taxon::get_taxon_name(&args)?;
            }
        }
        Commands::Completions(args) => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(args.shell, &mut cmd, bin_name, &mut io::stdout());
        }
    };

    Ok(())
}
