extern crate anyhow;

mod app;
mod cmd;

use std::env;

use anyhow::Result;
use cmd::search;

fn main() -> Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());

    match matches.subcommand() {
        Some(("search", sub_matches)) => {
            println!("count: {:?}", sub_matches.get_flag("count"));
            search::search_gtdb(
                sub_matches
                    .get_one::<String>("name")
                    .expect("name to search is required"),
                sub_matches
                    .get_one::<String>("level")
                    .expect("level is required"),
                sub_matches.get_flag("partial"),
                sub_matches.get_flag("count"),
            )
            .expect("Something went wrong")
        }
        _ => unreachable!("Implemented correctly"),
    };

    Ok(())
}
