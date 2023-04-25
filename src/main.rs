extern crate anyhow;

mod api;
mod app;
mod cmd;

use std::env;

use anyhow::Result;
use cmd::{genome, search, taxon, utils};

fn main() -> Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());

    match matches.subcommand() {
        Some(("search", sub_matches)) => {
            let args = utils::SearchArgs::from_arg_matches(sub_matches);
            search::search_gtdb(args)?;
        }
        Some(("genome", sub_matches)) => {
            let args = utils::GenomeArgs::from_arg_matches(sub_matches);
            genome::genome_gtdb(args)?;
        }
        Some(("taxon", sub_matches)) => {
            let args = utils::TaxonArgs::from_arg_matches(sub_matches);
            taxon::get_taxon_name(args)?;
        }
        _ => unreachable!("Implemented correctly"),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_search_command() {
        let needle = vec!["Aminobacter".to_string(), "Rhizobium".to_string()];
        let level = "phylum".to_string();
        let id = true;
        let partial = true;
        let count = true;
        let raw = true;
        let rep = true;
        let type_material = true;

        let matches = app::build_app().get_matches_from(vec![
            OsString::new(),
            OsString::from("search"),
            OsString::from("--file"),
            OsString::from("test/test.txt"),
            OsString::from("--level"),
            OsString::from(level.clone()),
            OsString::from("--id"),
            OsString::from("--partial"),
            OsString::from("--count"),
            OsString::from("--raw"),
            OsString::from("--rep"),
            OsString::from("--type"),
            OsString::from("--out"),
            OsString::from("out"),
        ]);

        let args =
            utils::SearchArgs::from_arg_matches(matches.subcommand_matches("search").unwrap());

        assert_eq!(args.get_needle(), needle);
        assert_eq!(args.get_level(), level);
        assert_eq!(args.get_gid(), id);
        assert_eq!(args.get_partial(), partial);
        assert_eq!(args.get_count(), count);
        assert_eq!(args.get_raw(), raw);
        assert_eq!(args.get_rep(), rep);
        assert_eq!(args.get_type_material(), type_material);
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
        assert_eq!(args.request_type, api::GenomeRequestType::Metadata);
        assert_eq!(args.output, Some(String::from("met.json")));
    }
}
