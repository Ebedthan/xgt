use anyhow::Result;
use clap::ArgMatches;

use std::fmt::Display;
use std::fs::OpenOptions;

use std::sync::Arc;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub enum SearchField {
    #[default]
    All,
    Acc,
    Org,
    Gtdb,
    Ncbi,
}

pub fn is_taxonomy_field(search_field: &SearchField) -> bool {
    search_field == &SearchField::Gtdb || search_field == &SearchField::Ncbi
}

impl From<String> for SearchField {
    fn from(value: String) -> Self {
        if value == "acc" {
            SearchField::Acc
        } else if value == "org" {
            SearchField::Org
        } else if value == "gtdb" {
            SearchField::Gtdb
        } else if value == "ncbi" {
            SearchField::Ncbi
        } else {
            SearchField::All
        }
    }
}

impl Display for SearchField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Acc => write!(f, "accession"),
            Self::All => write!(f, "all"),
            Self::Gtdb => write!(f, "gtdb_taxonomy"),
            Self::Ncbi => write!(f, "ncbi_taxonomy"),
            Self::Org => write!(f, "ncbi_organism_name"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Csv,
    Json,
    Tsv,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Csv => write!(f, "csv"),
            Self::Json => write!(f, "json"),
            Self::Tsv => write!(f, "tsv"),
        }
    }
}

impl From<String> for OutputFormat {
    fn from(value: String) -> Self {
        if value == "tsv" {
            Self::Tsv
        } else if value == "json" {
            Self::Json
        } else {
            Self::Csv
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchArgs {
    // search name supplied by the user
    pub(crate) needle: Vec<String>,
    // search field on GTDB: either gtdb or ncbi
    pub(crate) search_field: SearchField,
    // enable partial search
    pub(crate) is_partial_search: bool,
    // returns entries' ids
    pub(crate) id: bool,
    // count entries in result
    pub(crate) count: bool,
    // search representative species only
    pub(crate) is_representative_species_only: bool,
    // search type material species only
    pub(crate) is_type_species_only: bool,
    // output file or None for stdout
    pub(crate) out: Option<String>,
    // output format: either csv, tsv or json
    pub(crate) outfmt: OutputFormat,
    // SSL certificate verification: true => disable, false => enable
    pub(crate) disable_certificate_verification: bool,
}

impl SearchArgs {
    pub fn add_needle(&mut self, needle: &str) {
        self.needle.push(needle.to_string());
    }

    pub fn get_needles(&self) -> &Vec<String> {
        &self.needle
    }

    pub fn set_search_field(&mut self, search_field: &str) {
        self.search_field = SearchField::from(search_field.to_string());
    }

    pub fn get_search_field(&self) -> SearchField {
        self.search_field.clone()
    }

    pub fn is_partial_search(&self) -> bool {
        self.is_partial_search
    }

    pub fn set_search_mode(&mut self, is_partial_search: bool) {
        self.is_partial_search = is_partial_search;
    }

    pub(crate) fn set_id(&mut self, b: bool) {
        self.id = b;
    }

    pub fn is_only_print_ids(&self) -> bool {
        self.id
    }

    pub(crate) fn set_count(&mut self, b: bool) {
        self.count = b;
    }

    pub fn is_only_num_entries(&self) -> bool {
        self.count
    }

    pub fn is_representative_species_only(&self) -> bool {
        self.is_representative_species_only
    }

    fn set_is_representative_species_only(&mut self, b: bool) {
        self.is_representative_species_only = b;
    }

    pub fn is_type_species_only(&self) -> bool {
        self.is_type_species_only
    }

    pub fn set_is_type_species_only(&mut self, b: bool) {
        self.is_type_species_only = b;
    }

    pub fn disable_certificate_verification(&self) -> bool {
        self.disable_certificate_verification
    }

    pub fn set_disable_certificate_verification(&mut self, b: bool) {
        self.disable_certificate_verification = b;
    }

    pub fn get_output(&self) -> Option<String> {
        self.out.clone()
    }

    pub(crate) fn set_output(&mut self, s: Option<String>) {
        self.out = s;
    }

    pub fn set_outfmt(&mut self, outfmt: String) {
        self.outfmt = OutputFormat::from(outfmt);
    }

    pub fn get_outfmt(&self) -> OutputFormat {
        self.outfmt.clone()
    }

    pub fn new() -> Self {
        SearchArgs::default()
    }

    pub fn from_arg_matches(args: &ArgMatches) -> Self {
        let mut search_args = SearchArgs::new();

        if let Some(file_path) = args.get_one::<String>("file") {
            let file = File::open(file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path));
            for line in BufReader::new(file)
                .lines()
                .map(|l| l.unwrap_or_else(|e| panic!("Failed to read line: {}", e)))
            {
                let nline = line;
                search_args.add_needle(&nline);
            }
        } else if let Some(name) = args.get_one::<String>("NAME") {
            search_args.add_needle(name)
        }

        search_args.set_search_field(args.get_one::<String>("field").unwrap());

        search_args.set_search_mode(args.get_flag("partial"));

        search_args.set_id(args.get_flag("id"));

        search_args.set_count(args.get_flag("count"));

        search_args.set_is_representative_species_only(args.get_flag("rep"));

        search_args.set_is_type_species_only(args.get_flag("type"));

        if args.contains_id("out") {
            search_args.set_output(args.get_one::<String>("out").cloned());
        }
        search_args.set_outfmt(args.get_one::<String>("outfmt").unwrap().to_string());

        search_args.set_disable_certificate_verification(args.get_flag("insecure"));

        search_args
    }
}

#[derive(Debug, Clone)]
pub struct GenomeArgs {
    pub(crate) accession: Vec<String>,
    pub(crate) output: Option<String>,
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
        let mut accession = Vec::new();

        if let Some(file_path) = arg_matches.get_one::<String>("file") {
            let file = File::open(file_path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path));
            accession = BufReader::new(file)
                .lines()
                .map(|l| l.expect("Cannot parse line"))
                .collect();
        } else {
            accession.push(
                arg_matches
                    .get_one::<String>("accession")
                    .unwrap_or_else(|| panic!("Missing accession value"))
                    .to_string(),
            );
        }

        GenomeArgs {
            accession,
            output: arg_matches.get_one::<String>("out").map(String::from),
            disable_certificate_verification: arg_matches.get_flag("insecure"),
        }
    }
}

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

pub fn write_to_output(buffer: &[u8], output: Option<String>) -> Result<()> {
    let mut writer: Box<dyn Write> = match output {
        Some(path) => Box::new(OpenOptions::new().append(true).create(true).open(path)?),
        None => Box::new(io::stdout()),
    };

    writer.write_all(buffer)?;
    writer.flush()?;

    Ok(())
}

pub fn get_agent(disable_certificate_verification: bool) -> anyhow::Result<ureq::Agent> {
    match disable_certificate_verification {
        true => {
            let tls_connector = Arc::new(
                native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .build()?,
            );
            Ok(ureq::AgentBuilder::new()
                .tls_connector(tls_connector)
                .build())
        }
        false => Ok(ureq::AgentBuilder::new().build()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::app;
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
    fn test_write_to_output() {
        let s = "Hello, world!";

        // Test writing to a file
        let file_path = "test.txt";
        let output = Some(file_path.to_owned());
        write_to_output(s.as_bytes(), output).unwrap();
        let contents = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(contents, s);

        std::fs::remove_file(file_path).unwrap();
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
