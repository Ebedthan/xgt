// tests/integration.rs
//
// End-to-end integration tests against the live GTDB REST API.
//
// These tests are gated behind the `integration-tests` feature flag so they
// never run in CI. Execute manually before each release:
//
//   cargo test --features integration-tests --test integration
//
// Requirements:
//   - Internet connection
//   - Live GTDB API reachable at https://gtdb-api.ecogenomic.org
//
// The tests use well-known, stable GTDB accessions and taxa that are unlikely
// to disappear between releases. Exact field values (species names, counts)
// are NOT asserted because they change between GTDB releases. Instead, tests
// verify structural correctness: non-empty output, valid JSON, expected CSV
// columns, and correct flag behaviour.

#![cfg(feature = "integration-tests")]

use std::io::Write;

use tempfile::NamedTempFile;
use xgt::cli::{DiffArgs, GenomeArgs, SearchArgs, TaxonArgs};
use xgt::cmd::{diff, genome, search, taxon};

// Well-known test fixtures
// All accessions used here must be:
//   1. NCBI reference genomes or GTDB species representatives
//   2. Present in GTDB since at least R207
// Avoid non-representative accessions, they are frequently removed between
// releases and will cause spurious 404 failures in integration tests.
// REPLACE WITH:
const ECOLI_ACC: &str = "GCA_000005845.2"; // E. coli K-12 MG1655 — NCBI ref
const BSUB_ACC: &str = "GCA_000009045.1"; // B. subtilis 168 — NCBI ref
const SMALL_GENUS: &str = "g__Rhizobium";
const SMALL_TAXON: &str = "g__Escherichia";
const FROM_REL: &str = "R214";
const TO_REL: &str = "R220";

// Helpers

fn search_args_defaults() -> SearchArgs {
    SearchArgs {
        query: None,
        field: "all".into(),
        word: false,
        rep: false,
        r#type: false,
        id: false,
        count: false,
        file: None,
        out: None,
        outfmt: "json".into(),
        split: false,
        split_dir: None,
        release: None,
        insecure: false,
        max_concurrent: 5,
    }
}

fn genome_args_defaults() -> GenomeArgs {
    GenomeArgs {
        accession: None,
        file: None,
        history: false,
        metadata: false,
        outfmt: "json".into(),
        out: None,
        split: false,
        split_dir: None,
        release: None,
        insecure: false,
    }
}

fn taxon_args_defaults() -> TaxonArgs {
    TaxonArgs {
        name: None,
        file: None,
        out: None,
        word: false,
        search: false,
        all: false,
        genomes: false,
        reps: false,
        outfmt: "json".into(),
        split: false,
        split_dir: None,
        release: None,
        insecure: false,
    }
}

fn diff_args_defaults() -> DiffArgs {
    DiffArgs {
        query: None,
        file: None,
        from: FROM_REL.into(),
        to: Some(TO_REL.into()),
        outfmt: "json".into(),
        out: None,
        split: false,
        split_dir: None,
        insecure: false,
        max_concurrent: 5,
    }
}

/// Write lines to a temp file and return it (kept alive by caller).
fn make_input_file(lines: &[&str]) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("failed to create temp file");
    for line in lines {
        writeln!(f, "{}", line).unwrap();
    }
    f
}

/// Run a function that writes to stdout and capture what it produces.
/// We redirect stdout to a temp file, run the function, then read it back.
/*
fn capture_stdout<F: FnOnce() -> anyhow::Result<()>>(f: F) -> String {
    // Integration tests write to actual stdout; we capture via --out flag
    // in individual tests. This helper is a no-op placeholder that just
    // runs the function and returns an empty string for tests using --out.
    f().expect("xgt function returned an error");
    String::new()
}*/
// ═════════════════════════════════════════════════════════════════════════════
// search subcommand
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_search_basic_json_output() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out); // close so xgt can write

    let args = SearchArgs {
        query: Some(SMALL_GENUS.into()),
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..search_args_defaults()
    };

    search::search(&args, false).expect("search failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    // Valid JSON array
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("output is not valid JSON");
    assert!(parsed.is_array(), "JSON output should be an array");
    assert!(
        !parsed.as_array().unwrap().is_empty(),
        "result should not be empty"
    );
}

