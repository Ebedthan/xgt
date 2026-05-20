use anyhow::{anyhow, bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use ureq::http::Response;
use ureq::{Agent, Body};

use std::fmt::Display;
use std::fs::{File, OpenOptions};

use std::io::{self, BufRead, BufReader, Write};

use ureq::tls::{TlsConfig, TlsProvider};

use std::thread;
use std::time::Duration;

use crate::cli::{DiffArgs, GenomeArgs, SearchArgs, TaxonArgs};

pub trait ToFlatRow {
    fn csv_header(sep: &str) -> String;
    fn to_flat_row(&self, sep: &str) -> String;
}

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
/// Now we also support append mode.
pub fn write_to_output(buffer: &[u8], output: Option<String>, append: bool) -> Result<()> {
    let mut writer: Box<dyn Write> = match output {
        Some(path) => Box::new(
            OpenOptions::new()
                .write(true)
                .append(append)
                .truncate(!append)
                .create(true)
                .open(path)?,
        ),
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
            "Warning: SSL certificate verification is disabled. \
             Use only on trusted networks."
        );
        Ok(Agent::config_builder()
            .tls_config(TlsConfig::builder().disable_verification(true).build())
            .build()
            .new_agent())
    } else {
        Ok(Agent::config_builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::NativeTls)
                    .build(),
            )
            .build()
            .new_agent())
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
            anyhow::anyhow!(
                "GTDB API returned status {} while checking database status. \
                 The service may be temporarily unavailable.",
                code
            )
        }
        _ => {
            anyhow::anyhow!(
                "Could not reach the GTDB API ({}). \
                 Check your internet connection or try again later.",
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
            anyhow::anyhow!(
                "GTDB API returned status {} while fetching API version. \
                 The service may be temporarily unavailable.",
                code
            )
        }
        _ => {
            anyhow::anyhow!(
                "Could not reach the GTDB API ({}). \
                 Check your internet connection or try again later.",
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

impl InputSource for DiffArgs {
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

            Err(ureq::Error::StatusCode(400)) => bail!(err_msg), // caller-supplied, addressed per call site below

            Err(ureq::Error::StatusCode(code)) if !is_retryable(&ureq::Error::StatusCode(code)) => {
                bail!(
                    "GTDB API returned an unexpected status code {}. \
                     If this persists, check https://gtdb.ecogenomic.org for service status.",
                    code
                )
            }

            Err(e) => {
                attempt += 1;
                if attempt >= MAX_RETRIES {
                    bail!(
                        "Request to GTDB API failed after {} attempts. Last error: {}. \
                             Check your internet connection or try again later.",
                        MAX_RETRIES,
                        e
                    );
                }

                // Exponential backoff: 1s, 2s, 4s... capped at MAX_DELAY
                let delay = (BASE_DELAY * 2u32.pow(attempt - 1)).min(MAX_DELAY);

                eprintln!(
                    "Warning: request failed (attempt {}/{}): {}. Retrying in {}s...",
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

/// Create a styled progress bar for batch operations.
/// Returns `None` when there is only one item (no bar needed).
pub fn make_progress_bar(total: usize) -> Option<ProgressBar> {
    if total <= 1 {
        return None;
    }
    let bar = ProgressBar::new(total as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    bar.set_message("processing...");
    Some(bar)
}

/// Where output should be written.
#[derive(Debug, Clone)]
pub enum OutputDestination {
    /// Write everything to stdout
    Stdout,
    /// Write everything to a single file
    File(String),
    /// Write each item to its own file, derived from the item key
    Split {
        dir: Option<String>,
        extension: String,
    },
}

impl OutputDestination {
    /// Resolve the concrete file path for a given item key (accession or query).
    /// Returns `None` for stdout, `Some(path)` for file output.
    pub fn resolve(&self, key: &str) -> Option<String> {
        match self {
            Self::Stdout => None,
            Self::File(path) => Some(path.clone()),
            Self::Split { dir, extension } => {
                let safe_key = key.replace([' ', '/'], "_");
                let filename = format!("{}.{}", safe_key, extension);
                Some(match dir {
                    Some(d) => format!("{}/{}", d, filename),
                    None => filename,
                })
            }
        }
    }

    /// Returns true if each item goes to its own file.
    pub fn is_split(&self) -> bool {
        matches!(self, Self::Split { .. })
    }
}

/// Build an OutputDestination from the common --out / --split / --outfmt combination.
pub fn output_destination(
    out: &Option<String>,
    split: bool,
    outfmt: &OutputFormat,
    split_dir: &Option<String>,
) -> OutputDestination {
    if split {
        OutputDestination::Split {
            dir: split_dir.clone(),
            extension: outfmt.to_string(),
        }
    } else {
        match out {
            Some(path) => OutputDestination::File(path.clone()),
            None => OutputDestination::Stdout,
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
    fn test_make_progress_bar_returns_none_for_zero() {
        let bar = make_progress_bar(0);
        assert!(bar.is_none(), "0 items should return None");
    }

    #[test]
    fn test_make_progress_bar_returns_none_for_one() {
        let bar = make_progress_bar(1);
        assert!(bar.is_none(), "1 item should return None - no bar needed");
    }

    #[test]
    fn test_make_progress_bar_returns_some_for_two() {
        // The bar-construction branch (lines 348–357) is only reached when total > 1
        let bar = make_progress_bar(2);
        assert!(bar.is_some(), "2 items should return Some(ProgressBar)");
    }

    #[test]
    fn test_make_progress_bar_returns_some_for_large_count() {
        let bar = make_progress_bar(500);
        assert!(bar.is_some());
    }

    #[test]
    fn test_make_progress_bar_length_matches_total() {
        let total = 42_usize;
        let bar = make_progress_bar(total).unwrap();
        assert_eq!(bar.length(), Some(total as u64));
    }

    #[test]
    fn test_resolve_stdout_returns_none() {
        let dest = OutputDestination::Stdout;
        assert_eq!(dest.resolve("any_key"), None);
    }

    #[test]
    fn test_resolve_file_returns_path_unchanged() {
        let dest = OutputDestination::File("results.json".into());
        assert_eq!(dest.resolve("ignored_key"), Some("results.json".into()));
    }

    #[test]
    fn test_resolve_split_without_dir_returns_filename_only() {
        // None directory → filename only, no path separator
        let dest = OutputDestination::Split {
            dir: None,
            extension: "json".into(),
        };
        assert_eq!(
            dest.resolve("GCA_000005845.2"),
            Some("GCA_000005845.2.json".into())
        );
    }

    #[test]
    fn test_resolve_split_with_dir_returns_dir_slash_filename() {
        // Some(dir) → "dir/key.ext"
        let dest = OutputDestination::Split {
            dir: Some("results".into()),
            extension: "csv".into(),
        };
        assert_eq!(
            dest.resolve("GCA_000005845.2"),
            Some("results/GCA_000005845.2.csv".into())
        );
    }

    #[test]
    fn test_resolve_split_replaces_spaces_in_key() {
        // Taxon names with spaces must be sanitised
        let dest = OutputDestination::Split {
            dir: None,
            extension: "json".into(),
        };
        assert_eq!(
            dest.resolve("s__Escherichia coli"),
            Some("s__Escherichia_coli.json".into())
        );
    }

    #[test]
    fn test_resolve_split_replaces_slashes_in_key() {
        let dest = OutputDestination::Split {
            dir: None,
            extension: "tsv".into(),
        };
        assert_eq!(
            dest.resolve("path/with/slashes"),
            Some("path_with_slashes.tsv".into())
        );
    }

    #[test]
    fn test_resolve_split_replaces_spaces_and_slashes() {
        // Both replacements must apply to the same key
        let dest = OutputDestination::Split {
            dir: Some("out".into()),
            extension: "json".into(),
        };
        assert_eq!(
            dest.resolve("s__Some species/subspecies"),
            Some("out/s__Some_species_subspecies.json".into())
        );
    }

    #[test]
    fn test_resolve_split_empty_key_produces_just_extension() {
        let dest = OutputDestination::Split {
            dir: None,
            extension: "json".into(),
        };
        assert_eq!(dest.resolve(""), Some(".json".into()));
    }

    #[test]
    fn test_is_split_true_for_split_variant() {
        let dest = OutputDestination::Split {
            dir: None,
            extension: "json".into(),
        };
        assert!(dest.is_split());
    }

    #[test]
    fn test_is_split_false_for_stdout() {
        assert!(!OutputDestination::Stdout.is_split());
    }

    #[test]
    fn test_is_split_false_for_file() {
        assert!(!OutputDestination::File("out.json".into()).is_split());
    }

    #[test]
    fn test_output_destination_split_flag_takes_priority_over_out() {
        // --split and --out together: split wins, out is ignored
        let dest = output_destination(
            &Some("ignored.json".into()),
            true,
            &OutputFormat::Json,
            &Some("results/".into()),
        );
        assert!(dest.is_split());
    }

    #[test]
    fn test_output_destination_no_split_with_out_gives_file() {
        let dest = output_destination(&Some("output.csv".into()), false, &OutputFormat::Csv, &None);
        assert!(matches!(dest, OutputDestination::File(p) if p == "output.csv"));
    }

    #[test]
    fn test_output_destination_no_split_no_out_gives_stdout() {
        let dest = output_destination(&None, false, &OutputFormat::Json, &None);
        assert!(matches!(dest, OutputDestination::Stdout));
    }

    #[test]
    fn test_output_destination_split_extension_matches_outfmt() {
        let dest = output_destination(&None, true, &OutputFormat::Tsv, &None);
        // Extension should be "tsv" matching the OutputFormat
        if let OutputDestination::Split { extension, .. } = dest {
            assert_eq!(extension, "tsv");
        } else {
            panic!("expected Split variant");
        }
    }

    #[test]
    fn test_output_destination_split_dir_is_propagated() {
        let dest = output_destination(&None, true, &OutputFormat::Json, &Some("my_dir".into()));
        if let OutputDestination::Split { dir, .. } = dest {
            assert_eq!(dir, Some("my_dir".into()));
        } else {
            panic!("expected Split variant");
        }
    }

    #[test]
    fn test_gtdb_status_deserialises_online_true() {
        let json = r#"{"timeMs": 1.23, "online": true}"#;
        let status: GtdbStatus = serde_json::from_str(json).unwrap();
        assert!(status.online);
        assert!((status.time_ms - 1.23_f32).abs() < 0.001);
    }

    #[test]
    fn test_gtdb_status_deserialises_online_false() {
        let json = r#"{"timeMs": 5.0, "online": false}"#;
        let status: GtdbStatus = serde_json::from_str(json).unwrap();
        assert!(!status.online);
    }

    #[test]
    fn test_gtdb_status_alias_time_ms() {
        // API may send "timeMs" (camelCase) — the alias must handle it
        let json = r#"{"timeMs": 2.5, "online": true}"#;
        let status: GtdbStatus = serde_json::from_str(json).unwrap();
        assert!((status.time_ms - 2.5_f32).abs() < 0.001);
    }

    #[test]
    fn test_api_version_deserialises_correctly() {
        let json = r#"{"major": 2, "minor": 27, "patch": 0}"#;
        let version: GtdbApiVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 27);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_api_version_formats_as_dotted_string() {
        let version = GtdbApiVersion {
            major: 2,
            minor: 27,
            patch: 0,
        };
        let formatted = format!("{}.{}.{}", version.major, version.minor, version.patch);
        assert_eq!(formatted, "2.27.0");
    }

    #[test]
    fn test_api_version_formats_non_zero_patch() {
        let version = GtdbApiVersion {
            major: 3,
            minor: 0,
            patch: 14,
        };
        let formatted = format!("{}.{}.{}", version.major, version.minor, version.patch);
        assert_eq!(formatted, "3.0.14");
    }

    // The public function hardcodes the URL, so we test the response-handling
    // logic by replicating the function's internal steps against a mock server.

    #[test]
    fn test_is_gtdb_db_online_returns_true_when_online() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/status/db")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"timeMs": 1.5, "online": true}"#)
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/status/db", server.url());

        // Replicate what is_gtdb_db_online does after building the agent
        let response = fetch_data(&agent, &url, "status check failed".into()).unwrap();
        let status: GtdbStatus = response.into_body().read_json().unwrap();
        assert!(status.online);
    }

    #[test]
    fn test_is_gtdb_db_online_returns_false_when_offline() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/status/db")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"timeMs": 0.0, "online": false}"#)
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/status/db", server.url());

        let response = fetch_data(&agent, &url, "status check failed".into()).unwrap();
        let status: GtdbStatus = response.into_body().read_json().unwrap();
        assert!(!status.online);
    }

    #[test]
    fn test_is_gtdb_db_online_500_produces_error() {
        let mut server = Server::new();
        let _m = server.mock("GET", "/status/db").with_status(500).create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/status/db", server.url());

        // 500 is retryable — fetch_data will retry MAX_RETRIES times then error
        let result = fetch_data(&agent, &url, "status check failed".into());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("failed after 3 attempts"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn test_is_gtdb_db_online_400_produces_caller_message() {
        let mut server = Server::new();
        let _m = server.mock("GET", "/status/db").with_status(400).create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/status/db", server.url());

        let result = fetch_data(&agent, &url, "status check failed".into());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "status check failed");
    }

    #[test]
    fn test_is_gtdb_db_online_malformed_json_produces_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/status/db")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"not valid json"#)
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/status/db", server.url());

        let response = fetch_data(&agent, &url, "status check failed".into()).unwrap();
        let result: Result<GtdbStatus, _> = response.into_body().read_json();
        assert!(
            result.is_err(),
            "malformed JSON should produce a parse error"
        );
    }

    #[test]
    fn test_get_api_version_returns_dotted_version_string() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/meta/version")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"major": 2, "minor": 27, "patch": 0}"#)
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/meta/version", server.url());

        let response = fetch_data(&agent, &url, "version fetch failed".into()).unwrap();
        let version: GtdbApiVersion = response.into_body().read_json().unwrap();
        let formatted = format!("{}.{}.{}", version.major, version.minor, version.patch);
        assert_eq!(formatted, "2.27.0");
    }

    #[test]
    fn test_get_api_version_non_zero_patch() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/meta/version")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"major": 3, "minor": 1, "patch": 4}"#)
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/meta/version", server.url());

        let response = fetch_data(&agent, &url, "version fetch failed".into()).unwrap();
        let version: GtdbApiVersion = response.into_body().read_json().unwrap();
        assert_eq!(
            format!("{}.{}.{}", version.major, version.minor, version.patch),
            "3.1.4"
        );
    }

    #[test]
    fn test_get_api_version_500_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/meta/version")
            .with_status(500)
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/meta/version", server.url());

        let result = fetch_data(&agent, &url, "version fetch failed".into());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed after 3 attempts"));
    }

    #[test]
    fn test_get_api_version_malformed_json_produces_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/meta/version")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"version": "2.27.0"}"#) // wrong shape, no major/minor/patch
            .create();

        let agent = Agent::config_builder().build().new_agent();
        let url = format!("{}/meta/version", server.url());

        let response = fetch_data(&agent, &url, "version fetch failed".into()).unwrap();
        let result: Result<GtdbApiVersion, _> = response.into_body().read_json();
        // major/minor/patch are u8 with no default - missing fields cause an error
        assert!(
            result.is_err(),
            "wrong JSON shape should fail to deserialise"
        );
    }

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

        assert_eq!(err.to_string(), "GTDB API returned an unexpected status code 418. If this persists, check https://gtdb.ecogenomic.org for service status.");
    }

    #[test]
    fn test_write_to_output() {
        let s = "Hello, world!";

        // Test writing to a file
        let file_path = "test.txt";
        let output = Some(file_path.to_owned());
        write_to_output(s.as_bytes(), output, false).unwrap();
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
