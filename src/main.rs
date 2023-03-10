extern crate anyhow;

mod api;
mod app;
mod cmd;

use std::env;

use anyhow::Result;
use cmd::{genome, search, utils};

fn main() -> Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());

    match matches.subcommand() {
        Some(("search", sub_matches)) => {
            let args = utils::SearchArgs::from(vec![
                (
                    "needle",
                    sub_matches
                        .get_one::<String>("name")
                        .expect("name to search is required"),
                ),
                (
                    "level",
                    sub_matches
                        .get_one::<String>("level")
                        .expect("level is required"),
                ),
                (
                    "partial",
                    &utils::bool_as_string(sub_matches.get_flag("partial")),
                ),
                (
                    "count",
                    &utils::bool_as_string(sub_matches.get_flag("count")),
                ),
                ("raw", &utils::bool_as_string(sub_matches.get_flag("raw"))),
                ("id", &utils::bool_as_string(sub_matches.get_flag("id"))),
            ]);
            search::search_gtdb(args).expect("Something went wrong");
        }
        Some(("genome", sub_matches)) => {
            if sub_matches.get_flag("history") {
                genome::genome_gtdb(
                    sub_matches.get_one::<String>("accession").unwrap(),
                    api::GenomeRequestType::TaxonHistory,
                    sub_matches.get_flag("raw"),
                )?;
            } else if sub_matches.get_flag("metadata") {
                genome::genome_gtdb(
                    sub_matches.get_one::<String>("accession").unwrap(),
                    api::GenomeRequestType::Metadata,
                    sub_matches.get_flag("raw"),
                )?;
            } else {
                genome::genome_gtdb(
                    sub_matches.get_one::<String>("accession").unwrap(),
                    api::GenomeRequestType::Card,
                    sub_matches.get_flag("raw"),
                )?;
            }
        }
        _ => unreachable!("Implemented correctly"),
    };

    Ok(())
}