#[test]
fn test_search_csv_output_has_header() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = SearchArgs {
        query: Some(SMALL_GENUS.into()),
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..search_args_defaults()
    };

    search::search(&args, false).expect("search csv failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    // Header must contain expected column names
    assert!(
        first_line.contains("accession"),
        "CSV header missing 'accession'"
    );
    assert!(
        first_line.contains("ncbi_organism_name"),
        "CSV header missing 'ncbi_organism_name'"
    );
}

#[test]
fn test_search_tsv_output_uses_tabs() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = SearchArgs {
        query: Some(SMALL_GENUS.into()),
        outfmt: "tsv".into(),
        out: Some(out_path.clone()),
        ..search_args_defaults()
    };

    search::search(&args, false).expect("search tsv failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    assert!(
        first_line.contains('\t'),
        "TSV output should use tab separators"
    );
    assert!(
        !first_line.contains(','),
        "TSV output should not contain commas"
    );
}

#[test]
fn test_search_id_flag_returns_clean_accessions() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = SearchArgs {
        query: Some(SMALL_GENUS.into()),
        id: true,
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..search_args_defaults()
    };

    search::search(&args, false).expect("search --id failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(!content.is_empty(), "--id should produce output");

    let first = content.lines().next().unwrap_or("");
    assert!(
        !first.contains("accession,"),
        "--id must not write a CSV header: got {first}"
    );
    for line in content.lines().filter(|l| !l.is_empty()) {
        assert!(
            line.starts_with("GCA_") || line.starts_with("GCF_"),
            "unexpected accession format: {line}"
        );
    }

    // Every line must be a clean accession, no GB_ or RS_ prefixes
    for line in content.lines().filter(|l| !l.is_empty()) {
        assert!(
            !line.starts_with("GB_") && !line.starts_with("RS_"),
            "--id returned prefixed gid instead of clean accession: {line}"
        );
        // Must look like a valid INSDC accession (GCA_ or GCF_)
        assert!(
            line.starts_with("GCA_") || line.starts_with("GCF_"),
            "unexpected accession format: {line}"
        );
    }
}

#[test]
fn test_search_count_flag_returns_integer() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = SearchArgs {
        query: Some(SMALL_GENUS.into()),
        count: true,
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..search_args_defaults()
    };

    search::search(&args, false).expect("search --count failed");

    let content = std::fs::read_to_string(&out_path)
        .unwrap()
        .trim()
        .to_string();
    assert!(
        !content.contains(','),
        "--count output must not contain a header: got {content}"
    );
    let n: u64 = content
        .parse()
        .expect("--count should return a plain integer");
    assert!(n > 0, "--count returned 0");
}

#[test]
fn test_search_rep_flag_restricts_results() {
    let out_all = NamedTempFile::new().unwrap();
    let out_rep = NamedTempFile::new().unwrap();
    let path_all = out_all.path().to_str().unwrap().to_string();
    let path_rep = out_rep.path().to_str().unwrap().to_string();
    drop(out_all);
    drop(out_rep);

    let args_all = SearchArgs {
        query: Some(SMALL_GENUS.into()),
        count: true,
        out: Some(path_all.clone()),
        ..search_args_defaults()
    };
    let args_rep = SearchArgs {
        query: Some(SMALL_GENUS.into()),
        rep: true,
        count: true,
        out: Some(path_rep.clone()),
        ..search_args_defaults()
    };

    search::search(&args_all, false).unwrap();
    search::search(&args_rep, false).unwrap();

    let n_all: u64 = std::fs::read_to_string(&path_all)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let n_rep: u64 = std::fs::read_to_string(&path_rep)
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    assert!(
        n_rep < n_all,
        "--rep should return fewer results than no filter"
    );
    assert!(n_rep > 0, "--rep should return at least one result");
}

#[test]
fn test_search_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let input = make_input_file(&[SMALL_GENUS, "g__Bacillus"]);

    let args = SearchArgs {
        file: Some(input.path().to_str().unwrap().to_string()),
        count: true,
        outfmt: "csv".into(),
        split: true,
        split_dir: Some(dir.path().to_str().unwrap().to_string()),
        ..search_args_defaults()
    };

    search::search(&args, false).expect("search -f failed");

    // One file per query
    let files: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 2, "expected one output file per query");

    // Each file contains a valid integer count
    for entry in std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        let n: u64 = content
            .trim()
            .parse()
            .expect("each count file should contain a plain integer");
        assert!(n > 0, "count should be > 0");
    }
}

