use anyhow::Result;
use serde::{Deserialize, Serialize};

use std::fmt::Display;
use std::fs::OpenOptions;

use std::io::{self, Write};
use std::sync::Arc;

use regex::Regex;

/// Search field as provided by GTDB API
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub enum SearchField {
    // Search all fields
    #[default]
    All,

    // Search accession field
    NcbiId,

    // Search NCBI organism name field
    NcbiOrg,

    // Search GTDB taxonomy field
    GtdbTax,

    // Search NCBI taxonomy field
    NcbiTax,
}

/// Checks if the taxonomy string follows the correct format.
pub fn is_valid_taxonomy(taxonomy_str: &str) -> bool {
    let re = Regex::new(r"^(d__[^;]+)?(; p__[^;]+)?(; c__[^;]+)?(; o__[^;]+)?(; f__[^;]+)?(; g__[^;]+)?(; s__[^;]*)$").unwrap();
    re.is_match(taxonomy_str)
}

impl From<String> for SearchField {
    fn from(value: String) -> Self {
        if value == "acc" {
            SearchField::NcbiId
        } else if value == "org" {
            SearchField::NcbiOrg
        } else if value == "gtdb" {
            SearchField::GtdbTax
        } else if value == "ncbi" {
            SearchField::NcbiTax
        } else {
            SearchField::All
        }
    }
}

impl Display for SearchField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NcbiId => write!(f, "ncbi_id"),
            Self::All => write!(f, "all"),
            Self::GtdbTax => write!(f, "gtdb_tax"),
            Self::NcbiTax => write!(f, "ncbi_tax"),
            Self::NcbiOrg => write!(f, "ncbi_org"),
        }
    }
}

/// Search API possibles output format
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Csv,
    Json,
    Tsv,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Csv => write!(f, "csv"),
            Self::Json => write!(f, "json"),
            Self::Tsv => write!(f, "tsv"),
        }
    }
}

impl From<String> for OutputFormat {
    fn from(value: String) -> Self {
        if value == "tsv" {
            Self::Tsv
        } else if value == "json" {
            Self::Json
        } else {
            Self::Csv
        }
    }
}

/// Write `buffer` to `output` which can either be stdout or a file name.
pub fn write_to_output(buffer: &[u8], output: Option<String>) -> Result<()> {
    let mut writer: Box<dyn Write> = match output {
        Some(path) => Box::new(OpenOptions::new().append(true).create(true).open(path)?),
        None => Box::new(io::stdout()),
    };

    writer.write_all(buffer)?;
    writer.flush()?;

    Ok(())
}

/// Select agent request based on SSL peer verification activation
pub fn get_agent(disable_certificate_verification: bool) -> anyhow::Result<ureq::Agent> {
    match disable_certificate_verification {
        true => {
            let tls_connector = Arc::new(
                native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .build()?,
            );
            Ok(ureq::AgentBuilder::new()
                .tls_connector(tls_connector)
                .build())
        }
        false => Ok(ureq::AgentBuilder::new().build()),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct GtdbStatus {
    #[serde(alias = "timeMs")]
    time_ms: f32,
    online: bool,
}

pub fn is_gtdb_db_online(disable_certificate_verification: bool) -> Result<bool> {
    let agent = get_agent(disable_certificate_verification)?;
    let request_url = "https://gtdb-api.ecogenomic.org/status/db";
    let response = agent.get(request_url).call().map_err(|e| match e {
        ureq::Error::Status(code, _) => {
            anyhow::anyhow!("The server returned an unexpected status code ({})", code)
        }
        _ => {
            anyhow::anyhow!(
                "There was an error making the request or receiving the response:\n{}",
                e
            )
        }
    })?;

    let result: GtdbStatus = response.into_json()?;
    Ok(result.online)
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct GtdbApiVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

pub fn get_api_version(disable_certificate_verification: bool) -> Result<String> {
    let agent = get_agent(disable_certificate_verification)?;
    let request_url = "https://gtdb-api.ecogenomic.org/meta/version";
    let response = agent.get(request_url).call().map_err(|e| match e {
        ureq::Error::Status(code, _) => {
            anyhow::anyhow!("The server returned an unexpected status code ({})", code)
        }
        _ => {
            anyhow::anyhow!(
                "There was an error making the request or receiving the response:\n{}",
                e
            )
        }
    })?;
    let result: GtdbApiVersion = response.into_json()?;
    Ok(format!(
        "{}.{}.{}",
        result.major, result.minor, result.patch
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_write_to_output() {
        let s = "Hello, world!";

        // Test writing to a file
        let file_path = "test.txt";
        let output = Some(file_path.to_owned());
        write_to_output(s.as_bytes(), output).unwrap();
        let contents = std::fs::read_to_string(file_path).unwrap();
        assert_eq!(contents, s);

        std::fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_get_agent_with_certificate_verification() -> Result<()> {
        let agent = get_agent(false)?;
        let resp = agent.get("https://www.google.com").call();
        assert!(resp.is_ok());
        Ok(())
    }

    #[test]
    fn test_get_agent_without_certificate_verification() -> Result<()> {
        let agent = get_agent(true)?;
        let resp = agent.get("https://self-signed.badssl.com/").call();
        assert!(resp.is_ok());
        Ok(())
    }

    #[test]
    fn test_get_agent_invalid_url_with_certificate_verification() -> Result<()> {
        let agent = get_agent(false)?;
        let resp = agent.get("https://invalid-url").call();
        assert!(resp.is_err());
        Ok(())
    }

    #[test]
    fn test_get_agent_invalid_url_without_certificate_verification() -> Result<()> {
        let agent = get_agent(true)?;
        let resp = agent.get("https://invalid-url").call();
        assert!(resp.is_err());
        Ok(())
    }

    #[test]
    fn test_search_field_from_string() {
        assert_eq!(SearchField::from("acc".to_string()), SearchField::NcbiId);
        assert_eq!(SearchField::from("org".to_string()), SearchField::NcbiOrg);
        assert_eq!(SearchField::from("gtdb".to_string()), SearchField::GtdbTax);
        assert_eq!(SearchField::from("ncbi".to_string()), SearchField::NcbiTax);
        assert_eq!(SearchField::from("unknown".to_string()), SearchField::All);
    }

    #[test]
    fn test_search_field_display() {
        assert_eq!(SearchField::NcbiId.to_string(), "ncbi_id");
        assert_eq!(SearchField::All.to_string(), "all");
        assert_eq!(SearchField::GtdbTax.to_string(), "gtdb_tax");
        assert_eq!(SearchField::NcbiTax.to_string(), "ncbi_tax");
        assert_eq!(SearchField::NcbiOrg.to_string(), "ncbi_org");
    }

    #[test]
    fn test_output_format_from_string() {
        assert_eq!(OutputFormat::from("csv".to_string()), OutputFormat::Csv);
        assert_eq!(OutputFormat::from("json".to_string()), OutputFormat::Json);
        assert_eq!(OutputFormat::from("tsv".to_string()), OutputFormat::Tsv);
        assert_eq!(OutputFormat::from("unknown".to_string()), OutputFormat::Csv);
        // Default to Csv
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Csv.to_string(), "csv");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Tsv.to_string(), "tsv");
    }
}
