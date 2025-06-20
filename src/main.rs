mod api;
mod appi;
mod cli;
mod cmd;
mod utils;

use crate::cli::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use cmd::{genome, search, taxon};

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
                taxon::search_taxon(args)?;
            } else if args.genomes {
                taxon::get_taxon_genomes(args)?;
            } else {
                taxon::get_taxon_name(args)?;
            }
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests { /*
            use utils::OutputFormat;

            use super::*;
            use std::ffi::OsString;

            #[test]
            fn test_search_command() {
                let id = true;
                let count = true;
                let rsp = true;
                let tsp = true;

                let matches = cli::cli::build_app().get_matches_from(vec![
                    OsString::new(),
                    OsString::from("search"),
                    OsString::from("--file"),
                    OsString::from("test/test.txt"),
                    OsString::from("--id"),
                    OsString::from("-w"),
                    OsString::from("--count"),
                    OsString::from("--rep"),
                    OsString::from("--type"),
                    OsString::from("--out"),
                    OsString::from("out"),
                    OsString::from("--outfmt"),
                    OsString::from("json"),
                ]);

                let args = cli::search::SearchArgs::from_arg_matches(
                    matches.subcommand_matches("search").unwrap(),
                );

                assert_eq!(args.is_only_print_ids(), id);
                assert_eq!(args.is_only_num_entries(), count);
                assert_eq!(args.is_representative_species_only(), rsp);
                assert_eq!(args.is_type_species_only(), tsp);
                assert_eq!(args.get_output(), Some(String::from("out")));
                assert_eq!(args.get_outfmt(), OutputFormat::from("json".to_string()));
                assert!(args.is_whole_words_matching());
            }

            #[test]
            fn test_genome_command() {
                let args = vec![
                    "xgt",
                    "genome",
                    "NC_000912.1",
                    "--metadata",
                    "--out",
                    "met.json",
                ];
                let matches = cli::cli::build_app().get_matches_from(args);
                let sub_matches = matches.subcommand_matches("genome").unwrap();
                let args = cli::genome::GenomeArgs::from_arg_matches(sub_matches);
                assert_eq!(args.accession, vec!["NC_000912.1".to_string()]);
                assert_eq!(args.output, Some(String::from("met.json")));
            }*/
}
