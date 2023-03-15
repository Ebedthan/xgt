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
            let args = utils::SearchArgs::from_arg_matches(sub_matches);
            search::search_gtdb(args)?;
        }
        Some(("genome", sub_matches)) => {
            let args = utils::GenomeArgs::from_arg_matches(sub_matches);
            genome::genome_gtdb(args)?;
        }
        _ => unreachable!("Implemented correctly"),
    };

    Ok(())
}
