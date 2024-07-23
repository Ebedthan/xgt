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

    /// Check if tool was ran into partial search mode
    pub fn is_partial_search(&self) -> bool {
        self.is_partial_search
    }

    /// Setter for search mode attribute
    pub fn set_search_mode(&mut self, is_partial_search: bool) {
        self.is_partial_search = is_partial_search;
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
