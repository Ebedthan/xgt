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
                .about("Search GTDB by taxa level")
                .arg(Arg::new("name").required(true))
                .arg(
                    Arg::new("level")
                        .short('l')
                        .long("level")
                        .value_name("STR")
                        .help("Set the taxon level to search")
                        .default_value("species")
                        .value_parser([
                            "species", "genus", "family", "order", "class", "phylum", "domain",
                        ]),
                )
                .arg(
                    Arg::new("count")
                        .short('c')
                        .long("count")
                        .action(ArgAction::SetFalse)
                        .help("Count the number of genome found for this taxonomic level"),
                )
                .arg(
                    Arg::new("download")
                        .short('d')
                        .long("download")
                        .action(ArgAction::SetFalse)
                        .help("Download all matched taxon metadata"),
                ),
        )
}
