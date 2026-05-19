use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::api::{GtdbApiRequest, TaxonEndPoint};
use crate::cli::TaxonArgs;
use crate::utils;

use crate::utils::ToFlatRow;

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

impl ToFlatRow for TaxonResult {
    fn csv_header(sep: &str) -> String {
        format!(
            "taxon{sep}total{sep}n_desc_children{sep}is_genome{sep}is_rep\
             {sep}type_material{sep}bergeys_url{sep}seq_code_url{sep}lpsn_url{sep}ncbi_tax_id"
        )
    }

    fn to_flat_row(&self, sep: &str) -> String {
        let mut lines = vec![Self::csv_header(sep)];

        for t in &self.data {
            lines.push(format!(
                "{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}",
                t.taxon,
                t.total.map(|v| v.to_string()).unwrap_or_default(),
                t.n_desc_children.as_deref().unwrap_or(""),
                t.is_genome.map(|v| v.to_string()).unwrap_or_default(),
                t.is_rep.map(|v| v.to_string()).unwrap_or_default(),
                t.type_material.as_deref().unwrap_or(""),
                t.bergeys_url.as_deref().unwrap_or(""),
                t.seq_code_url.as_deref().unwrap_or(""),
                t.lpsn_url.as_deref().unwrap_or(""),
                t.ncbi_tax_id.map(|v| v.to_string()).unwrap_or_default(),
            ));
        }

        lines.join("\n") + "\n"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaxonSearchResult {
    matches: Vec<String>,
}

impl ToFlatRow for TaxonSearchResult {
    fn csv_header(_sep: &str) -> String {
        "taxon".to_string()
    }

    fn to_flat_row(&self, _sep: &str) -> String {
        let mut lines = vec![Self::csv_header(_sep)];
        lines.extend(self.matches.iter().cloned());
        lines.join("\n") + "\n"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct TaxonGenomes {
    data: Vec<String>,
}

impl ToFlatRow for TaxonGenomes {
    fn csv_header(_sep: &str) -> String {
        "accession".to_string()
    }

    fn to_flat_row(&self, _sep: &str) -> String {
        let mut lines = vec![Self::csv_header(_sep)];
        lines.extend(self.data.iter().cloned());
        lines.join("\n") + "\n"
    }
}

// The Taxon command actually repeats a certain logic:
// - Create a request URL from a GtdbApiRequest
// - Call utils::fetch_data
// - Deserialize the response with into_json()
// - Serialize with serde_json::to_string_pretty
// - Write using utils::write_to_output
// To avoid code duplication, we can create a helper function that encapsulates this logic.

// Pure fetch: deserializes and returns data, no writing
fn fetch_json<T: for<'de> Deserialize<'de> + Serialize + ToFlatRow>(
    agent: &Agent,
    request: GtdbApiRequest,
    err_msg: String,
) -> Result<T> {
    let url = request.to_url();
    let response = utils::fetch_data(agent, &url, err_msg)?;
    let data: T = response.into_body().read_json()?;
    Ok(data)
}

// Fetch and immediately write: used by callers with no post-processing
fn fetch_and_write_json<T: for<'de> Deserialize<'de> + Serialize + ToFlatRow>(
    agent: &Agent,
    request: GtdbApiRequest,
    err_msg: String,
    outfmt: &utils::OutputFormat,
    key: &str, // query/name used for split filename
    dest: &utils::OutputDestination,
) -> Result<T> {
    let data: T = fetch_json(agent, request, err_msg)?;
    let sep = if *outfmt == utils::OutputFormat::Tsv {
        "\t"
    } else {
        ","
    };
    let output = match outfmt {
        utils::OutputFormat::Json => serde_json::to_string_pretty(&data)?,
        _ => data.to_flat_row(sep),
    };
    utils::write_to_output(output.as_bytes(), dest.resolve(key), false)?;
    Ok(data)
}
pub fn get_taxon_name(args: &TaxonArgs) -> Result<()> {
    if let Some(name) = &args.name {
        let agent = utils::get_agent(args.insecure)?;
        let outfmt = utils::OutputFormat::from(args.outfmt.clone());
        let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);
        let request = GtdbApiRequest::Taxon {
            name: name.clone(),
            kind: TaxonEndPoint::Name,
            limit: None,
            is_reps_only: None,
        };
        fetch_and_write_json::<TaxonResult>(
            &agent,
            request,
            format!("Taxon '{}' was not found in GTDB...", name),
            &outfmt,
            name,
            &dest,
        )?;
    }
    Ok(())
}

pub fn get_taxon_genomes(args: &TaxonArgs) -> Result<()> {
    if let Some(name) = &args.name {
        let agent = utils::get_agent(args.insecure)?;
        let outfmt = utils::OutputFormat::from(args.outfmt.clone());
        let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);
        let request = GtdbApiRequest::Taxon {
            name: name.clone(),
            kind: TaxonEndPoint::Genomes,
            limit: None,
            is_reps_only: Some(args.reps),
        };
        let data = fetch_and_write_json::<TaxonGenomes>(
            &agent,
            request,
            format!("No genomes found for taxon '{}'...", name),
            &outfmt,
            name,
            &dest,
        )?;
        ensure!(
            !data.data.is_empty(),
            "Taxon '{}' exists but has no associated genomes.",
            name
        );
    }
    Ok(())
}

pub fn search_taxon(args: &TaxonArgs) -> Result<()> {
    if let Some(name) = args.name.as_deref() {
        let agent = utils::get_agent(args.insecure)?;
        let outfmt = utils::OutputFormat::from(args.outfmt.clone());
        let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);

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

        let mut data: TaxonSearchResult = fetch_json(
            &agent,
            request,
            format!("No taxa matching '{}' found in GTDB.", name),
        )?;

        if args.word {
            data.matches.retain(|x| x == name);
        }
        ensure!(
            !data.matches.is_empty(),
            "No taxa matching '{}' found in GTDB{}.",
            name,
            if args.word {
                " (exact match with --word)"
            } else {
                ""
            }
        );

        let sep = if outfmt == utils::OutputFormat::Tsv {
            "\t"
        } else {
            ","
        };
        let output = match outfmt {
            utils::OutputFormat::Json => serde_json::to_string_pretty(&data)?,
            _ => data.to_flat_row(sep),
        };
        utils::write_to_output(output.as_bytes(), dest.resolve(name), false)?;
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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };
        let actual_output = args.out.clone();
        get_taxon_name(&args)?;

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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };

        get_taxon_name(&args)?;

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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };
        let result = get_taxon_name(&args);
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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };
        let result = get_taxon_name(&args);
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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };
        let result = search_taxon(&args);
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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };
        let result = search_taxon(&args);
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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };
        let result = search_taxon(&args);
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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };
        let result = search_taxon(&args);
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
            outfmt: "json".to_string(),
            split: false,
            split_dir: None,
        };

        let actual_output = args.out.clone();

        get_taxon_genomes(&args)?;

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
