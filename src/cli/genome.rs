use clap::ArgMatches;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone)]
/// Genome subcmd arguments.
pub struct GenomeArgs {
    // Accession
    pub(crate) accession: Vec<String>,
    // Output format
    pub(crate) output: Option<String>,
    // Check SSL peer verification
    pub(crate) disable_certificate_verification: bool,
}

impl GenomeArgs {
    pub fn get_accession(&self) -> Vec<String> {
        self.accession.clone()
    }

    pub fn get_output(&self) -> Option<String> {
        self.output.clone()
    }

    pub fn get_disable_certificate_verification(&self) -> bool {
        self.disable_certificate_verification
    }

    pub fn from_arg_matches(arg_matches: &ArgMatches) -> Self {
        let accession = match arg_matches.get_one::<String>("file") {
            Some(file_path) => {
                let file = File::open(file_path).expect("Failed to open file");
                BufReader::new(file)
                    .lines()
                    .map(|l| l.expect("Cannot parse line"))
                    .collect()
            }
            None => vec![arg_matches
                .get_one::<String>("accession")
                .expect("Missing accession value")
                .to_string()],
        };

        GenomeArgs {
            accession,
            output: arg_matches.get_one::<String>("out").cloned(),
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
    fn test_get_accession() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            output: None,
            disable_certificate_verification: true,
        };

        assert_eq!(genome_args.get_accession(), vec!["NC_000001.11"]);
    }

    #[test]
    fn test_get_output() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            output: Some(String::from("output4.txt")),
            disable_certificate_verification: true,
        };

        assert_eq!(genome_args.get_output(), Some(String::from("output4.txt")));
    }

    #[test]
    fn test_genome_from_args() {
        let name = vec!["GCF_018555685.1".to_string()];

        let matches = app::build_app().get_matches_from(vec![
            OsString::new(),
            OsString::from("genome"),
            OsString::from("GCF_018555685.1"),
        ]);

        let args = GenomeArgs::from_arg_matches(matches.subcommand_matches("genome").unwrap());

        assert_eq!(args.get_accession(), name);
        assert_eq!(args.get_output(), None);
    }

    #[test]
    fn test_genome_from_args_2() {
        let name = vec!["GCF_018555685.1".to_string(), "GCF_900445235.1".to_string()];

        let matches = app::build_app().get_matches_from(vec![
            OsString::new(),
            OsString::from("genome"),
            OsString::from("--file"),
            OsString::from("test/acc.txt"),
            OsString::from("-o"),
            OsString::from("out"),
        ]);

        let args = GenomeArgs::from_arg_matches(matches.subcommand_matches("genome").unwrap());

        assert_eq!(args.get_accession(), name);
        assert_eq!(args.get_output(), Some("out".to_string()));
    }
}
