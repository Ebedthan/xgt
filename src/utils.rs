use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use ureq::{Agent, Response};

use std::fmt::Display;
use std::fs::{File, OpenOptions};

use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;

use regex::Regex;

use crate::cli::{GenomeArgs, SearchArgs, TaxonArgs};

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

// Generic helper to load accession or name from file or stdin
pub trait InputSource {
    fn file(&self) -> Option<&String>;
    fn fallback(&self) -> Option<&String>;
}

impl InputSource for GenomeArgs {
    fn file(&self) -> Option<&String> {
        self.file.as_ref()
    }

    fn fallback(&self) -> Option<&String> {
        self.accession.as_ref()
    }
}

impl InputSource for TaxonArgs {
    fn file(&self) -> Option<&String> {
        self.file.as_ref()
    }

    fn fallback(&self) -> Option<&String> {
        self.name.as_ref()
    }
}

impl InputSource for SearchArgs {
    fn file(&self) -> Option<&String> {
        self.file.as_ref()
    }

    fn fallback(&self) -> Option<&String> {
        self.query.as_ref()
    }
}

pub fn load_input<T: InputSource>(args: &T, err_msg: String) -> Result<Vec<String>> {
    if let Some(file_path) = args.file() {
        let file =
            File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;
        BufReader::new(file)
            .lines()
            .collect::<std::io::Result<Vec<String>>>()
            .map_err(Into::into)
    } else if let Some(value) = args.fallback() {
        Ok(vec![value.clone()])
    } else {
        Err(anyhow!(err_msg))
    }
}

pub fn fetch_data(agent: &Agent, url: &str, err_msg: String) -> Result<Response, anyhow::Error> {
    match agent.get(url).call() {
        Ok(r) => Ok(r),
        Err(ureq::Error::Status(400, _)) => bail!(err_msg),
        Err(ureq::Error::Status(code, _)) => bail!("Unexpected status code: {}", code),
        Err(_) => bail!("Error making the request or receiving the response."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use mockito::Server;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use ureq::Agent;

    #[test]
    fn test_fetch_data_ok() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/success")
            .with_status(200)
            .with_body("{\"status\": \"ok\"}")
            .create();

        let agent = Agent::new();
        let url = format!("{}/success", server.url());
        let response = fetch_data(&agent, &url, "Failed to fetch".to_string()).unwrap();

        let text = response.into_string().unwrap();
        assert!(text.contains("ok"));
    }

    #[test]
    fn test_fetch_data_bad_request() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/bad")
            .with_status(400)
            .with_body("Bad Request")
            .create();

        let agent = Agent::new();
        let url = format!("{}/bad", server.url());
        let err = fetch_data(&agent, &url, "Bad Request occurred".to_string()).unwrap_err();

        assert_eq!(err.to_string(), "Bad Request occurred");
    }

    #[test]
    fn test_fetch_data_unexpected_status() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/teapot")
            .with_status(418)
            .with_body("I'm a teapot")
            .create();

        let agent = Agent::new();
        let url = format!("{}/teapot", server.url());
        let err = fetch_data(&agent, &url, "Error!".to_string()).unwrap_err();

        assert_eq!(err.to_string(), "Unexpected status code: 418");
    }

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

    struct TestArgs {
        file: Option<String>,
        fallback: Option<String>,
    }

    impl InputSource for TestArgs {
        fn file(&self) -> Option<&String> {
            self.file.as_ref()
        }

        fn fallback(&self) -> Option<&String> {
            self.fallback.as_ref()
        }
    }

    #[test]
    fn test_load_input_from_fallback() {
        let args = TestArgs {
            file: None,
            fallback: Some("ABC123".to_string()),
        };

        let result = load_input(&args, "Missing input".to_string()).unwrap();
        assert_eq!(result, vec!["ABC123".to_string()]);
    }

    #[test]
    fn test_load_input_from_file() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "line1\nline2").unwrap();

        let path = tmpfile.path().to_str().unwrap().to_string();
        let args = TestArgs {
            file: Some(path),
            fallback: None,
        };

        let result = load_input(&args, "Missing input".to_string()).unwrap();
        assert_eq!(result, vec!["line1".to_string(), "line2".to_string()]);
    }

    #[test]
    fn test_load_input_error() {
        let args = TestArgs {
            file: None,
            fallback: None,
        };

        let result = load_input(&args, "Missing input".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Missing input".to_string());
    }
}
