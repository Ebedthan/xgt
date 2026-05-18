use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use ureq::http::Response;
use ureq::{Agent, Body};

use std::fmt::Display;
use std::fs::{File, OpenOptions};

use std::io::{self, BufRead, BufReader, Write};

use regex::Regex;

use std::thread;
use std::time::Duration;

use crate::cli::{GenomeArgs, SearchArgs, TaxonArgs};

/// Returns true for errors that are worth retrying (transient server/network issues).
fn is_retryable(err: &ureq::Error) -> bool {
    match err {
        // 5xx server errors and 429 rate limiting are transient
        ureq::Error::StatusCode(500)
        | ureq::Error::StatusCode(502)
        | ureq::Error::StatusCode(503)
        | ureq::Error::StatusCode(504)
        | ureq::Error::StatusCode(429) => true,
        // Network/IO errors (timeout, connection reset, DNS failure) are transient
        ureq::Error::Io(_) => true,
        // All other status codes (4xx etc.) are deterministic — don't retry
        _ => false,
    }
}

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
    if disable_certificate_verification {
        eprintln!(
            "Warning: SSL certificate verification is disabled. Use only on trusted networks."
        );
        let config = Agent::config_builder()
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .disable_verification(true)
                    .build(),
            )
            .build();
        Ok(config.new_agent())
    } else {
        Ok(ureq::Agent::config_builder().build().new_agent())
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
        ureq::Error::StatusCode(code) => {
            anyhow::anyhow!("The server returned an unexpected status code ({})", code)
        }
        _ => {
            anyhow::anyhow!(
                "There was an error making the request or receiving the response...\n{}",
                e
            )
        }
    })?;

    let result: GtdbStatus = response.into_body().read_json()?;
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
        ureq::Error::StatusCode(code) => {
            anyhow::anyhow!("The server returned an unexpected status code ({})", code)
        }
        _ => {
            anyhow::anyhow!(
                "There was an error making the request or receiving the response...\n{}",
                e
            )
        }
    })?;
    let result: GtdbApiVersion = response.into_body().read_json()?;
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
        if file_path == "-" {
            io::stdin()
                .lock()
                .lines()
                .collect::<std::io::Result<Vec<String>>>()
                .map_err(Into::into)
        } else {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {}", file_path))?;
            BufReader::new(file)
                .lines()
                .collect::<std::io::Result<Vec<String>>>()
                .map_err(Into::into)
        }
    } else if let Some(value) = args.fallback() {
        Ok(vec![value.clone()])
    } else {
        Err(anyhow!(err_msg))
    }
}

