use crate::utils::{OutputFormat, SearchField};
use clap::ArgMatches;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

/// Command line arguments struct for search cmd
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchArgs {
    // search name supplied by the user
    pub(crate) needle: Vec<String>,

    // search field on GTDB
    pub(crate) search_field: SearchField,

    // enable whole words matching
    pub(crate) is_whole_words_matching: bool,

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
    /// Add a needle needle attribute
    pub fn add_needle(&mut self, needle: &str) {
        self.needle.push(needle.to_string());
    }

    /// Getter for needle attribute
    pub fn get_needles(&self) -> &Vec<String> {
        &self.needle
    }

    /// Setter for search field attribute
    pub fn set_search_field(&mut self, search_field: &str) {
        self.search_field = SearchField::from(search_field.to_string());
    }

    /// Getter for search field attribute
    pub fn get_search_field(&self) -> SearchField {
        self.search_field.clone()
    }

    /// Is match only whole words enabled
    pub fn is_whole_words_matching(&self) -> bool {
        self.is_whole_words_matching
    }

    /// Setter for search mode attribute
    pub fn set_matching_mode(&mut self, is_whole_words_matching: bool) {
        self.is_whole_words_matching = is_whole_words_matching;
    }

    /// Setter for id attribute
    pub(crate) fn set_id(&mut self, b: bool) {
        self.id = b;
    }

    /// Check if tool was called in only prints ids mode
    pub fn is_only_print_ids(&self) -> bool {
        self.id
    }

    /// Setter for count attribute
    pub(crate) fn set_count(&mut self, b: bool) {
        self.count = b;
    }

    /// Check if tool was called in count entries mode
    pub fn is_only_num_entries(&self) -> bool {
        self.count
    }

    /// Check if tool was called with search representative species only
    pub fn is_representative_species_only(&self) -> bool {
        self.is_representative_species_only
    }

    /// Set the search representative species only mode
    fn set_is_representative_species_only(&mut self, b: bool) {
        self.is_representative_species_only = b;
    }

    /// Check if tool was ran in type species mode
    pub fn is_type_species_only(&self) -> bool {
        self.is_type_species_only
    }

    /// Set the type species only mode
    pub fn set_is_type_species_only(&mut self, b: bool) {
        self.is_type_species_only = b;
    }

    /// Check if SSL peer verification is enabled
    pub fn disable_certificate_verification(&self) -> bool {
        self.disable_certificate_verification
    }

    /// Set if tool should perform SSL peer verification
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

        search_args.set_matching_mode(args.get_flag("word"));

        search_args.set_id(args.get_flag("id"));

        search_args.set_count(args.get_flag("count"));

        search_args.set_is_representative_species_only(args.get_flag("rep"));

        search_args.set_is_type_species_only(args.get_flag("type"));

        if args.contains_id("out") {
            search_args.set_output(args.get_one::<String>("out").cloned());
        }
        if args.get_flag("count") || args.get_flag("id") {
            // If the user set --count or --id flag, automatically set
            // --outfmt=json.
            // This will help cope with potential issue arising when the queried
            // taxon has big data and cannot be fitted into a string (which is the corresponding
            // CSV and TSV output representation).
            // An example of such taxa is Escherichia. Before fixing this issue, when lauching
            // xgt search -ki g__Escherichia
            // we would get: Error: response too big for into_string
            search_args.set_outfmt("json".to_string());
        } else {
            search_args.set_outfmt(args.get_one::<String>("outfmt").unwrap().to_string());
        }

        search_args.set_disable_certificate_verification(args.get_flag("insecure"));

        search_args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli;
    use crate::utils::{OutputFormat, SearchField};
    use std::ffi::OsString;

    #[test]
    fn test_add_needle() {
        let mut search_args = SearchArgs::new();
        search_args.add_needle("test_needle");
        assert_eq!(search_args.get_needles(), &vec!["test_needle".to_string()]);
    }

    #[test]
    fn test_set_search_field() {
        let mut search_args = SearchArgs::new();
        search_args.set_search_field("gtdb");
        assert_eq!(search_args.get_search_field(), SearchField::GtdbTax);
    }

    #[test]
    fn test_set_matching_mode() {
        let mut search_args = SearchArgs::new();
        search_args.set_matching_mode(true);
        assert!(search_args.is_whole_words_matching());
    }

    #[test]
    fn test_set_id() {
        let mut search_args = SearchArgs::new();
        search_args.set_id(true);
        assert!(search_args.is_only_print_ids());
    }

    #[test]
    fn test_set_count() {
        let mut search_args = SearchArgs::new();
        search_args.set_count(true);
        assert!(search_args.is_only_num_entries());
    }

    #[test]
    fn test_set_is_representative_species_only() {
        let mut search_args = SearchArgs::new();
        search_args.set_is_representative_species_only(true);
        assert!(search_args.is_representative_species_only());
    }

    #[test]
    fn test_set_is_type_species_only() {
        let mut search_args = SearchArgs::new();
        search_args.set_is_type_species_only(true);
        assert!(search_args.is_type_species_only());
    }

    #[test]
    fn test_set_disable_certificate_verification() {
        let mut search_args = SearchArgs::new();
        search_args.set_disable_certificate_verification(true);
        assert!(search_args.disable_certificate_verification());
    }

    #[test]
    fn test_set_output() {
        let mut search_args = SearchArgs::new();
        search_args.set_output(Some("output.txt".to_string()));
        assert_eq!(search_args.get_output(), Some("output.txt".to_string()));
    }

    #[test]
    fn test_set_outfmt() {
        let mut search_args = SearchArgs::new();
        search_args.set_outfmt("json".to_string());
        assert_eq!(search_args.get_outfmt(), OutputFormat::Json);
    }

    #[test]
    fn test_from_arg_matches_with_name() {
        let matches = cli::app::build_app().get_matches_from(vec![
            OsString::new(),
            OsString::from("search"),
            OsString::from("test_name"),
            OsString::from("--id"),
            OsString::from("-w"),
            OsString::from("--count"),
            OsString::from("--rep"),
            OsString::from("--type"),
            OsString::from("--out"),
            OsString::from("output.txt"),
            OsString::from("--outfmt"),
            OsString::from("json"),
            OsString::from("--field"),
            OsString::from("gtdb"),
            OsString::from("--insecure"),
        ]);

        let search_args = cli::search::SearchArgs::from_arg_matches(
            matches.subcommand_matches("search").unwrap(),
        );

        assert_eq!(search_args.get_needles(), &vec!["test_name".to_string()]);
        assert_eq!(search_args.get_search_field(), SearchField::GtdbTax);
        assert!(search_args.is_whole_words_matching());
        assert!(search_args.is_only_print_ids());
        assert!(search_args.is_only_num_entries());
        assert!(search_args.is_representative_species_only());
        assert!(search_args.is_type_species_only());
        assert_eq!(search_args.get_output(), Some("output.txt".to_string()));
        assert_eq!(search_args.get_outfmt(), OutputFormat::Json);
        assert!(search_args.disable_certificate_verification());
    }
}
