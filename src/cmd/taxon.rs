use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::api::{GtdbApiRequest, TaxonEndPoint};
use crate::cli::TaxonArgs;
use crate::utils;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Taxon {
    taxon: String,
    total: Option<f32>,
    #[serde(alias = "nDescChildren")]
    n_desc_children: Option<String>,
    #[serde(alias = "isGenome")]
    is_genome: Option<bool>,
    #[serde(alias = "isRep")]
    is_rep: Option<bool>,
    #[serde(alias = "typeMaterial")]
    type_material: Option<String>,
    #[serde(alias = "bergeysUrl")]
    bergeys_url: Option<String>,
    #[serde(alias = "seqcodeUrl")]
    seq_code_url: Option<String>,
    #[serde(alias = "lpsnUrl")]
    lpsn_url: Option<String>,
    #[serde(alias = "ncbiTaxId")]
    ncbi_tax_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct TaxonResult {
    data: Vec<Taxon>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaxonSearchResult {
    matches: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct TaxonGenomes {
    data: Vec<String>,
}

// Struct for error 400 occuring from wrongly formatted
// taxon name
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaxonGenomesError {
    detail: String,
}

impl TaxonSearchResult {
    fn filter(&mut self, pattern: String) {
        self.matches.retain(|x| x == &pattern);
    }
}

pub fn get_taxon_name(args: TaxonArgs) -> Result<()> {
    let agent: Agent = utils::get_agent(args.insecure)?;

    if let Some(name) = args.name {
        let taxon = GtdbApiRequest::Taxon {
            name: name.clone(),
            kind: TaxonEndPoint::Name,
            limit: None,
            is_reps_only: None,
        };
        let request_url = taxon.to_url();
        let response = match agent.get(&request_url).call() {
            Ok(r) => r,
            Err(ureq::Error::Status(400, _)) => bail!("Taxon {} not found", name),
            Err(ureq::Error::Status(code, _)) => bail!("Unexpected status code: {}", code),
            Err(_) => bail!("Error making the request or receiving the response."),
        };

        let taxon_data: TaxonResult = response.into_json()?;
        let taxon_string = serde_json::to_string_pretty(&taxon_data)?;
        utils::write_to_output(taxon_string.as_bytes(), args.out.clone())?;
    }

    Ok(())
}

pub fn search_taxon(args: TaxonArgs) -> Result<()> {
    let is_whole_words_matching = args.word;
    let agent: Agent = utils::get_agent(args.insecure)?;

    if let Some(name) = args.name {
        let request_url = if args.all {
            let search = GtdbApiRequest::Taxon {
                name: name.clone(),
                kind: TaxonEndPoint::SearchAll,
                limit: None,
                is_reps_only: None,
            };
            search.to_url()
        } else {
            let search = GtdbApiRequest::Taxon {
                name: name.clone(),
                kind: TaxonEndPoint::Search,
                limit: None,
                is_reps_only: None,
            };
            search.to_url()
        };

        let response = match agent.get(&request_url).call() {
            Ok(r) => r,
            Err(ureq::Error::Status(400, _)) => bail!("No match found for {}", name),
            Err(ureq::Error::Status(code, _)) => bail!("Unexpected status code: {}", code),
            Err(_) => bail!("Error making the request or receiving the response."),
        };

        let mut taxon_data: TaxonSearchResult = response.into_json()?;
        if is_whole_words_matching {
            taxon_data.filter(name.to_string());
        }

        ensure!(
            !taxon_data.matches.is_empty(),
            "No match found for {}",
            name
        );

        let taxon_string = serde_json::to_string_pretty(&taxon_data)?;

        utils::write_to_output(taxon_string.as_bytes(), args.out.clone())?;
    }

    Ok(())
}

pub fn get_taxon_genomes(args: TaxonArgs) -> Result<()> {
    let agent: Agent = utils::get_agent(args.insecure)?;

    if let Some(name) = args.name {
        let search = GtdbApiRequest::Taxon {
            name: name.clone(),
            kind: TaxonEndPoint::Genomes,
            limit: None,
            is_reps_only: Some(args.reps),
        };
        let request_url = search.to_url();

        let response = match agent.get(&request_url).call() {
            Ok(r) => r,
            Err(ureq::Error::Status(400, _)) => bail!("No match found for {}", name),
            Err(ureq::Error::Status(code, _)) => bail!("Unexpected status code: {}", code),
            Err(_) => bail!("Error making the request or receiving the response."),
        };

        let taxon_data: TaxonGenomes = response.into_json()?;

        ensure!(!taxon_data.data.is_empty(), "No data found for {}", name);

        let taxon_string = serde_json::to_string_pretty(&taxon_data)?;

        utils::write_to_output(taxon_string.as_bytes(), args.out.clone())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests { /*
            use super::*;
            use mockito::Server;
            use std::fs;

            #[test]
            fn test_get_taxon_name_with_output() -> Result<()> {
                let args = TaxonArgs {
                    name: vec!["g__Escherichia".to_string()],
                    output: Some("output.json".to_string()),
                    is_whole_words_matching: false,
                    search: false,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };

                get_taxon_name(args.clone())?;

                let expected_output = fs::read_to_string("output.json")?;
                let expected_taxon_data: TaxonResult = serde_json::from_str(&expected_output)?;

                let actual_output = args.get_output().unwrap();
                let actual_output = fs::read_to_string(actual_output)?;
                let actual_taxon_data: TaxonResult = serde_json::from_str(&actual_output)?;

                assert_eq!(expected_taxon_data, actual_taxon_data);

                // Clean up the output file
                fs::remove_file("output.json")?;

                Ok(())
            }

            #[test]
            fn test_get_taxon_name_without_output() -> Result<()> {
                let args = TaxonArgs {
                    name: vec!["g__Escherichia".to_string()],
                    output: None,
                    is_whole_words_matching: false,
                    search: false,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };

                get_taxon_name(args)?;

                Ok(())
            }

            #[test]
            fn test_get_taxon_name_not_found() -> Result<()> {
                let taxon_args = TaxonArgs {
                    name: vec!["UnknownTaxonName".to_string()],
                    output: None,
                    is_whole_words_matching: true,
                    search: false,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };
                let result = get_taxon_name(taxon_args);
                assert!(result.is_err());
                let err = result.unwrap_err().to_string();
                assert!(err.contains("Taxon UnknownTaxonName not found"));
                Ok(())
            }

            #[test]
            fn test_get_taxon_name_server_error() {
                let mut s = Server::new();
                let url = s.url();
                s.mock("GET", url.as_str()).with_status(450).create();
                let taxon_args = TaxonArgs {
                    name: vec!["UnknownTaxonName".to_string()],
                    output: None,
                    is_whole_words_matching: true,
                    search: false,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };
                let result = get_taxon_name(taxon_args);
                assert!(result.is_err());
            }

            #[test]
            fn test_taxon_search_result_filter() {
                let mut taxon_search_result = TaxonSearchResult {
                    matches: vec!["abc".to_string(), "abcd".to_string()],
                };
                taxon_search_result.filter("abc".to_string());
                assert_eq!(taxon_search_result.matches, vec!["abc".to_string()]);
            }

            #[test]
            fn test_filter() {
                let mut result = TaxonSearchResult {
                    matches: vec!["dog".to_string(), "cat".to_string(), "rat".to_string()],
                };
                result.filter("cat".to_string());
                assert_eq!(result.matches, vec!["cat".to_string()]);
            }

            #[test]
            fn test_filter_no_match() {
                let mut result = TaxonSearchResult {
                    matches: vec!["dog".to_string(), "cat".to_string(), "rat".to_string()],
                };
                result.filter("bird".to_string());
                let v: Vec<String> = Vec::new();
                assert_eq!(result.matches, v);
            }

            #[test]
            fn search_taxon_should_return_error_for_nonexistent_taxon() {
                let args = TaxonArgs {
                    name: vec!["nonexistent_taxon".to_string()],
                    is_whole_words_matching: false,
                    output: None,
                    search: true,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };
                let result = search_taxon(args);
                assert!(result.is_err());
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "No match found for nonexistent_taxon".to_string()
                );
            }

            #[test]
            fn search_taxon_should_print_raw_output_to_stdout() {
                let args = TaxonArgs {
                    name: vec!["g__Aminobacter".to_string()],
                    is_whole_words_matching: false,
                    output: None,
                    search: true,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };
                let result = search_taxon(args);
                assert!(result.is_ok());
            }

            #[test]
            fn taxon_should_print_raw_output_to_stdout() {
                let args = TaxonArgs {
                    name: vec!["g__Aminobacter".to_string()],
                    is_whole_words_matching: false,
                    output: None,
                    search: false,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };
                let result = search_taxon(args);
                assert!(result.is_ok());
            }

            #[test]
            fn search_taxon_should_write_pretty_output_to_file() {
                let args = TaxonArgs {
                    name: vec!["g__Aminobacter".to_string()],
                    is_whole_words_matching: false,
                    output: Some("test_search.json".to_string()),
                    search: true,
                    search_all: false,
                    genomes: false,
                    reps_only: false,
                    disable_certificate_verification: true,
                };
                let result = search_taxon(args);
                assert!(result.is_ok());

                // Check that the output file was created and contains the taxon name
                let file_contents = std::fs::read_to_string("test_search.json").unwrap();
                assert!(file_contents.contains("g__Aminobacter"));
                std::fs::remove_file("test_search.json").unwrap();
            }

            #[test]
            fn test_get_genomes_with_output() -> Result<()> {
                let args = TaxonArgs {
                    name: vec!["g__Escherichia".to_string()],
                    output: Some("output.json".to_string()),
                    is_whole_words_matching: false,
                    search: false,
                    search_all: false,
                    genomes: true,
                    reps_only: false,
                    disable_certificate_verification: true,
                };

                let actual_output = args.get_output().unwrap();

                get_taxon_genomes(args)?;

                let expected_output = fs::read_to_string("output.json")?;
                let expected_taxon_data: TaxonGenomes = serde_json::from_str(&expected_output)?;

                let actual_output = fs::read_to_string(actual_output)?;
                let actual_taxon_data: TaxonGenomes = serde_json::from_str(&actual_output)?;

                assert_eq!(expected_taxon_data, actual_taxon_data);

                // Clean up the output file
                fs::remove_file("output.json")?;

                Ok(())
            }*/
}
