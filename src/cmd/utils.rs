use anyhow::{bail, Context, Result};
use clap::ArgMatches;

use std::fs::OpenOptions;

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchArgs {
    pub(crate) needle: Vec<String>,
    pub(crate) level: String,
    pub(crate) id: bool,
    pub(crate) count: bool,
    pub(crate) raw: bool,
    pub(crate) rep: bool,
    pub(crate) type_material: bool,
    pub(crate) out: Option<String>,
}

impl Default for SearchArgs {
    fn default() -> Self {
        SearchArgs {
            needle: Vec::new(),
            level: "genus".to_string(),
            id: false,
            count: false,
            raw: false,
            rep: false,
            type_material: false,
            out: None,
        }
    }
}

impl SearchArgs {
    pub fn get_needle(&self) -> Vec<String> {
        self.needle.clone()
    }

    pub fn set_needle(&mut self, v: Vec<String>) {
        self.needle.extend(v.into_iter());
    }

    pub fn get_level(&self) -> String {
        self.level.clone()
    }

    pub(crate) fn set_level(&mut self, s: String) {
        self.level = s;
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

    pub fn new() -> Self {
        SearchArgs::default()
    }

    pub fn from_arg_matches(args: &ArgMatches) -> Self {
        let mut search_args = SearchArgs::new();
        let mut needles: Vec<String> = Vec::new();

        if args.contains_id("file") {
            let file = File::open(args.get_one::<String>("file").unwrap())
                .expect("file should be well-formatted");

            needles.extend(
                BufReader::new(file)
                    .lines()
                    .map(|l| l.expect("Cannot parse line")),
            );
        } else {
            needles.extend(vec![args.get_one::<String>("name").unwrap().to_string()].into_iter());
        }

        search_args.set_needle(needles);

        if args.contains_id("level") {
            search_args.set_level(args.get_one::<String>("level").unwrap().to_string());
        }

        if args.get_flag("id") {
            search_args.set_id(true);
        }

        if args.get_flag("count") {
            search_args.set_count(true);
        }

        if args.get_flag("raw") {
            search_args.set_raw(true);
        }

        if args.get_flag("rep") {
            search_args.set_rep(true);
        }

        if args.get_flag("type") {
            search_args.set_type_material(true);
        }

        if args.contains_id("out") {
            search_args.set_out(args.get_one::<String>("out").cloned());
        }

        search_args
    }
}

#[derive(Debug, Clone)]
pub struct GenomeArgs {
    pub(crate) accession: Vec<String>,
    pub(crate) raw: bool,
    pub(crate) output: Option<String>,
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

    pub fn from_arg_matches(arg_matches: &ArgMatches) -> Self {
        let mut accession = Vec::new();

        if arg_matches.contains_id("file") {
            let file = File::open(arg_matches.get_one::<String>("file").unwrap())
                .expect("file should be well-formatted");

            accession = BufReader::new(file)
                .lines()
                .map(|l| l.expect("Cannot parse line"))
                .collect();
        } else {
            accession.push(
                arg_matches
                    .get_one::<String>("accession")
                    .unwrap()
                    .to_string(),
            );
        }

        GenomeArgs {
            accession,
            raw: arg_matches.get_flag("raw"),
            output: if arg_matches.contains_id("out") {
                arg_matches.get_one::<String>("out").cloned()
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaxonArgs {
    pub(crate) name: Vec<String>,
    pub(crate) raw: bool,
    pub(crate) output: Option<String>,
    pub(crate) partial: bool,
    pub(crate) search: bool,
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

    pub fn is_search(&self) -> bool {
        self.search
    }

    pub fn from_arg_matches(arg_matches: &ArgMatches) -> Self {
        let mut names = Vec::new();

        if arg_matches.contains_id("file") {
            let file = File::open(arg_matches.get_one::<String>("file").unwrap())
                .expect("file should be well-formatted");

            names = BufReader::new(file)
                .lines()
                .map(|l| l.expect("Cannot parse line"))
                .collect();
        } else {
            names.push(arg_matches.get_one::<String>("name").unwrap().to_string());
        }

        TaxonArgs {
            name: names,
            raw: arg_matches.get_flag("raw"),
            output: if arg_matches.contains_id("out") {
                arg_matches.get_one::<String>("out").cloned()
            } else {
                None
            },
            partial: arg_matches.get_flag("partial"),
            search: arg_matches.get_flag("search"),
        }
    }
}

pub fn check_status(response: &reqwest::blocking::Response) -> Result<()> {
    let binding = response.status();
    let status_str = binding.as_str();
    if !response.status().is_success() {
        bail!("something wrong happened (code status {})", status_str);
    }
    Ok(())
}

pub fn write_to_output(s: String, output: Option<String>) -> Result<()> {
    let mut writer: Box<dyn Write> = match output {
        Some(path) => Box::new(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .with_context(|| "Failed to create file".to_string())?,
        ),
        None => Box::new(io::stdout()),
    };

    writer
        .write_all(s.as_bytes())
        .with_context(|| "Failed to write".to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn test_get_accession() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            raw: false,
            output: None,
        };

        assert_eq!(genome_args.get_accession(), vec!["NC_000001.11"]);
    }

    #[test]
    fn test_get_raw() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            raw: true,
            output: None,
        };

        assert!(genome_args.get_raw());
    }

    #[test]
    fn test_get_output() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            raw: false,
            output: Some(String::from("output4.txt")),
        };

        assert_eq!(genome_args.get_output(), Some(String::from("output4.txt")));
    }

    #[test]
    fn test_get_needle() {
        let args = SearchArgs {
            needle: vec!["needle".to_string()],
            level: "level".to_string(),
            id: false,
            count: false,
            raw: false,
            rep: false,
            type_material: false,
            out: Some(String::from("test1")),
        };
        assert_eq!(args.get_needle(), vec!["needle".to_string()]);
        assert_eq!(args.get_level(), "level".to_string());
        assert!(!args.get_gid());
        assert!(!args.get_count());
        assert!(!args.get_raw());
        assert!(!args.get_rep());
        assert!(!args.get_type_material());
        assert_eq!(args.get_out(), Some(String::from("test1")));
    }

    #[test]
    fn test_check_status_server_error() {
        let mut s = Server::new();
        let url = s.url();
        s.mock("GET", url.as_str()).with_status(500).create();
        let response = reqwest::blocking::get(&url).unwrap();
        assert!(check_status(&response).is_err());
    }

    #[test]
    fn test_check_status_something_wrong() {
        let mut s = Server::new();
        let url = s.url();
        s.mock("GET", url.as_str()).with_status(300).create();
        let response = reqwest::blocking::get(&url).unwrap();
        assert!(check_status(&response).is_err());
    }

    #[test]
    fn test_get_name() {
        let args = TaxonArgs {
            name: vec!["name1".to_string(), "name2".to_string()],
            raw: false,
            output: None,
            partial: false,
            search: false,
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
        };

        assert_eq!(args.is_search(), true);
    }

    #[test]
    fn test_write_to_output() {
        let s = "Hello, world!".to_owned();

        // Test writing to a file
        let file_path = "test.txt";
        let output = Some(file_path.to_owned());
        write_to_output(s.clone(), output).unwrap();
        let contents = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(contents, s);

        std::fs::remove_file(file_path).unwrap();
    }
}
