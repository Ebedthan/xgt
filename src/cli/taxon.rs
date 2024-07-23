use clap::ArgMatches;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone, PartialEq)]
pub struct TaxonArgs {
    pub(crate) name: Vec<String>,
    pub(crate) output: Option<String>,
    pub(crate) partial: bool,
    pub(crate) search: bool,
    pub(crate) search_all: bool,
    pub(crate) genomes: bool,
    pub(crate) reps_only: bool,
    pub(crate) disable_certificate_verification: bool,
}

impl TaxonArgs {
    pub fn get_name(&self) -> Vec<String> {
        self.name.clone()
    }

    pub fn get_output(&self) -> Option<String> {
        self.output.clone()
    }

    pub fn get_partial(&self) -> bool {
        self.partial
    }

    pub fn get_disable_certificate_verification(&self) -> bool {
        self.disable_certificate_verification
    }

    pub fn is_search(&self) -> bool {
        self.search
    }

    pub fn is_search_all(&self) -> bool {
        self.search_all
    }

    pub fn is_genome(&self) -> bool {
        self.genomes
    }

    pub fn is_reps_only(&self) -> bool {
        self.reps_only
    }

    pub fn from_arg_matches(arg_matches: &ArgMatches) -> Self {
        let mut names = Vec::new();

        if let Some(file_path) = arg_matches.get_one::<String>("file") {
            let file = File::open(file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path));
            names = BufReader::new(file)
                .lines()
                .map(|l| l.expect("Cannot parse line"))
                .collect();
        } else {
            names.push(
                arg_matches
                    .get_one::<String>("NAME")
                    .unwrap_or_else(|| panic!("Missing name value"))
                    .to_string(),
            );
        }

        TaxonArgs {
            name: names,
            output: arg_matches.get_one::<String>("out").map(String::from),
            partial: arg_matches.get_flag("partial"),
            search: arg_matches.get_flag("search"),
            search_all: arg_matches.get_flag("all"),
            genomes: arg_matches.get_flag("genomes"),
            reps_only: arg_matches.get_flag("reps"),
            disable_certificate_verification: arg_matches.get_flag("insecure"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::app;
    use std::ffi::OsString;

    #[test]
    fn test_get_name() {
        let args = TaxonArgs {
            name: vec!["name1".to_string(), "name2".to_string()],
            output: None,
            partial: false,
            search: false,
            search_all: false,
            genomes: false,
            reps_only: false,
            disable_certificate_verification: true,
        };

        assert_eq!(args.get_name(), vec!["name1", "name2"]);
    }

    #[test]
    fn test_get_partial() {
        let args = TaxonArgs {
            name: vec!["name1".to_string(), "name2".to_string()],
            output: None,
            partial: true,
            search: false,
            search_all: false,
            genomes: false,
            reps_only: false,
            disable_certificate_verification: true,
        };

        assert_eq!(args.get_partial(), true);
    }

    #[test]
    fn test_is_search() {
        let args = TaxonArgs {
            name: vec!["name1".to_string(), "name2".to_string()],
            output: None,
            partial: false,
            search: true,
            search_all: false,
            genomes: false,
            reps_only: false,
            disable_certificate_verification: true,
        };

        assert_eq!(args.is_search(), true);
    }

    #[test]
    fn test_taxon_from_args() {
        let name = vec!["g__Aminobacter".to_string()];

        let matches = app::build_app().get_matches_from(vec![
            OsString::new(),
            OsString::from("taxon"),
            OsString::from("g__Aminobacter"),
            OsString::from("--partial"),
        ]);

        let args = TaxonArgs::from_arg_matches(matches.subcommand_matches("taxon").unwrap());

        assert_eq!(args.get_name(), name);
        assert!(args.get_partial());
        assert!(!args.is_search());
        assert_eq!(args.get_output(), None);
    }

    #[test]
    fn test_taxon_from_args_2() {
        let name = vec!["g__Aminobacter".to_string(), "g__Rhizobium".to_string()];

        let matches = app::build_app().get_matches_from(vec![
            OsString::new(),
            OsString::from("taxon"),
            OsString::from("--file"),
            OsString::from("test/test2.txt"),
            OsString::from("-o"),
            OsString::from("out"),
            OsString::from("-s"),
        ]);

        let args = TaxonArgs::from_arg_matches(matches.subcommand_matches("taxon").unwrap());

        assert_eq!(args.get_name(), name);
        assert!(!args.get_partial());
        assert!(args.is_search());
        assert_eq!(args.get_output(), Some("out".to_string()));
    }
}
