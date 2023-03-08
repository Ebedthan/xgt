use clap::{Arg, ArgAction, ColorChoice, Command};

pub fn build_app() -> Command {
    let clap_color_setting = if std::env::var_os("NO_COLOR").is_none() {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    };

    Command::new("xgt")
        .color(clap_color_setting)
        .about("Search and parse GTDB data")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("search")
                .about("Search GTDB by taxa name")
                .arg(Arg::new("name").required(true))
                .arg(
                    Arg::new("level")
                        .short('l')
                        .long("level")
                        .value_name("STR")
                        .help("Taxon level to search")
                        .default_value("genus")
                        .value_parser([
                            "species", "genus", "family", "order", "class", "phylum", "domain",
                        ]),
                )
                .arg(
                    Arg::new("partial")
                        .short('p')
                        .long("partial")
                        .action(ArgAction::SetTrue)
                        .help("Matching partially the taxon name"),
                )
                .arg(
                    Arg::new("count")
                        .short('c')
                        .long("count")
                        .action(ArgAction::SetTrue)
                        .help("Count the number of genomes"),
                )
                .arg(
                    Arg::new("raw")
                        .short('r')
                        .long("raw")
                        .action(ArgAction::SetTrue)
                        .help("Print raw response JSON "),
                )
                .arg(
                    Arg::new("id")
                        .short('i')
                        .long("id")
                        .action(ArgAction::SetTrue)
                        .help("Print only genome ID"),
                )
                .arg(
                    Arg::new("field")
                        .short('F')
                        .long("field")
                        .value_name("STR")
                        .default_value("gtdb_tax")
                        .value_parser(["all", "gtdb_tax", "ncbi_tax", "ncbi_org", "ncbi_id"])
                        .help("Search field"),
                )
                .arg(
                    Arg::new("rep")
                        .long("rep")
                        .action(ArgAction::SetTrue)
                        .help("Search GTDB species representative only"),
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .action(ArgAction::SetTrue)
                        .help("Search NCBI type material only"),
                ),
        )
}
