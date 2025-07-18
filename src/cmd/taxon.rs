use anyhow::{ensure, Result};
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

// The Taxon command actually repeats a certain logic:
// - Create a request URL from a GtdbApiRequest
// - Call utils::fetch_data
// - Deserialize the response with into_json()
// - Serialize with serde_json::to_string_pretty
// - Write using utils::write_to_output
// To avoid code duplication, we can create a helper function that encapsulates this logic.

// Helper function to fetch and write JSON
fn fetch_and_write_json<T: for<'de> Deserialize<'de> + Serialize>(
    agent: &Agent,
    request: GtdbApiRequest,
    err_msg: String,
    out_path: Option<String>,
) -> Result<T> {
    let url = request.to_url();
    let response = utils::fetch_data(agent, &url, err_msg)?;
    let data: T = response.into_json()?;
    let json = serde_json::to_string_pretty(&data)?;
    utils::write_to_output(json.as_bytes(), out_path)?;
    Ok(data)
}

pub fn get_taxon_name(args: TaxonArgs) -> Result<()> {
    if let Some(name) = args.name {
        let agent = utils::get_agent(args.insecure)?;
        let request = GtdbApiRequest::Taxon {
            name: name.clone(),
            kind: TaxonEndPoint::Name,
            limit: None,
            is_reps_only: None,
        };

        fetch_and_write_json::<TaxonResult>(
            &agent,
            request,
            format!("Taxon {} not found", name),
            args.out,
        )?;
    }

    Ok(())
}

pub fn get_taxon_genomes(args: TaxonArgs) -> Result<()> {
    if let Some(name) = args.name {
        let agent = utils::get_agent(args.insecure)?;
        let request = GtdbApiRequest::Taxon {
            name: name.clone(),
            kind: TaxonEndPoint::Genomes,
            limit: None,
            is_reps_only: Some(args.reps),
        };
        let data = fetch_and_write_json::<TaxonGenomes>(
            &agent,
            request,
            format!("No match found for {}", name),
            args.out,
        )?;

        ensure!(!data.data.is_empty(), "No data found for {}", name);
    }

    Ok(())
}

pub fn search_taxon(args: TaxonArgs) -> Result<()> {
    if let Some(name) = args.name.as_deref() {
        let agent = utils::get_agent(args.insecure)?;

        let kind = if args.all {
            TaxonEndPoint::SearchAll
        } else {
            TaxonEndPoint::Search
        };
        let request = GtdbApiRequest::Taxon {
            name: name.into(),
            kind,
            limit: None,
            is_reps_only: None,
        };

        let mut data = fetch_and_write_json::<TaxonSearchResult>(
            &agent,
            request,
            format!("No match found for {}", name),
            None,
        )?;

        if args.word {
            data.matches.retain(|x| x == name);
        }
        ensure!(!data.matches.is_empty(), "No match found for {}", name);

        let json = serde_json::to_string_pretty(&data)?;
        utils::write_to_output(json.as_bytes(), args.out)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::fs;

    #[test]
    fn test_get_taxon_name_with_output() -> Result<()> {
        let args = TaxonArgs {
            name: Some("g__Escherichia".to_string()),
            out: Some("output.json".to_string()),
            word: false,
            search: false,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
        };
        let actual_output = args.out.clone();
        get_taxon_name(args)?;

        let expected_output = fs::read_to_string("output.json")?;
        let expected_taxon_data: TaxonResult = serde_json::from_str(&expected_output)?;

        let actual_output = fs::read_to_string(actual_output.unwrap())?;
        let actual_taxon_data: TaxonResult = serde_json::from_str(&actual_output)?;

        assert_eq!(expected_taxon_data, actual_taxon_data);

        // Clean up the output file
        fs::remove_file("output.json")?;

        Ok(())
    }

    #[test]
    fn test_get_taxon_name_without_output() -> Result<()> {
        let args = TaxonArgs {
            name: Some("g__Escherichia".to_string()),
            out: None,
            word: false,
            search: false,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
        };

        get_taxon_name(args)?;

        Ok(())
    }

    #[test]
    fn test_get_taxon_name_not_found() -> Result<()> {
        let args = TaxonArgs {
            name: Some("UnknownTaxonName".to_string()),
            out: None,
            word: true,
            search: false,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
        };
        let result = get_taxon_name(args);
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
        let args = TaxonArgs {
            name: Some("UnknownTaxonName".to_string()),
            out: None,
            word: true,
            search: false,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
        };
        let result = get_taxon_name(args);
        assert!(result.is_err());
    }

    #[test]
    fn search_taxon_should_return_error_for_nonexistent_taxon() {
        let args = TaxonArgs {
            name: Some("nonexistent_taxon".to_string()),
            out: None,
            word: false,
            search: true,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
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
            name: Some("g__Aminobacter".to_string()),
            out: None,
            word: false,
            search: true,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
        };
        let result = search_taxon(args);
        assert!(result.is_ok());
    }

    #[test]
    fn taxon_should_print_raw_output_to_stdout() {
        let args = TaxonArgs {
            name: Some("g__Aminobacter".to_string()),
            out: None,
            word: false,
            search: false,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
        };
        let result = search_taxon(args);
        assert!(result.is_ok());
    }

    #[test]
    fn search_taxon_should_write_pretty_output_to_file() {
        let args = TaxonArgs {
            name: Some("g__Aminobacter".to_string()),
            out: Some("test_search.json".to_string()),
            word: false,
            search: true,
            all: false,
            genomes: false,
            reps: false,
            insecure: true,
            file: None,
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
            name: Some("g__Aminobacter".to_string()),
            out: Some("output.json".to_string()),
            word: false,
            search: false,
            all: false,
            genomes: true,
            reps: false,
            insecure: true,
            file: None,
        };

        let actual_output = args.out.clone();

        get_taxon_genomes(args)?;

        let expected_output = fs::read_to_string("output.json")?;
        let expected_taxon_data: TaxonGenomes = serde_json::from_str(&expected_output)?;

        let actual_output = fs::read_to_string(actual_output.unwrap())?;
        let actual_taxon_data: TaxonGenomes = serde_json::from_str(&actual_output)?;

        assert_eq!(expected_taxon_data, actual_taxon_data);

        // Clean up the output file
        fs::remove_file("output.json")?;

        Ok(())
    }
}
