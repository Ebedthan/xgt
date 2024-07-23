use std::path::Path;

use clap::{Arg, ArgAction, Command};

pub fn build_app() -> Command {
    Command::new("xgt")
        .about("Query and parse GTDB data")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            // Search a taxon on GTDB
            Command::new("search")
                .about("Search a taxon on GTDB")
                .arg(
                    Arg::new("NAME").conflicts_with("file").help(
                        "a value (typically a species or genus name/taxon) used for searching.",
                    ),
                )
                .arg(
                    Arg::new("field")
                        .long("field")
                        .short('F')
                        .value_name("STR")
                        .default_value("all")
                        .value_parser(["all", "acc", "org", "gtdb", "ncbi"])
                        .help("search field"),
                )
                .arg(
                    Arg::new("partial")
                        .short('p')
                        .long("partial")
                        .action(ArgAction::SetTrue)
                        .help("perform partial matching"),
                )
                .arg(
                    Arg::new("rep")
                        .long("rep")
                        .short('r')
                        .action(ArgAction::SetTrue)
                        .help("search GTDB representative species only"),
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .short('t')
                        .action(ArgAction::SetTrue)
                        .help("search NCBI type species only"),
                )
                .arg(
                    Arg::new("id")
                        .short('i')
                        .long("id")
                        .action(ArgAction::SetTrue)
                        .help("only print matched genomes ID"),
                )
                .arg(
                    Arg::new("count")
                        .short('c')
                        .long("count")
                        .action(ArgAction::SetTrue)
                        .help("only print a count of matched genomes"),
                )
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .value_name("FILE")
                        .help("takes NAME from FILE"),
                )
                .arg(
                    Arg::new("out")
                        .short('o')
                        .long("out")
                        .help("output to FILE")
                        .value_name("FILE")
                        .value_parser(is_existing),
                )
                .arg(
                    Arg::new("outfmt")
                        .long("outfmt")
                        .short('O')
                        .help("output format")
                        .value_name("STR")
                        .default_value("csv")
                        .value_parser(["csv", "json", "tsv"]),
                )
                .arg(
                    Arg::new("insecure")
                        .short('k')
                        .long("insecure")
                        .help("disable SSL certificate verification")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("genome")
                .about("Information about a genome")
                .arg(
                    Arg::new("accession")
                        .conflicts_with("file")
                        .help("Genome accession"),
                )
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .value_name("FILE")
                        .help("Search from name in FILE"),
                )
                .arg(
                    Arg::new("history")
                        .short('H')
                        .long("history")
                        .action(ArgAction::SetTrue)
                        .help("Get genome taxon history"),
                )
                .arg(
                    Arg::new("metadata")
                        .short('m')
                        .long("metadata")
                        .action(ArgAction::SetTrue)
                        .conflicts_with("history")
                        .help("Get genome metadata"),
                )
                .arg(
                    Arg::new("out")
                        .short('o')
                        .long("out")
                        .help("Output raw JSON")
                        .value_name("FILE")
                        .value_parser(is_existing),
                )
                .arg(
                    Arg::new("insecure")
                        .short('k')
                        .long("insecure")
                        .help("Disable SSL certificate verification")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("taxon")
                .about("Information about a specific taxon")
                .arg(
                    Arg::new("NAME")
                        .conflicts_with("file")
                        .help("taxon name")
                        .value_parser(is_valid_taxon),
                )
                .arg(
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .value_name("FILE")
                        .help("Search from name in FILE"),
                )
                .arg(
                    Arg::new("out")
                        .short('o')
                        .long("out")
                        .help("Redirect output to FILE")
                        .value_name("FILE")
                        .value_parser(is_existing),
                )
                .arg(
                    Arg::new("partial")
                        .short('p')
                        .long("partial")
                        .action(ArgAction::SetTrue)
                        .help("Matching partially the taxon name"),
                )
                .arg(
                    Arg::new("search")
                        .short('s')
                        .long("search")
                        .action(ArgAction::SetTrue)
                        .help("Search for a taxon in current release"),
                )
                .arg(
                    Arg::new("all")
                        .short('a')
                        .long("all")
                        .action(ArgAction::SetTrue)
                        .help("Search for a taxon across all releases"),
                )
                .arg(
                    Arg::new("genomes")
                        .short('g')
                        .long("genomes")
                        .action(ArgAction::SetTrue)
                        .help("Get V taxon genomes"),
                )
                .arg(
                    Arg::new("reps")
                        .short('e')
                        .long("reps")
                        .action(ArgAction::SetTrue)
                        .help("Set taxon V genomes search to lookup reps seqs only"),
                )
                .arg(
                    Arg::new("insecure")
                        .short('k')
                        .long("insecure")
                        .help("Disable SSL certificate verification")
                        .action(ArgAction::SetTrue),
                ),
        )
}

fn is_valid_taxon(s: &str) -> Result<String, String> {
    let prefixes = ["d__", "p__", "c__", "o__", "f__", "g__", "s__"];
    for prefix in &prefixes {
        if s.starts_with(prefix) {
            return Ok(s.to_string());
        }
    }
    Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
}

fn is_existing(s: &str) -> Result<String, String> {
    if !Path::new(s).exists() {
        Ok(s.to_string())
    } else {
        Err("file should not already exists".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_existing() {
        // Test with a non-existing file
        let result = is_existing("test/acc.txt");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "file should not already exists");

        // Test with an existing file
        let result = is_existing("non_existing_file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "non_existing_file.txt".to_string());
    }

    #[test]
    fn test_app() {
        std::env::set_var("NO_COLOR", "true");
        let app = build_app();
        let args = vec!["xgt", "search", "p__taxon", "--count"];
        let matches = app.get_matches_from(args);
        assert_eq!(matches.subcommand_name(), Some("search"));
        let submatches = matches.subcommand_matches("search").unwrap();
        assert_eq!(
            submatches.get_one::<String>("NAME"),
            Some(&"p__taxon".to_owned())
        );
        assert!(submatches.get_flag("count"));
    }

    #[test]
    fn test_arg_parser() {
        let arg_parser = build_app().get_matches_from(vec![
            "xgt",
            "search",
            "p__taxon",
            "--count",
            "--id",
            "--partial",
            "--rep",
            "--type",
            "--field",
            "ncbi",
        ]);
        let subcommand_parser = arg_parser.subcommand_matches("search").unwrap();
        assert!(subcommand_parser.get_flag("count"));
        assert!(subcommand_parser.get_flag("id"));
        assert!(subcommand_parser.get_flag("partial"));
        assert!(subcommand_parser.get_flag("rep"));
        assert!(subcommand_parser.get_flag("type"));
        assert_eq!(
            subcommand_parser.get_one::<String>("NAME"),
            Some(&"p__taxon".to_owned())
        );
        assert_eq!(
            subcommand_parser.get_one::<String>("field"),
            Some(&"ncbi".to_owned())
        );
    }

    #[test]
    fn verify_cmd() {
        build_app().debug_assert();
    }

    #[test]
    fn test_is_valid_taxon() {
        // Positive test cases
        assert_eq!(is_valid_taxon("d__Bacteria"), Ok("d__Bacteria".to_string()));
        assert_eq!(
            is_valid_taxon("g__Actinobacteria"),
            Ok("g__Actinobacteria".to_string())
        );
        assert_eq!(
            is_valid_taxon("s__Staphylococcus aureus"),
            Ok("s__Staphylococcus aureus".to_string())
        );

        // Negative test cases
        assert_eq!(
            is_valid_taxon("Bacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("d_"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("Actinobacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("__Actinobacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("d_Actinobacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
    }
}
