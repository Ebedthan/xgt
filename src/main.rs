mod api;
mod cli;
mod cmd;
mod utils;

use std::env;

use anyhow::Result;
use cmd::{genome, search, taxon};

fn main() -> Result<()> {
    let matches = cli::app::build_app().get_matches_from(env::args_os());
    let subcommand = matches.subcommand();

    // Check GTDB db status
    if utils::is_gtdb_db_online(true)? {
        eprintln!("GTDB status: online");
    } else {
        eprintln!("GTDB status: offline. Please try again later.");
        std::process::exit(0);
    }

    // Log API Version
    let api_version = utils::get_api_version(true)?;
    eprintln!("GTDB API Version: {}", api_version);

    match subcommand {
        Some(("search", sub_matches)) => {
            let args = cli::search::SearchArgs::from_arg_matches(sub_matches);
            search::search(args)?;
        }
        Some(("genome", sub_matches)) => handle_genome_command(sub_matches)?,
        Some(("taxon", sub_matches)) => handle_taxon_command(sub_matches)?,
        _ => unreachable!("Implemented correctly"),
    };

    Ok(())
}

fn handle_genome_command(sub_matches: &clap::ArgMatches) -> Result<()> {
    let args = cli::genome::GenomeArgs::from_arg_matches(sub_matches);
    if sub_matches.get_flag("history") {
        genome::get_genome_taxon_history(args)?;
    } else if sub_matches.get_flag("metadata") {
        genome::get_genome_metadata(args)?;
    } else {
        genome::get_genome_card(args)?
    }
    Ok(())
}

fn handle_taxon_command(sub_matches: &clap::ArgMatches) -> Result<()> {
    let args = cli::taxon::TaxonArgs::from_arg_matches(sub_matches);
    if args.is_search() || args.is_search_all() {
        taxon::search_taxon(args)?;
    } else if args.is_genome() {
        taxon::get_taxon_genomes(args)?;
    } else {
        taxon::get_taxon_name(args)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use utils::OutputFormat;

    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_search_command() {
        let id = true;
        let count = true;
        let rsp = true;
        let tsp = true;

        let matches = cli::app::build_app().get_matches_from(vec![
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
        let matches = cli::app::build_app().get_matches_from(args);
        let sub_matches = matches.subcommand_matches("genome").unwrap();
        let args = cli::genome::GenomeArgs::from_arg_matches(sub_matches);
        assert_eq!(args.accession, vec!["NC_000912.1".to_string()]);
        assert_eq!(args.output, Some(String::from("met.json")));
    }
}
