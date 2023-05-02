use crate::api::GenomeRequestType;
use anyhow::{bail, Result};
use clap::ArgMatches;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchArgs {
    pub(crate) needle: Vec<String>,
    pub(crate) level: String,
    pub(crate) id: bool,
    pub(crate) partial: bool,
    pub(crate) count: bool,
    pub(crate) raw: bool,
    pub(crate) rep: bool,
    pub(crate) type_material: bool,
    pub(crate) out: Option<String>,
}

impl SearchArgs {
    pub fn get_needle(&self) -> Vec<String> {
        self.needle.clone()
    }

    pub fn get_level(&self) -> String {
        self.level.clone()
    }

    pub fn get_gid(&self) -> bool {
        self.id
    }

    pub fn get_partial(&self) -> bool {
        self.partial
    }

    pub fn get_count(&self) -> bool {
        self.count
    }

    pub fn get_raw(&self) -> bool {
        self.raw
    }

    pub fn get_type_material(&self) -> bool {
        self.type_material
    }

    pub fn get_rep(&self) -> bool {
        self.rep
    }

    pub fn get_out(&self) -> Option<String> {
        self.out.clone()
    }

    pub fn from_arg_matches(args: &ArgMatches) -> Self {
        if args.contains_id("file") {
            let file = File::open(args.get_one::<String>("file").unwrap())
                .expect("file should be well-formatted");

            let needle = BufReader::new(file)
                .lines()
                .map(|l| l.expect("Cannot parse line"))
                .collect();

            SearchArgs {
                needle,
                level: args.get_one::<String>("level").unwrap().to_string(),
                id: args.get_flag("id"),
                partial: args.get_flag("partial"),
                count: args.get_flag("count"),
                raw: args.get_flag("raw"),
                rep: args.get_flag("rep"),
                type_material: args.get_flag("type"),
                out: if args.contains_id("out") {
                    args.get_one::<String>("out").cloned()
                } else {
                    None
                },
            }
        } else {
            SearchArgs {
                needle: vec![args.get_one::<String>("name").unwrap().to_string()],
                level: args.get_one::<String>("level").unwrap().to_string(),
                id: args.get_flag("id"),
                partial: args.get_flag("partial"),
                count: args.get_flag("count"),
                raw: args.get_flag("raw"),
                rep: args.get_flag("rep"),
                type_material: args.get_flag("type"),
                out: if args.contains_id("out") {
                    args.get_one::<String>("out").cloned()
                } else {
                    None
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenomeArgs {
    pub(crate) accession: Vec<String>,
    pub(crate) request_type: GenomeRequestType,
    pub(crate) raw: bool,
    pub(crate) output: Option<String>,
}

impl GenomeArgs {
    pub fn get_accession(&self) -> Vec<String> {
        self.accession.clone()
    }

    pub fn get_request_type(&self) -> GenomeRequestType {
        self.request_type
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

        if arg_matches.get_flag("history") {
            GenomeArgs {
                accession,
                request_type: GenomeRequestType::TaxonHistory,
                raw: arg_matches.get_flag("raw"),
                output: if arg_matches.contains_id("out") {
                    arg_matches.get_one::<String>("out").cloned()
                } else {
                    None
                },
            }
        } else if arg_matches.get_flag("metadata") {
            GenomeArgs {
                accession,
                request_type: GenomeRequestType::Metadata,
                raw: arg_matches.get_flag("raw"),
                output: if arg_matches.contains_id("out") {
                    arg_matches.get_one::<String>("out").cloned()
                } else {
                    None
                },
            }
        } else {
            GenomeArgs {
                accession,
                request_type: GenomeRequestType::Card,
                raw: arg_matches.get_flag("raw"),
                output: if arg_matches.contains_id("out") {
                    arg_matches.get_one::<String>("out").cloned()
                } else {
                    None
                },
            }
        }
    }
}

pub fn bool_as_string(b: bool) -> String {
    if b {
        String::from("true")
    } else {
        String::from("false")
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
    if response.status().is_server_error() {
        bail!("server error (code status {})", response.status().as_str());
    } else if !response.status().is_success() {
        bail!(
            "something wrong happened (code status {})",
            response.status().as_str()
        );
    }
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
            request_type: GenomeRequestType::Card,
            raw: false,
            output: None,
        };

        assert_eq!(genome_args.get_accession(), vec!["NC_000001.11"]);
    }

    #[test]
    fn test_get_request_type() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            request_type: GenomeRequestType::Card,
            raw: false,
            output: None,
        };

        assert_eq!(genome_args.get_request_type(), GenomeRequestType::Card);
    }

    #[test]
    fn test_get_raw() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            request_type: GenomeRequestType::Card,
            raw: true,
            output: None,
        };

        assert!(genome_args.get_raw());
    }

    #[test]
    fn test_get_output() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            request_type: GenomeRequestType::Card,
            raw: false,
            output: Some(String::from("output4.txt")),
        };

        assert_eq!(genome_args.get_output(), Some(String::from("output4.txt")));
    }

    #[test]
    fn test_bool_as_string() {
        assert_eq!(bool_as_string(true), "true");
        assert_eq!(bool_as_string(false), "false");
    }

    #[test]
    fn test_get_needle() {
        let args = SearchArgs {
            needle: vec!["needle".to_string()],
            level: "level".to_string(),
            id: false,
            partial: false,
            count: false,
            raw: false,
            rep: false,
            type_material: false,
            out: Some(String::from("test1")),
        };
        assert_eq!(args.get_needle(), vec!["needle".to_string()]);
        assert_eq!(args.get_level(), "level".to_string());
        assert!(!args.get_gid());
        assert!(!args.get_partial());
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
}
