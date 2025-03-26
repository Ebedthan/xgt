use anyhow::Result;

use std::fmt::Display;
use std::fs::OpenOptions;

use std::io::{self, Write};
use std::sync::Arc;

/// Search field as provided by GTDB API
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub enum SearchField {
    // Search all fields
    #[default]
    All,

    // Search accession field
    Acc,

    // Search NCBI organism name field
    Org,

    // Search GTDB taxonomy field
    Gtdb,

    // Search NCBI taxonomy field
    Ncbi,
}

/// Check if a SearchField is a taxonomy field (either GTDB taxonomy or NCBI taxonomy).
pub fn is_taxonomy_field(search_field: &SearchField) -> bool {
    search_field == &SearchField::Gtdb || search_field == &SearchField::Ncbi
}

impl From<String> for SearchField {
    fn from(value: String) -> Self {
        if value == "acc" {
            SearchField::Acc
        } else if value == "org" {
            SearchField::Org
        } else if value == "gtdb" {
            SearchField::Gtdb
        } else if value == "ncbi" {
            SearchField::Ncbi
        } else {
            SearchField::All
        }
    }
}

impl Display for SearchField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Acc => write!(f, "ncbi_id"),
            Self::All => write!(f, "all"),
            Self::Gtdb => write!(f, "gtdb_tax"),
            Self::Ncbi => write!(f, "ncbi_tax"),
            Self::Org => write!(f, "ncbi_org"),
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
        assert_eq!(SearchField::from("acc".to_string()), SearchField::Acc);
        assert_eq!(SearchField::from("org".to_string()), SearchField::Org);
        assert_eq!(SearchField::from("gtdb".to_string()), SearchField::Gtdb);
        assert_eq!(SearchField::from("ncbi".to_string()), SearchField::Ncbi);
        assert_eq!(SearchField::from("unknown".to_string()), SearchField::All);
    }

    #[test]
    fn test_search_field_display() {
        assert_eq!(SearchField::Acc.to_string(), "ncbi_id");
        assert_eq!(SearchField::All.to_string(), "all");
        assert_eq!(SearchField::Gtdb.to_string(), "gtdb_tax");
        assert_eq!(SearchField::Ncbi.to_string(), "ncbi_tax");
        assert_eq!(SearchField::Org.to_string(), "ncbi_org");
    }

    #[test]
    fn test_is_taxonomy_field() {
        assert!(is_taxonomy_field(&SearchField::Gtdb));
        assert!(is_taxonomy_field(&SearchField::Ncbi));
        assert!(!is_taxonomy_field(&SearchField::Acc));
        assert!(!is_taxonomy_field(&SearchField::Org));
        assert!(!is_taxonomy_field(&SearchField::All));
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
