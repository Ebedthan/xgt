use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::api::taxon_api::TaxonAPI;

use super::utils::{self, TaxonArgs};

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

impl TaxonSearchResult {
    fn filter(&mut self, pattern: String) {
        self.matches.retain(|x| x == &pattern);
    }
}

pub fn get_taxon_name(args: TaxonArgs) -> Result<()> {
    // format the request
    let taxon_api: Vec<TaxonAPI> = args
        .get_name()
        .iter()
        .map(|x| TaxonAPI::from(x.to_string()))
        .collect();
    let raw = args.get_raw();

    if let Some(filename) = args.get_output() {
        let path = Path::new(&filename);
        ensure!(
            !path.exists(),
            "file {} should not already exist",
            path.display()
        );
    }

    for name in taxon_api {
        let request_url = name.get_name_request();

        let response = reqwest::blocking::get(request_url)
            .with_context(|| "Failed to get response from GTDB API".to_string())?;

        if response.status().is_client_error() {
            bail!("Taxon {} not found", name.get_name());
        }

        utils::check_status(&response)?;

        let taxon_data: TaxonResult = response.json().with_context(|| {
            "Failed to convert request response to genome metadata structure".to_string()
        })?;

        match raw {
            true => {
                let taxon_string = serde_json::to_string(&taxon_data).with_context(|| {
                    "Failed to convert taxon structure to json string".to_string()
                })?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(taxon_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{taxon_string}")?;
                }
            }
            false => {
                let taxon_string =
                    serde_json::to_string_pretty(&taxon_data).with_context(|| {
                        "Failed to convert genome card structure to json string".to_string()
                    })?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(taxon_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{taxon_string}")?;
                }
            }
        };
    }

    Ok(())
}

pub fn search_taxon(args: TaxonArgs) -> Result<()> {
    let taxon_api: Vec<TaxonAPI> = args
        .get_name()
        .iter()
        .map(|x| TaxonAPI::from(x.to_string()))
        .collect();
    let raw = args.get_raw();
    let partial = args.get_partial();

    if let Some(filename) = args.get_output() {
        let path = Path::new(&filename);
        ensure!(
            !path.exists(),
            "file {} should not already exist",
            path.display()
        );
    }

    for search in taxon_api {
        let request_url = search.get_search_request();

        let response = reqwest::blocking::get(request_url)
            .with_context(|| "Failed to get response from GTDB API".to_string())?;

        utils::check_status(&response)?;

        let mut taxon_data: TaxonSearchResult = response.json().with_context(|| {
            "Failed to convert request response to genome metadata structure".to_string()
        })?;

        if !partial {
            taxon_data.filter(search.get_name());
        }

        ensure!(
            !taxon_data.matches.is_empty(),
            "No match found for {}",
            search.get_name()
        );

        match raw {
            true => {
                let taxon_string = serde_json::to_string(&taxon_data).with_context(|| {
                    "Failed to convert taxon structure to json string".to_string()
                })?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(taxon_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{taxon_string}")?;
                }
            }
            false => {
                let taxon_string =
                    serde_json::to_string_pretty(&taxon_data).with_context(|| {
                        "Failed to convert genome card structure to json string".to_string()
                    })?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(taxon_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{taxon_string}")?;
                }
            }
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_taxon_name_with_output() -> Result<()> {
        let args = TaxonArgs {
            name: vec!["g__Escherichia".to_string()],
            raw: false,
            output: Some("output.json".to_string()),
            partial: false,
            search: false,
        };

        get_taxon_name(args.clone())?;

        let expected_output = fs::read_to_string("output.json")
            .with_context(|| "Failed to read output file".to_string())?;
        let expected_taxon_data: TaxonResult = serde_json::from_str(&expected_output)
            .with_context(|| "Failed to convert expected output to TaxonResult".to_string())?;

        let actual_output = args.get_output().unwrap();
        let actual_output = fs::read_to_string(actual_output)
            .with_context(|| "Failed to read actual output file".to_string())?;
        let actual_taxon_data: TaxonResult = serde_json::from_str(&actual_output)
            .with_context(|| "Failed to convert actual output to TaxonResult".to_string())?;

        assert_eq!(expected_taxon_data, actual_taxon_data);

        // Clean up the output file
        fs::remove_file("output.json")
            .with_context(|| "Failed to delete output file".to_string())?;

        Ok(())
    }

    #[test]
    fn test_get_taxon_name_without_output() -> Result<()> {
        let args = TaxonArgs {
            name: vec!["g__Escherichia".to_string()],
            raw: false,
            output: None,
            partial: false,
            search: false,
        };

        get_taxon_name(args)?;

        Ok(())
    }

    #[test]
    fn test_get_taxon_name_not_found() -> Result<()> {
        let taxon_args = TaxonArgs {
            name: vec!["UnknownTaxonName".to_string()],
            raw: false,
            output: None,
            partial: true,
            search: false,
        };
        let result = get_taxon_name(taxon_args);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Taxon UnknownTaxonName not found"));
        Ok(())
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
    fn search_taxon_should_return_error_for_nonexistent_file() {
        let args = TaxonArgs {
            name: vec!["g__Aminobacter".to_string()],
            raw: true,
            partial: false,
            output: Some("test/acc.txt".to_string()),
            search: true,
        };
        let result = search_taxon(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "file test/acc.txt should not already exist".to_string()
        );
    }

    #[test]
    fn search_taxon_should_return_error_for_nonexistent_taxon() {
        let args = TaxonArgs {
            name: vec!["nonexistent_taxon".to_string()],
            raw: true,
            partial: false,
            output: None,
            search: true,
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
            raw: true,
            partial: false,
            output: None,
            search: true,
        };
        let result = search_taxon(args);
        assert!(result.is_ok());
    }

    #[test]
    fn search_taxon_should_write_pretty_output_to_file() {
        let args = TaxonArgs {
            name: vec!["g__Aminobacter".to_string()],
            raw: false,
            partial: false,
            output: Some("test_search.json".to_string()),
            search: true,
        };
        let result = search_taxon(args);
        assert!(result.is_ok());
        // Check that the output file was created and contains the taxon name
        let file_contents = std::fs::read_to_string("test_search.json").unwrap();
        assert!(file_contents.contains("g__Aminobacter"));
        std::fs::remove_file("test_search.json").unwrap();
    }
}
