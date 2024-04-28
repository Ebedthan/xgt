use std::path::Path;

use clap::{Arg, ArgAction, Command};

pub fn build_app() -> Command {
    Command::new("xgt")
        .about("Search and parse GTDB data")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("search")
                .about("Search GTDB by taxa name")
                .arg(Arg::new("name").conflicts_with("file").help("taxon name"))
                .arg(
                    Arg::new("count")
                        .short('c')
                        .long("count")
                        .action(ArgAction::SetTrue)
                        .help("Count the number of genomes"),
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
                    Arg::new("file")
                        .short('f')
                        .long("file")
                        .value_name("FILE")
                        .help("Search from name in FILE"),
                )
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
                    Arg::new("raw")
                        .short('r')
                        .long("raw")
                        .action(ArgAction::SetTrue)
                        .help("Output raw JSON"),
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
                    Arg::new("raw")
                        .short('r')
                        .long("raw")
                        .action(ArgAction::SetTrue)
                        .help("Print raw JSON"),
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
                    Arg::new("name")
                        .conflicts_with("file")
                        .help("taxon name")
                        .value_parser(is_correct_taxon),
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
                    Arg::new("raw")
                        .short('r')
                        .long("raw")
                        .action(ArgAction::SetTrue)
                        .help("Output raw JSON"),
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

fn is_correct_taxon(s: &str) -> Result<String, String> {
    let prefixes = ["d__", "p__", "c__", "o__", "f__", "g__", "s__"];
    for prefix in &prefixes {
        if s.starts_with(prefix) {
            return Ok(s.to_string());
        }
    }
    Err("Taxon must be in greengenes format, e.g. g__Foo".to_string())
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
        assert!(submatches.get_flag("count"));
        assert!(submatches.get_flag("raw"));
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
        assert!(subcommand_parser.get_flag("count"));
        assert!(subcommand_parser.get_flag("raw"));
        assert!(subcommand_parser.get_flag("id"));
        assert!(subcommand_parser.get_flag("partial"));
        assert!(subcommand_parser.get_flag("rep"));
        assert!(subcommand_parser.get_flag("type"));
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

    #[test]
    fn verify_cmd() {
        build_app().debug_assert();
    }

    #[test]
    fn test_is_correct_taxon() {
        // Positive test cases
        assert_eq!(
            is_correct_taxon("d__Bacteria"),
            Ok("d__Bacteria".to_string())
        );
        assert_eq!(
            is_correct_taxon("g__Actinobacteria"),
            Ok("g__Actinobacteria".to_string())
        );
        assert_eq!(
            is_correct_taxon("s__Staphylococcus aureus"),
            Ok("s__Staphylococcus aureus".to_string())
        );

        // Negative test cases
        assert_eq!(
            is_correct_taxon("Bacteria"),
            Err("Taxon must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_correct_taxon("d_"),
            Err("Taxon must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_correct_taxon("Actinobacteria"),
            Err("Taxon must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_correct_taxon("__Actinobacteria"),
            Err("Taxon must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_correct_taxon("d_Actinobacteria"),
            Err("Taxon must be in greengenes format, e.g. g__Foo".to_string())
        );
    }
}