#[test]
fn test_search_pagination_returns_all_results() {
    // g__Escherichia has >1000 genomes, requiring multiple pages
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args_count = SearchArgs {
        query: Some("g__Escherichia".into()),
        count: true,
        out: Some(out_path.clone()),
        ..search_args_defaults()
    };
    search::search(&args_count, false).unwrap();
    let total: u64 = std::fs::read_to_string(&out_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(total > 1000, "g__Escherichia should have >1000 genomes");

    // Now fetch --id and count lines, should match the count
    let out2 = NamedTempFile::new().unwrap();
    let out_path2 = out2.path().to_str().unwrap().to_string();
    drop(out2);

    let args_id = SearchArgs {
        query: Some("g__Escherichia".into()),
        id: true,
        out: Some(out_path2.clone()),
        ..search_args_defaults()
    };
    search::search(&args_id, false).unwrap();
    let lines = std::fs::read_to_string(&out_path2).unwrap();
    let n_lines = lines.lines().filter(|l| !l.is_empty()).count() as u64;
    assert_eq!(
        n_lines, total,
        "paginated --id should return exactly as many accessions as --count"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// genome subcommand
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_genome_card_json_output() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = GenomeArgs {
        accession: Some(ECOLI_ACC.into()),
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..genome_args_defaults()
    };

    genome::get_genome_card(&args, false).expect("genome card failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("genome card output is not valid JSON");

    // Must have the expected top-level keys
    assert!(
        parsed["genome"]["accession"].is_string(),
        "missing genome.accession"
    );
    assert!(
        parsed["metadata_gene"]["checkm_completeness"].is_number(),
        "checkm_completeness should be a number"
    );
    assert!(
        parsed["metadata_nucleotide"]["genome_size"].is_number(),
        "genome_size should be a number"
    );
}

#[test]
fn test_genome_card_csv_output_column_count() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = GenomeArgs {
        accession: Some(ECOLI_ACC.into()),
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..genome_args_defaults()
    };

    genome::get_genome_card(&args, false).expect("genome card csv failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let mut lines = content.lines();
    let header = lines.next().unwrap();
    let data = lines.next().unwrap();

    let header_cols = header.split(',').count();
    let data_cols = data.split(',').count();
    assert_eq!(
        header_cols, data_cols,
        "CSV header column count ({header_cols}) != data column count ({data_cols})"
    );
    assert!(
        header_cols >= 50,
        "genome card CSV should have at least 50 columns"
    );
}

#[test]
fn test_genome_metadata_json_output() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = GenomeArgs {
        accession: Some(ECOLI_ACC.into()),
        metadata: true,
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..genome_args_defaults()
    };

    genome::get_genome_metadata(&args, false).expect("genome metadata failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        parsed["accession"].is_string(),
        "metadata must have accession field"
    );
}

#[test]
fn test_genome_history_json_output() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = GenomeArgs {
        accession: Some(ECOLI_ACC.into()),
        history: true,
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..genome_args_defaults()
    };

    genome::get_genome_taxon_history(&args, false).expect("genome history failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_array(), "history output should be a JSON array");
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty(), "history array should not be empty");
    // Each entry must have a release field
    assert!(
        arr[0]["release"].is_string(),
        "history entry missing release field"
    );
}

#[test]
fn test_genome_history_csv_single_header() {
    // Verifies bug 3 fix: one header written, not repeated per accession
    let input = make_input_file(&[ECOLI_ACC, "GCA_000009045.1"]);
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = GenomeArgs {
        file: Some(input.path().to_str().unwrap().to_string()),
        history: true,
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..genome_args_defaults()
    };

    genome::get_genome_taxon_history(&args, false).expect("genome history batch csv failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let header_count = content
        .lines()
        .filter(|l| l.starts_with("release,"))
        .count();
    assert_eq!(
        header_count, 1,
        "header should appear exactly once, found {header_count}"
    );
}

#[test]
fn test_genome_batch_from_file() {
    let input = make_input_file(&[ECOLI_ACC, "GCA_000009045.1"]);
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = GenomeArgs {
        file: Some(input.path().to_str().unwrap().to_string()),
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..genome_args_defaults()
    };

    genome::get_genome_card(&args, false).expect("genome batch failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    // One header + two data rows
    assert_eq!(lines.len(), 3, "expected header + 2 data rows");
}

#[test]
fn test_genome_split_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = make_input_file(&[ECOLI_ACC, "GCA_000009045.1"]);

    let args = GenomeArgs {
        file: Some(input.path().to_str().unwrap().to_string()),
        outfmt: "json".into(),
        split: true,
        split_dir: Some(dir.path().to_str().unwrap().to_string()),
        ..genome_args_defaults()
    };

    genome::get_genome_card(&args, false).expect("genome --split failed");

    let files: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 2, "expected one file per accession");
}

// ═════════════════════════════════════════════════════════════════════════════
// taxon subcommand
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_taxon_name_json_output() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = TaxonArgs {
        name: Some(SMALL_TAXON.into()),
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..taxon_args_defaults()
    };

    taxon::get_taxon_name(&args, false).expect("taxon name failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        parsed.is_array() || parsed.is_object(),
        "taxon output should be JSON"
    );
    assert!(!content.is_empty(), "taxon output should not be empty");
}