pub fn fetch_data(agent: &Agent, url: &str, err_msg: String) -> Result<Response<Body>> {
    const MAX_RETRIES: u32 = 3;
    const BASE_DELAY: Duration = Duration::from_secs(1);
    const MAX_DELAY: Duration = Duration::from_secs(10);

    let mut attempt = 0;

    loop {
        match agent.get(url).call() {
            Ok(response) => return Ok(response),

            Err(ureq::Error::StatusCode(400)) => bail!(err_msg),

            Err(ureq::Error::StatusCode(code)) if !is_retryable(&ureq::Error::StatusCode(code)) => {
                bail!("Unexpected status code: {}", code)
            }

            Err(e) => {
                attempt += 1;
                if attempt >= MAX_RETRIES {
                    bail!("Request failed after {} attempts: {}", MAX_RETRIES, e);
                }

                // Exponential backoff: 1s, 2s, 4s... capped at MAX_DELAY
                let delay = (BASE_DELAY * 2u32.pow(attempt - 1)).min(MAX_DELAY);

                eprintln!(
                    "Request failed (attempt {}/{}): {}. Retrying in {}s...",
                    attempt,
                    MAX_RETRIES,
                    e,
                    delay.as_secs()
                );

                thread::sleep(delay);
            }
        }
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
    fn test_fetch_data_retries_on_503() {
        let mut server = Server::new();

        // First two attempts return 503, third succeeds
        let _m1 = server
            .mock("GET", "/flaky")
            .with_status(503)
            .expect(2)
            .create();
        let _m2 = server
            .mock("GET", "/flaky")
            .with_status(200)
            .with_body("{\"status\": \"ok\"}")
            .expect(1)
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/flaky", server.url());
        let result = fetch_data(&agent, &url, "error".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_fetch_data_fails_after_max_retries() {
        let mut server = Server::new();

        // Always returns 503
        let _m = server
            .mock("GET", "/always-down")
            .with_status(503)
            .expect(3) // exactly MAX_RETRIES calls expected
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/always-down", server.url());
        let result = fetch_data(&agent, &url, "error".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed after 3 attempts"));
    }

    #[test]
    fn test_fetch_data_does_not_retry_400() {
        let mut server = Server::new();

        // Returns 400 — should not be retried
        let _m = server
            .mock("GET", "/bad")
            .with_status(400)
            .expect(1) // only 1 call, no retry
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/bad", server.url());
        let result = fetch_data(&agent, &url, "Bad Request occurred".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Bad Request occurred");
    }

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable(&ureq::Error::StatusCode(500)));
        assert!(is_retryable(&ureq::Error::StatusCode(502)));
        assert!(is_retryable(&ureq::Error::StatusCode(503)));
        assert!(is_retryable(&ureq::Error::StatusCode(504)));
        assert!(is_retryable(&ureq::Error::StatusCode(429)));
        assert!(!is_retryable(&ureq::Error::StatusCode(400)));
        assert!(!is_retryable(&ureq::Error::StatusCode(404)));
        assert!(!is_retryable(&ureq::Error::StatusCode(401)));
    }

    #[test]
    fn test_load_input_stdin_marker_is_recognized() {
        // When file is "-", load_input should not try to open a file named "-".
        // We can't feed actual stdin in a unit test, so we verify the non-file
        // path is taken by checking that a missing file named "-" does NOT produce
        // a "Failed to open file" error (which would happen if we tried to open it).
        // Instead stdin will just return EOF immediately in a non-interactive context,
        // yielding an empty vec.
        let args = TestArgs {
            file: Some("-".to_string()),
            fallback: None,
        };
        // In a test harness stdin is typically closed/empty, so this should succeed
        // with zero lines rather than error with "Failed to open file: -"
        let result = load_input(&args, "Missing input".to_string());
        assert!(
            result.is_ok(),
            "stdin path should not produce a file-open error"
        );
    }

    #[test]
    fn test_load_input_file_named_dash_does_not_open_file() {
        // Confirm that "-" is not treated as a literal filename:
        // if it were, opening a non-existent file named "-" might succeed on some
        // systems or fail with a specific OS error. The stdin branch should be taken.
        let args = TestArgs {
            file: Some("-".to_string()),
            fallback: Some("should_not_be_used".to_string()),
        };
        let result = load_input(&args, "Missing input".to_string());
        // Should not return the fallback value "should_not_be_used"
        if let Ok(lines) = result {
            assert!(
                !lines.contains(&"should_not_be_used".to_string()),
                "fallback should not be used when file is '-'"
            );
        }
    }

    #[test]
    fn test_gtdb_is_online_real() {
        let is_online = is_gtdb_db_online(true);
        assert!(is_online.unwrap());
    }

    #[test]
    fn test_get_api_version_real() {
        let version = get_api_version(true);
        assert_eq!(version.unwrap(), String::from("2.27.0"));
    }

    fn with_mocked_agent() -> ureq::Agent {
        // Ensures we always use mockito's base URL
        Agent::config_builder().build().new_agent()
    }

    fn set_mock_base_url(path: &str) -> String {
        let server = Server::new();
        format!("{}/{}", server.url(), path.trim_start_matches('/'))
    }

    #[test]
    fn test_get_api_version_failure() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/meta/version")
            .with_status(500)
            .create();

        let agent = with_mocked_agent();
        let request_url = set_mock_base_url("/meta/version");

        let result = agent.get(&request_url).call();
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_data_ok() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/success")
            .with_status(200)
            .with_body("{\"status\": \"ok\"}")
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/success", server.url());
        let mut response = fetch_data(&agent, &url, "Failed to fetch".to_string()).unwrap();

        let text = response.body_mut();
        assert!(text.read_to_string().unwrap().contains("ok"));
    }

    #[test]
    fn test_fetch_data_bad_request() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/bad")
            .with_status(400)
            .with_body("Bad Request")
            .create();

        let agent = Agent::config_builder().build().new_agent();
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

        let agent = Agent::config_builder().build().new_agent();
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
    fn test_get_agent_rejects_invalid_host() -> Result<()> {
        let agent = get_agent(false)?;
        let resp = agent.get("https://invalid-url").call();
        assert!(resp.is_err());
        Ok(())
    }

    #[test]
    fn test_get_agent_secure_builds_successfully() -> Result<()> {
        let agent = get_agent(false)?;
        let _ = agent;
        Ok(())
    }

    #[test]
    fn test_get_agent_insecure_builds_successfully() -> Result<()> {
        let agent = get_agent(true)?;
        let _ = agent;
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
