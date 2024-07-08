mod api;
mod app;
mod cmd;

use std::env;

use anyhow::Result;
use cmd::{genome, search, taxon, utils};

fn main() -> Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());
    let subcommand = matches.subcommand();

    match subcommand {
        Some(("search", sub_matches)) => {
            let args = utils::SearchArgs::from_arg_matches(sub_matches);
            if sub_matches.get_flag("partial") {
                search::partial_search(args)?;
            } else {
                search::exact_search(args)?;
            }
        }
        Some(("genome", sub_matches)) => handle_genome_command(sub_matches)?,
        Some(("taxon", sub_matches)) => handle_taxon_command(sub_matches)?,
        _ => unreachable!("Implemented correctly"),
    };

    Ok(())
}

fn handle_genome_command(sub_matches: &clap::ArgMatches) -> Result<()> {
    let args = utils::GenomeArgs::from_arg_matches(sub_matches);
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
    let args = utils::TaxonArgs::from_arg_matches(sub_matches);
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
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_search_command() {
        let id = true;
        let count = true;
        let rsp = true;
        let tsp = true;

        let matches = app::build_app().get_matches_from(vec![
            OsString::new(),
            OsString::from("search"),
            OsString::from("--file"),
            OsString::from("test/test.txt"),
            OsString::from("--id"),
            OsString::from("--partial"),
            OsString::from("--count"),
            OsString::from("--rsp"),
            OsString::from("--tsp"),
            OsString::from("--out"),
            OsString::from("out"),
        ]);

        let args =
            utils::SearchArgs::from_arg_matches(matches.subcommand_matches("search").unwrap());

        assert_eq!(args.get_gid(), id);
        assert_eq!(args.get_count(), count);
        assert_eq!(args.get_rep(), rsp);
        assert_eq!(args.get_type_material(), tsp);
        assert_eq!(args.get_out(), Some(String::from("out")));
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
        let matches = app::build_app().get_matches_from(args);
        let sub_matches = matches.subcommand_matches("genome").unwrap();
        let args = utils::GenomeArgs::from_arg_matches(sub_matches);
        assert_eq!(args.accession, vec!["NC_000912.1".to_string()]);
        assert_eq!(args.output, Some(String::from("met.json")));
    }
}
