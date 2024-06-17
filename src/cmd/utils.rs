use anyhow::Result;
use clap::ArgMatches;

use std::collections::HashMap;
use std::fs::OpenOptions;

use std::sync::Arc;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

/// A taxon is known by a particular name and given a particular ranking
/// Following GreenGenes notation the ranking comes first as a one letter code
/// followed by the name which start with a Upper case
/// Here the hashmap key will be the taxon name and the hashmap value the rank
pub type Taxa = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchArgs {
    pub(crate) taxa: Taxa,
    pub(crate) id: bool,
    pub(crate) count: bool,
    pub(crate) raw: bool,
    pub(crate) rep: bool,
    pub(crate) type_material: bool,
    pub(crate) out: Option<String>,
    pub(crate) disable_certificate_verification: bool,
}

impl Default for SearchArgs {
    fn default() -> Self {
        SearchArgs {
            taxa: Taxa::new(),
            id: false,
            count: false,
            raw: false,
            rep: false,
            type_material: false,
            out: None,
            disable_certificate_verification: false,
        }
    }
}

impl SearchArgs {
    pub fn add_taxon(&mut self, taxon: &str) {
        let parts = taxon.split("__").collect::<Vec<&str>>();
        self.taxa.insert(parts[1].to_string(), parts[0].to_string());
    }

    pub fn get_taxon_names(&self) -> Vec<String> {
        self.taxa.keys().map(|x| x.to_string()).collect()
    }

    pub fn get_taxon_rank(&self, name: &str) -> String {
        self.taxa.get(name).unwrap_or(&String::from("")).to_string()
    }

    pub fn get_gid(&self) -> bool {
        self.id
    }

    pub(crate) fn set_id(&mut self, b: bool) {
        self.id = b;
    }

    pub fn get_count(&self) -> bool {
        self.count
    }

    pub(crate) fn set_count(&mut self, b: bool) {
        self.count = b;
    }

    pub fn get_raw(&self) -> bool {
        self.raw
    }

    pub(crate) fn set_raw(&mut self, b: bool) {
        self.raw = b;
    }

    pub fn get_type_material(&self) -> bool {
        self.type_material
    }

    pub fn get_disable_certificate_verification(&self) -> bool {
        self.disable_certificate_verification
    }

    fn set_type_material(&mut self, b: bool) {
        self.type_material = b;
    }

    pub fn get_rep(&self) -> bool {
        self.rep
    }

    fn set_rep(&mut self, b: bool) {
        self.rep = b;
    }

    pub fn get_out(&self) -> Option<String> {
        self.out.clone()
    }

    pub(crate) fn set_out(&mut self, s: Option<String>) {
        self.out = s;
    }

    pub fn set_disable_certificate_verification(&mut self, b: bool) {
        self.disable_certificate_verification = b;
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
                search_args.add_taxon(&nline);
            }
        } else if let Some(name) = args.get_one::<String>("name") {
            search_args.add_taxon(&name)
        }

        search_args.set_id(args.get_flag("id"));

        search_args.set_count(args.get_flag("count"));

        search_args.set_raw(args.get_flag("raw"));

        search_args.set_rep(args.get_flag("rep"));

        search_args.set_type_material(args.get_flag("type"));

        if args.contains_id("out") {
            search_args.set_out(args.get_one::<String>("out").cloned());
        }

        search_args.set_disable_certificate_verification(args.get_flag("insecure"));

        search_args
    }
}

#[derive(Debug, Clone)]
pub struct GenomeArgs {
    pub(crate) accession: Vec<String>,
    pub(crate) raw: bool,
    pub(crate) output: Option<String>,
    pub(crate) disable_certificate_verification: bool,
}

impl GenomeArgs {
    pub fn get_accession(&self) -> Vec<String> {
        self.accession.clone()
    }

    pub fn get_raw(&self) -> bool {
        self.raw
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
            raw: arg_matches.get_flag("raw"),
            output: arg_matches.get_one::<String>("out").map(String::from),
            disable_certificate_verification: arg_matches.get_flag("insecure"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaxonArgs {
    pub(crate) name: Vec<String>,
    pub(crate) raw: bool,
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

    pub fn get_raw(&self) -> bool {
        self.raw
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
                    .get_one::<String>("name")
                    .unwrap_or_else(|| panic!("Missing name value"))
                    .to_string(),
            );
        }

        TaxonArgs {
            name: names,
            raw: arg_matches.get_flag("raw"),
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
            raw: false,
            output: None,
            disable_certificate_verification: true,
        };

        assert_eq!(genome_args.get_accession(), vec!["NC_000001.11"]);
    }

    #[test]
    fn test_get_raw() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            raw: true,
            output: None,
            disable_certificate_verification: true,
        };

        assert!(genome_args.get_raw());
    }

    #[test]
    fn test_get_output() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            raw: false,
            output: Some(String::from("output4.txt")),
            disable_certificate_verification: true,
        };

        assert_eq!(genome_args.get_output(), Some(String::from("output4.txt")));
    }

    #[test]
    fn test_get_name() {
        let args = TaxonArgs {
            name: vec!["name1".to_string(), "name2".to_string()],
            raw: false,
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
            raw: false,
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
            raw: false,
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
            OsString::from("--raw"),
        ]);

        let args = TaxonArgs::from_arg_matches(matches.subcommand_matches("taxon").unwrap());

        assert_eq!(args.get_name(), name);
        assert!(args.get_raw());
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
        assert!(!args.get_raw());
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
            OsString::from("--raw"),
        ]);

        let args = GenomeArgs::from_arg_matches(matches.subcommand_matches("genome").unwrap());

        assert_eq!(args.get_accession(), name);
        assert!(args.get_raw());
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
        assert!(!args.get_raw());
        assert_eq!(args.get_output(), Some("out".to_string()));
    }
}
