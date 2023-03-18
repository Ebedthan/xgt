use crate::api::GenomeRequestType;
use anyhow::Result;
use clap::ArgMatches;
use serde::{Deserialize, Deserializer};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
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
            let file = File::open(args.get_one::<PathBuf>("file").unwrap())
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
                needle: if args.get_one::<String>("name").is_some() {
                    vec![args.get_one::<String>("name").unwrap().to_string()]
                } else {
                    eprintln!("error: search string should be supplied");
                    std::process::exit(1);
                },
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
            let file = File::open(arg_matches.get_one::<PathBuf>("file").unwrap())
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

pub fn parse_gtdb<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_accession() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            request_type: GenomeRequestType::Card,
            raw: false,
            output: Some(String::from("output.txt")),
        };

        assert_eq!(genome_args.get_accession(), vec!["NC_000001.11"]);
    }

    #[test]
    fn test_get_request_type() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            request_type: GenomeRequestType::Card,
            raw: false,
            output: Some(String::from("output.txt")),
        };

        assert_eq!(genome_args.get_request_type(), GenomeRequestType::Card);
    }

    #[test]
    fn test_get_raw() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            request_type: GenomeRequestType::Card,
            raw: true,
            output: Some(String::from("output.txt")),
        };

        assert!(genome_args.get_raw());
    }

    #[test]
    fn test_get_output() {
        let genome_args = GenomeArgs {
            accession: vec![String::from("NC_000001.11")],
            request_type: GenomeRequestType::Card,
            raw: false,
            output: Some(String::from("output.txt")),
        };

        assert_eq!(genome_args.get_output(), Some(String::from("output.txt")));
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
            out: Some(String::from("test")),
        };
        assert_eq!(args.get_needle(), vec!["needle".to_string()]);
        assert_eq!(args.get_level(), "level".to_string());
        assert!(!args.get_gid());
        assert!(!args.get_partial());
        assert!(!args.get_count());
        assert!(!args.get_raw());
        assert!(!args.get_rep());
        assert!(!args.get_type_material());
        assert_eq!(args.get_out(), Some(String::from("test")));
    }
}
