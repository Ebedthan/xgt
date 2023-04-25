use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crate::api::TaxonApi;

use super::utils::TaxonArgs;

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
            writeln!(
                io::stderr(),
                "error: file {} should not already exist",
                path.display()
            )?;
            std::process::exit(1);
        }
    }

    for name in taxon_api {
        let request_url = name.get_name_request();

        let response = reqwest::blocking::get(request_url)
            .with_context(|| "Failed to get response from GTDB API".to_string())?;

        if response.status().is_client_error() {
            writeln!(
                io::stderr(),
                "{}",
                format!("Taxon {} not found", name.get_name())
            )?;
            std::process::exit(1);
        }

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