#[test]
fn test_taxon_genomes_returns_accessions() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = TaxonArgs {
        name: Some(SMALL_TAXON.into()),
        genomes: true,
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..taxon_args_defaults()
    };

    taxon::get_taxon_genomes(&args, false).expect("taxon --genomes failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(!content.is_empty(), "taxon --genomes should produce output");
}

#[test]
fn test_taxon_search_current_release() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = TaxonArgs {
        name: Some("Escherichia".into()),
        search: true,
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..taxon_args_defaults()
    };

    taxon::search_taxon(&args, false).expect("taxon --search failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(!content.is_empty(), "taxon --search should produce output");
}

#[test]
fn test_taxon_search_all_releases() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = TaxonArgs {
        name: Some("Escherichia".into()),
        search: true,
        all: true,
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..taxon_args_defaults()
    };

    taxon::search_taxon(&args, false).expect("taxon --search --all failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        !content.is_empty(),
        "taxon --search --all should produce output"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// diff subcommand
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_diff_json_output_structure() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = DiffArgs {
        query: Some(ECOLI_ACC.into()),
        from: FROM_REL.into(),
        to: Some(TO_REL.into()),
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..diff_args_defaults()
    };

    diff::diff(&args, false).expect("diff failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Required fields
    assert!(parsed["query"].is_string(), "diff output missing query");
    assert!(
        parsed["from_release"].is_string(),
        "diff output missing from_release"
    );
    assert!(
        parsed["to_release"].is_string(),
        "diff output missing to_release"
    );
    assert!(
        parsed["changed"].is_boolean(),
        "diff output missing changed flag"
    );
    assert!(
        parsed["changes"].is_array(),
        "diff output missing changes array"
    );
    assert!(
        parsed["from_taxonomy"].is_object(),
        "diff output missing from_taxonomy"
    );
    assert!(
        parsed["to_taxonomy"].is_object(),
        "diff output missing to_taxonomy"
    );

    // Query must match what we passed
    assert_eq!(parsed["query"].as_str().unwrap(), ECOLI_ACC);
    assert_eq!(parsed["from_release"].as_str().unwrap(), FROM_REL);
    assert_eq!(parsed["to_release"].as_str().unwrap(), TO_REL);
}

#[test]
fn test_diff_csv_output_structure() {
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = DiffArgs {
        query: Some(ECOLI_ACC.into()),
        from: FROM_REL.into(),
        to: Some(TO_REL.into()),
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..diff_args_defaults()
    };

    diff::diff(&args, false).expect("diff csv failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let header = content.lines().next().unwrap_or("");
    assert!(header.contains("query"), "diff CSV missing query column");
    assert!(
        header.contains("from_release"),
        "diff CSV missing from_release column"
    );
    assert!(
        header.contains("changed"),
        "diff CSV missing changed column"
    );
    assert!(header.contains("rank"), "diff CSV missing rank column");
}

#[test]
fn test_diff_to_omitted_defaults_to_latest() {
    // When --to is omitted, xgt should resolve the latest release automatically
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = DiffArgs {
        query: Some(ECOLI_ACC.into()),
        from: FROM_REL.into(),
        to: None, // omitted
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        ..diff_args_defaults()
    };

    diff::diff(&args, false).expect("diff without --to failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // to_release should be populated even though we did not pass --to
    let to_rel = parsed["to_release"].as_str().unwrap_or("");
    assert!(!to_rel.is_empty(), "to_release should be auto-resolved");
    assert!(
        to_rel.starts_with('R') && to_rel[1..].parse::<u32>().is_ok(),
        "to_release should be a valid release identifier, got: {to_rel}"
    );
}

#[test]
fn test_diff_batch_from_file() {
    let input = make_input_file(&[ECOLI_ACC, "GCA_000008865.1"]);
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = DiffArgs {
        file: Some(input.path().to_str().unwrap().to_string()),
        from: FROM_REL.into(),
        to: Some(TO_REL.into()),
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        ..diff_args_defaults()
    };

    diff::diff(&args, false).expect("diff batch failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    // Header + at least one data row per accession
    let data_rows = content.lines().skip(1).filter(|l| !l.is_empty()).count();
    assert!(data_rows >= 2, "expected at least one row per accession");
}

#[test]
fn test_diff_invalid_release_returns_error() {
    let args = DiffArgs {
        query: Some(ECOLI_ACC.into()),
        from: "R214".into(),
        to: Some("R999".into()), // almost certainly does not exist
        ..diff_args_defaults()
    };

    let result = diff::diff(&args, false);
    assert!(
        result.is_err(),
        "diff with a non-existent --to release should return an error"
    );
}

#[test]
fn test_diff_batch_parallel_preserves_order() {
    // Run a batch of three known accessions and verify output order
    // matches input order regardless of which thread finishes first.
    let input = make_input_file(&[
        ECOLI_ACC,         // GCA_000005845.2
        BSUB_ACC,          // GCA_000009045.1
        "GCA_000006765.1", // P. aeruginosa PAO1
    ]);
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = DiffArgs {
        file: Some(input.path().to_str().unwrap().to_string()),
        from: FROM_REL.into(),
        to: Some(TO_REL.into()),
        outfmt: "csv".into(),
        out: Some(out_path.clone()),
        max_concurrent: 3,
        ..diff_args_defaults()
    };

    diff::diff(&args, false).expect("parallel diff failed");

    let content = std::fs::read_to_string(&out_path).unwrap();
    let rows: Vec<&str> = content
        .lines()
        .skip(1) // skip header
        .filter(|l| !l.is_empty())
        .collect();

    // At least one row per accession
    assert!(rows.len() >= 3, "expected at least 3 rows");

    // First data row must be for ECOLI_ACC (input order preserved)
    assert!(
        rows[0].starts_with(ECOLI_ACC),
        "first row must be for {ECOLI_ACC}, got: {}",
        rows[0]
    );
}

#[test]
fn test_diff_max_concurrent_one_is_sequential() {
    // max_concurrent=1 must produce correct results (degenerates to sequential)
    let out = NamedTempFile::new().unwrap();
    let out_path = out.path().to_str().unwrap().to_string();
    drop(out);

    let args = DiffArgs {
        query: Some(ECOLI_ACC.into()),
        from: FROM_REL.into(),
        to: Some(TO_REL.into()),
        outfmt: "json".into(),
        out: Some(out_path.clone()),
        max_concurrent: 1,
        ..diff_args_defaults()
    };

    diff::diff(&args, false).expect("sequential diff failed");
    let content = std::fs::read_to_string(&out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed["query"].is_string());
}

// ═════════════════════════════════════════════════════════════════════════════
// API availability
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_gtdb_api_is_online() {
    let online = xgt::utils::is_gtdb_db_online(false).expect("is_gtdb_db_online returned an error");
    assert!(online, "GTDB API should be online during integration tests");
}

#[test]
fn test_get_api_version_returns_dotted_version() {
    let version = xgt::utils::get_api_version(false).expect("get_api_version returned an error");
    assert!(!version.is_empty(), "API version should not be empty");
    // Format: "major.minor.patch"
    assert!(
        version.contains('.'),
        "API version should be a dotted string, got: {version}"
    );
}
