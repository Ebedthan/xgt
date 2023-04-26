use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::api::TaxonApi;

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

pub fn get_taxon_name(args: TaxonArgs) -> Result<()> {
    // format the request
    let taxon_api: Vec<TaxonApi> = args
        .get_name()
        .iter()
        .map(|x| TaxonApi::from(x.to_string()))
        .collect();
    let raw = args.get_raw();

    if let Some(filename) = args.get_output() {
        let path = Path::new(&filename);
        if path.exists() {
            bail!("file {} should not already exist", path.display());
        }
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
        };
        let result = get_taxon_name(taxon_args);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Taxon UnknownTaxonName not found"));
        Ok(())
    }
}
