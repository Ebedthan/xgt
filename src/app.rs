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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app() {
        let app = build_app();
        let args = vec![
            "xgt",
            "search",
            "taxon_name",
            "-l",
            "class",
            "--count",
            "--raw",
        ];
        let matches = app.get_matches_from(args);
        assert_eq!(matches.subcommand_name(), Some("search"));
        let submatches = matches.subcommand_matches("search").unwrap();
        assert_eq!(
            submatches.get_one::<String>("name"),
            Some(&"taxon_name".to_owned())
        );
        assert_eq!(
            submatches.get_one::<String>("level"),
            Some(&"class".to_owned())
        );
        assert_eq!(submatches.get_flag("count"), true);
        assert_eq!(submatches.get_flag("raw"), true);
    }

    #[test]
    fn test_arg_parser() {
        let arg_parser = build_app().get_matches_from(vec![
            "xgt",
            "search",
            "taxon_name",
            "-l",
            "class",
            "--count",
            "--raw",
            "--id",
            "--partial",
            "--rep",
            "--type",
            "--field",
            "ncbi_tax",
        ]);
        let subcommand_parser = arg_parser.subcommand_matches("search").unwrap();
        assert_eq!(subcommand_parser.get_flag("count"), true);
        assert_eq!(subcommand_parser.get_flag("raw"), true);
        assert_eq!(subcommand_parser.get_flag("id"), true);
        assert_eq!(subcommand_parser.get_flag("partial"), true);
        assert_eq!(subcommand_parser.get_flag("rep"), true);
        assert_eq!(subcommand_parser.get_flag("type"), true);
        assert_eq!(
            subcommand_parser.get_one::<String>("name"),
            Some(&"taxon_name".to_owned())
        );
        assert_eq!(
            subcommand_parser.get_one::<String>("level"),
            Some(&"class".to_owned())
        );
        assert_eq!(
            subcommand_parser.get_one::<String>("field"),
            Some(&"ncbi_tax".to_owned())
        );
    }
}
