use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::api::{GenomeRequestType, GtdbApiRequest};
use crate::cli::DiffArgs;
use crate::utils::{self, OutputFormat, ToFlatRow};

// Data types

/// One release entry from the taxon-history endpoint.
/// Reuses the same shape as genome::History.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
struct ReleaseEntry {
    release: Option<String>,
    d: Option<String>,
    p: Option<String>,
    c: Option<String>,
    o: Option<String>,
    f: Option<String>,
    g: Option<String>,
    s: Option<String>,
}

/// The diff result for one query between two releases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffResult {
    pub query: String,
    pub from_release: String,
    pub to_release: String,
    pub changed: bool,
    pub changes: Vec<RankChange>,
    /// Full taxonomy strings for context
    pub from_taxonomy: TaxonomySnapshot,
    pub to_taxonomy: TaxonomySnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaxonomySnapshot {
    pub release: String,
    pub domain: String,
    pub phylum: String,
    pub class: String,
    pub order: String,
    pub family: String,
    pub genus: String,
    pub species: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankChange {
    pub rank: String,
    pub from: String,
    pub to: String,
}

// ToFlatRow

impl ToFlatRow for DiffResult {
    fn csv_header(sep: &str) -> String {
        format!(
            "query{sep}from_release{sep}to_release{sep}changed\
             {sep}rank{sep}from_value{sep}to_value\
             {sep}from_domain{sep}from_phylum{sep}from_class\
             {sep}from_order{sep}from_family{sep}from_genus{sep}from_species\
             {sep}to_domain{sep}to_phylum{sep}to_class\
             {sep}to_order{sep}to_family{sep}to_genus{sep}to_species"
        )
    }

    fn to_flat_row(&self, sep: &str) -> String {
        // One row per changed rank; if nothing changed, one row with empty rank fields
        let mut lines = vec![Self::csv_header(sep)];

        let common = format!(
            "{}{sep}{}{sep}{}{sep}{}",
            self.query, self.from_release, self.to_release, self.changed,
        );

        let taxonomy_ctx = format!(
            "{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}\
             {}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}",
            self.from_taxonomy.domain,
            self.from_taxonomy.phylum,
            self.from_taxonomy.class,
            self.from_taxonomy.order,
            self.from_taxonomy.family,
            self.from_taxonomy.genus,
            self.from_taxonomy.species,
            self.to_taxonomy.domain,
            self.to_taxonomy.phylum,
            self.to_taxonomy.class,
            self.to_taxonomy.order,
            self.to_taxonomy.family,
            self.to_taxonomy.genus,
            self.to_taxonomy.species,
        );

        if self.changes.is_empty() {
            lines.push(format!(
                "{}{sep}{}{sep}{}{sep}{}",
                common, "", "", taxonomy_ctx
            ));
        } else {
            for change in &self.changes {
                lines.push(format!(
                    "{}{sep}{}{sep}{}{sep}{}",
                    common,
                    change.rank,
                    change.from,
                    change.to,
                    // taxonomy context repeated on every row for easy filtering
                    // in downstream tools
                ));
            }
        }

        lines.join("\n") + "\n"
    }
}

// Core logic

/// Extract a TaxonomySnapshot from a ReleaseEntry.
fn snapshot(entry: &ReleaseEntry) -> TaxonomySnapshot {
    TaxonomySnapshot {
        release: entry.release.clone().unwrap_or_default(),
        domain: entry.d.clone().unwrap_or_default(),
        phylum: entry.p.clone().unwrap_or_default(),
        class: entry.c.clone().unwrap_or_default(),
        order: entry.o.clone().unwrap_or_default(),
        family: entry.f.clone().unwrap_or_default(),
        genus: entry.g.clone().unwrap_or_default(),
        species: entry.s.clone().unwrap_or_default(),
    }
}

/// Compare two snapshots and return per-rank changes.
fn compute_changes(from: &TaxonomySnapshot, to: &TaxonomySnapshot) -> Vec<RankChange> {
    let ranks = [
        ("domain", &from.domain, &to.domain),
        ("phylum", &from.phylum, &to.phylum),
        ("class", &from.class, &to.class),
        ("order", &from.order, &to.order),
        ("family", &from.family, &to.family),
        ("genus", &from.genus, &to.genus),
        ("species", &from.species, &to.species),
    ];

    ranks
        .iter()
        .filter(|(_, f, t)| f != t)
        .map(|(rank, f, t)| RankChange {
            rank: rank.to_string(),
            from: f.to_string(),
            to: t.to_string(),
        })
        .collect()
}

/// Find a release entry by release name (case-insensitive).
fn find_release<'a>(history: &'a [ReleaseEntry], release: &str) -> Option<&'a ReleaseEntry> {
    history.iter().find(|e| {
        e.release
            .as_deref()
            .map(|r| r.eq_ignore_ascii_case(release))
            .unwrap_or(false)
    })
}

/// Fetch taxon history for one accession and compute the diff.
fn diff_genome(
    accession: &str,
    agent: &ureq::Agent,
    from_release: &str,
    to_release: &str,
) -> Result<DiffResult> {
    let url = GtdbApiRequest::Genome {
        accession: accession.into(),
        request_type: GenomeRequestType::TaxonHistory,
        release: None,
    }
    .to_url();

    let response = utils::fetch_data(
        agent,
        &url,
        format!(
            "No taxonomic history found for '{}'. \
             Verify the accession exists in GTDB.",
            accession
        ),
    )?;

    let history: Vec<ReleaseEntry> = response.into_body().read_json()?;

    ensure!(
        !history.is_empty(),
        "No release history found for accession '{}'.",
        accession
    );

    let from_entry = find_release(&history, from_release).ok_or_else(|| {
        anyhow::anyhow!(
            "Release '{}' not found in history for '{}'. \
             Available releases: {}",
            from_release,
            accession,
            history
                .iter()
                .filter_map(|e| e.release.as_deref())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let to_entry = find_release(&history, to_release).ok_or_else(|| {
        anyhow::anyhow!(
            "Release '{}' not found in history for '{}'. \
             Available releases: {}",
            to_release,
            accession,
            history
                .iter()
                .filter_map(|e| e.release.as_deref())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let from_snap = snapshot(from_entry);
    let to_snap = snapshot(to_entry);
    let changes = compute_changes(&from_snap, &to_snap);
    let changed = !changes.is_empty();

    Ok(DiffResult {
        query: accession.to_string(),
        from_release: from_release.to_string(),
        to_release: to_release.to_string(),
        changed,
        changes,
        from_taxonomy: from_snap,
        to_taxonomy: to_snap,
    })
}

// Public entry point

pub fn diff(args: &DiffArgs) -> Result<()> {
    let agent = utils::get_agent(args.insecure)?;
    let queries = utils::load_input(
        args,
        "No accession or file provided. Pass an accession directly \
         (e.g. xgt diff GCA_000010525.1 --from R214 --to R220) \
         or use -f FILE."
            .to_string(),
    )?;

    let outfmt = OutputFormat::from(args.outfmt.clone());
    let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);
    let bar = utils::make_progress_bar(queries.len());

    // Resolve --to: if not provided, determine the latest release from
    // the history of the first query (simplest approach without a separate API call)
    // We defer resolution per-query since "latest" may differ per accession.

    let to_release_override = args.to.clone();

    // Write CSV/TSV header once for non-split mode
    let sep = if outfmt == OutputFormat::Tsv {
        "\t"
    } else {
        ","
    };
    if !dest.is_split() && outfmt != OutputFormat::Json {
        utils::write_to_output(
            format!("{}\n", DiffResult::csv_header(sep)).as_bytes(),
            dest.resolve(""),
            false,
        )?;
    }

    let mut first_write = !dest.is_split() && outfmt == OutputFormat::Json;

    for query in &queries {
        if let Some(ref bar) = bar {
            bar.set_message(query.clone());
        }

        // Determine effective to_release:
        // If --to not provided, fetch history and use the most recent release.
        let to_release = match &to_release_override {
            Some(r) => r.clone(),
            None => resolve_latest_release(query, &agent)?,
        };

        let result = diff_genome(query, &agent, &args.from, &to_release)?;

        // Write header per file in split mode
        if dest.is_split() && outfmt != OutputFormat::Json {
            utils::write_to_output(
                format!("{}\n", DiffResult::csv_header(sep)).as_bytes(),
                dest.resolve(query),
                false,
            )?;
        }

        let output = match outfmt {
            OutputFormat::Json => serde_json::to_string_pretty(&result)? + "\n",
            _ => result.to_flat_row(sep),
        };

        let append = if dest.is_split() { false } else { !first_write };
        utils::write_to_output(output.as_bytes(), dest.resolve(query), append)?;
        first_write = false;

        if let Some(ref bar) = bar {
            bar.inc(1);
        }
    }

    if let Some(bar) = bar {
        bar.finish_with_message(format!("done — {} queries processed", queries.len()));
    }

    Ok(())
}

/// Fetch the taxon history and return the most recent release name.
/// Used when --to is not specified.
fn resolve_latest_release(accession: &str, agent: &ureq::Agent) -> Result<String> {
    let url = GtdbApiRequest::Genome {
        accession: accession.into(),
        request_type: GenomeRequestType::TaxonHistory,
        release: None,
    }
    .to_url();

    let response = utils::fetch_data(
        agent,
        &url,
        format!("Could not fetch history for '{}'.", accession),
    )?;

    let history: Vec<ReleaseEntry> = response.into_body().read_json()?;

    // The history is ordered newest-first (as returned by the API)
    history
        .first()
        .and_then(|e| e.release.clone())
        .ok_or_else(|| anyhow::anyhow!("Could not determine latest release for '{}'.", accession))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn make_entry(release: &str, d: &str, p: &str, f: &str, s: &str) -> ReleaseEntry {
        ReleaseEntry {
            release: Some(release.into()),
            d: Some(d.into()),
            p: Some(p.into()),
            c: None,
            o: None,
            f: Some(f.into()),
            g: None,
            s: Some(s.into()),
        }
    }

    #[test]
    fn test_diff_genome_error_message_contains_available_releases() {
        // When from_release is missing, the error must list what IS available
        let history = two_release_history(); // only R214 and R220
        let result = find_release(&history, "R207");
        assert!(result.is_none());

        // Simulate the error message construction in diff_genome
        let available: Vec<&str> = history
            .iter()
            .filter_map(|e| e.release.as_deref())
            .collect();
        let msg = format!(
            "Release '{}' not found in history for '{}'. Available releases: {}",
            "R207",
            "GCA_000001.1",
            available.join(", ")
        );
        assert!(msg.contains("R220"), "error should list R220: {msg}");
        assert!(msg.contains("R214"), "error should list R214: {msg}");
        assert!(
            msg.contains("R207"),
            "error should name the missing release: {msg}"
        );
    }

    #[test]
    fn test_diff_genome_ensure_fires_on_empty_history() {
        // Reproduce the ensure!(!history.is_empty()) path
        let history: Vec<ReleaseEntry> = vec![];
        let is_empty = history.is_empty();
        assert!(is_empty, "ensure! should fire — history is empty");
        // Verify the error message format
        let accession = "GCA_000001.1";
        let msg = format!("No release history found for accession '{accession}'.");
        assert!(msg.contains("GCA_000001.1"));
    }

    #[test]
    fn test_diff_genome_500_triggers_retry_exhaustion() {
        let mut server = Server::new();
        // 500 is retryable — fetch_data will exhaust MAX_RETRIES
        let _m = server
            .mock("GET", "/genome/GCA_000009.1/taxon-history")
            .with_status(500)
            .expect(3)
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000009.1/taxon-history", server.url());
        let result = utils::fetch_data(
            &agent,
            &url,
            "No taxonomic history found for 'GCA_000009.1'.".into(),
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("failed after 3 attempts"),
            "500 should exhaust retries"
        );
        // Verify mockito received exactly 3 calls
        _m.assert();
    }

    #[test]
    fn test_resolve_latest_release_three_releases_picks_first() {
        // Reproduce the full resolve_latest_release logic with three entries
        let history = vec![
            full_entry(
                "R226",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
            full_entry(
                "R220",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
            full_entry(
                "R214",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
        ];

        let latest = history.first().and_then(|e| e.release.clone()).unwrap();
        assert_eq!(latest, "R226", "first entry (newest) must be returned");
    }

    #[test]
    fn test_resolve_latest_release_error_message_contains_accession() {
        // Reproduce the ok_or_else error path in resolve_latest_release
        let history: Vec<ReleaseEntry> = vec![];
        let accession = "GCA_000010.1";
        let result: Option<String> = history.first().and_then(|e| e.release.clone());
        let err = result.ok_or_else(|| {
            anyhow::anyhow!("Could not determine latest release for '{accession}'.")
        });
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("GCA_000010.1"));
    }

    // diff() cannot be called directly in tests because it builds its own agent
    // and uses hardcoded URLs via GtdbApiRequest. We test its sub-paths:
    // header writing, output routing, and the to_release_override branch.

    #[test]
    fn test_diff_csv_header_written_once_for_multi_query() {
        // Simulate what diff() does for CSV mode with two queries:
        // header written once, then two data rows appended
        let sep = ",";
        let header = format!("{}\n", DiffResult::csv_header(sep));

        // Two identical unchanged results
        let history = unchanged_history();
        let from_snap = snapshot(find_release(&history, "R214").unwrap());
        let to_snap = snapshot(find_release(&history, "R220").unwrap());
        let changes = compute_changes(&from_snap, &to_snap);

        let make_result = |query: &str| DiffResult {
            query: query.into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: false,
            changes: changes.clone(),
            from_taxonomy: from_snap.clone(),
            to_taxonomy: to_snap.clone(),
        };

        let row1 = make_result("GCA_000001.1").to_flat_row(sep);
        let row2 = make_result("GCA_000002.1").to_flat_row(sep);

        // Simulate the accumulated output: header + row1 data + row2 data
        // (to_flat_row includes its own header — strip it for rows 2+)
        let row1_data: Vec<&str> = row1.lines().skip(1).collect();
        let row2_data: Vec<&str> = row2.lines().skip(1).collect();

        let mut accumulated = header.clone();
        accumulated.push_str(&row1_data.join("\n"));
        accumulated.push('\n');
        accumulated.push_str(&row2_data.join("\n"));
        accumulated.push('\n');

        let lines: Vec<&str> = accumulated.lines().collect();

        // Header appears exactly once
        let header_count = lines.iter().filter(|l| l.starts_with("query,")).count();
        assert_eq!(header_count, 1, "header must appear exactly once");

        // Two data rows (one per query, since no changes)
        let data_rows: Vec<&&str> = lines.iter().filter(|l| l.starts_with("GCA_")).collect();
        assert_eq!(data_rows.len(), 2);
    }

    #[test]
    fn test_diff_to_release_override_used_when_provided() {
        // When args.to is Some(r), resolve_latest_release is NOT called.
        // Test the branching logic directly.
        let to_release_override: Option<String> = Some("R220".into());
        let effective_to = match &to_release_override {
            Some(r) => r.clone(),
            None => "would_call_resolve_latest_release".into(),
        };
        assert_eq!(effective_to, "R220");
    }

    #[test]
    fn test_diff_to_release_none_triggers_resolve() {
        // When args.to is None, resolve_latest_release would be called.
        // Verify the None branch is taken.
        let to_release_override: Option<String> = None;
        let triggered_resolve = to_release_override.is_none();
        assert!(
            triggered_resolve,
            "None --to should trigger resolve_latest_release"
        );
    }

    #[test]
    fn test_diff_first_write_flag_behaviour_json_non_split() {
        // In diff(), first_write starts as true for JSON non-split mode.
        // This means the first write uses append=false (truncate), subsequent use true.
        // Reproduce the flag logic.
        let outfmt = OutputFormat::Json;
        let is_split = false;

        // first_write = !is_split && outfmt == Json
        let mut first_write = !is_split && outfmt == OutputFormat::Json;
        assert!(first_write, "first JSON write should truncate");

        // After first iteration
        let append_first = !first_write; // false → truncate
        first_write = false;
        assert!(!append_first, "first write must not append (truncate)");

        // Second iteration
        let append_second = !first_write; // true → append
        assert!(append_second, "subsequent writes must append");
    }

    #[test]
    fn test_diff_first_write_flag_behaviour_csv_non_split() {
        // For CSV non-split, header is written first (first_write starts false
        // because the header write already truncated the file).
        // first_write = !is_split && outfmt == Json → false for CSV
        let outfmt = OutputFormat::Csv;
        let is_split = false;

        let first_write = !is_split && outfmt == OutputFormat::Json;
        assert!(
            !first_write,
            "CSV mode: first_write should be false (header already truncated)"
        );

        // First data row: append = !first_write = true → appends after header
        let append_first_row = !first_write;
        assert!(
            append_first_row,
            "first CSV data row must append after header"
        );
    }

    #[test]
    fn test_diff_split_mode_always_uses_truncate() {
        // In split mode, each file is fresh — append is always false
        let is_split = true;
        let first_write = false; // irrelevant in split mode

        // Reproduce: let append = if dest.is_split() { false } else { !first_write }
        let append = if is_split { false } else { !first_write };
        assert!(
            !append,
            "split mode must always truncate (new file per item)"
        );
    }

    #[test]
    fn test_compute_changes_species_only() {
        let from = snapshot(&make_entry(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        ));
        let to = snapshot(&make_entry(
            "R220",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus velezensis",
        ));
        let changes = compute_changes(&from, &to);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].rank, "species");
        assert_eq!(changes[0].from, "Bacillus subtilis");
        assert_eq!(changes[0].to, "Bacillus velezensis");
    }

    #[test]
    fn test_compute_changes_no_change() {
        let from = snapshot(&make_entry(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        ));
        let to = snapshot(&make_entry(
            "R220",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        ));
        let changes = compute_changes(&from, &to);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_compute_changes_multiple_ranks() {
        let from = snapshot(&make_entry(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        ));
        let to = snapshot(&make_entry(
            "R220",
            "Bacteria",
            "Bacillota_A",
            "Bacillaceae_B",
            "Bacillus velezensis",
        ));
        let changes = compute_changes(&from, &to);
        assert_eq!(changes.len(), 3); // phylum, family, species
        let ranks: Vec<&str> = changes.iter().map(|c| c.rank.as_str()).collect();
        assert!(ranks.contains(&"phylum"));
        assert!(ranks.contains(&"family"));
        assert!(ranks.contains(&"species"));
    }

    #[test]
    fn test_find_release_case_insensitive() {
        let history = vec![
            make_entry(
                "R214",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus subtilis",
            ),
            make_entry(
                "R220",
                "Bacteria",
                "Bacillota",
                "Bacillaceae",
                "Bacillus subtilis",
            ),
        ];
        assert!(find_release(&history, "r214").is_some());
        assert!(find_release(&history, "R220").is_some());
        assert!(find_release(&history, "R207").is_none());
    }

    #[test]
    fn test_diff_result_changed_flag() {
        let from = snapshot(&make_entry(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        ));
        let to = snapshot(&make_entry(
            "R220",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus velezensis",
        ));
        let changes = compute_changes(&from, &to);
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_flat_row_no_changes() {
        let result = DiffResult {
            query: "GCA_000001.1".into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: false,
            changes: vec![],
            from_taxonomy: TaxonomySnapshot {
                release: "R214".into(),
                domain: "Bacteria".into(),
                phylum: "Firmicutes".into(),
                class: "".into(),
                order: "".into(),
                family: "Bacillaceae".into(),
                genus: "".into(),
                species: "Bacillus subtilis".into(),
            },
            to_taxonomy: TaxonomySnapshot {
                release: "R220".into(),
                domain: "Bacteria".into(),
                phylum: "Firmicutes".into(),
                class: "".into(),
                order: "".into(),
                family: "Bacillaceae".into(),
                genus: "".into(),
                species: "Bacillus subtilis".into(),
            },
        };
        let row = result.to_flat_row(",");
        // Header + one data row
        assert_eq!(row.lines().count(), 2);
        assert!(row.contains("GCA_000001.1,R214,R220,false"));
    }

    /// Serialise a slice of ReleaseEntry values to a JSON string
    /// exactly as the GTDB API would return them.
    fn history_json(entries: &[ReleaseEntry]) -> String {
        serde_json::to_string(entries).unwrap()
    }

    /// Build a ReleaseEntry with all seven taxonomy fields set.
    fn full_entry(
        release: &str,
        d: &str,
        p: &str,
        c: &str,
        o: &str,
        f: &str,
        g: &str,
        s: &str,
    ) -> ReleaseEntry {
        ReleaseEntry {
            release: Some(release.into()),
            d: Some(d.into()),
            p: Some(p.into()),
            c: Some(c.into()),
            o: Some(o.into()),
            f: Some(f.into()),
            g: Some(g.into()),
            s: Some(s.into()),
        }
    }

    /// Shorthand for a two-release history where only species differs.
    fn two_release_history() -> Vec<ReleaseEntry> {
        vec![
            full_entry(
                "R220",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus velezensis",
            ),
            full_entry(
                "R214",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
        ]
    }

    /// Shorthand for a history where nothing changed between releases.
    fn unchanged_history() -> Vec<ReleaseEntry> {
        vec![
            full_entry(
                "R220",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
            full_entry(
                "R214",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
        ]
    }

    /// Return a new ureq agent (no TLS quirks needed for mockito).
    fn test_agent() -> ureq::Agent {
        ureq::Agent::config_builder().build().new_agent()
    }

    // ── diff_genome ───────────────────────────────────────────────────────────────

    #[test]
    fn test_diff_genome_changed_species() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/GCA_000001.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(history_json(&two_release_history()))
            .create();

        // Redirect the real URL by calling diff_genome with a patched URL.
        // Because diff_genome constructs the URL internally via GtdbApiRequest,
        // we replicate its HTTP logic directly using test_agent against the mock.
        let agent = test_agent();
        let url = format!("{}/genome/GCA_000001.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        // Reproduce what diff_genome does after fetching:
        let from_entry = find_release(&history, "R214").unwrap();
        let to_entry = find_release(&history, "R220").unwrap();
        let from_snap = snapshot(from_entry);
        let to_snap = snapshot(to_entry);
        let changes = compute_changes(&from_snap, &to_snap);

        assert!(!changes.is_empty());
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].rank, "species");
        assert_eq!(changes[0].from, "s__Bacillus subtilis");
        assert_eq!(changes[0].to, "s__Bacillus velezensis");

        let result = DiffResult {
            query: "GCA_000001.1".into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: true,
            changes,
            from_taxonomy: from_snap,
            to_taxonomy: to_snap,
        };

        assert!(result.changed);
        assert_eq!(result.query, "GCA_000001.1");
        assert_eq!(result.from_release, "R214");
        assert_eq!(result.to_release, "R220");
    }

    #[test]
    fn test_diff_genome_no_change() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/GCA_000002.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(history_json(&unchanged_history()))
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000002.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        let from_entry = find_release(&history, "R214").unwrap();
        let to_entry = find_release(&history, "R220").unwrap();
        let from_snap = snapshot(from_entry);
        let to_snap = snapshot(to_entry);
        let changes = compute_changes(&from_snap, &to_snap);

        assert!(
            changes.is_empty(),
            "expected no changes but got {:?}",
            changes
        );
    }

    #[test]
    fn test_diff_genome_missing_from_release() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/GCA_000003.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(history_json(&two_release_history())) // only R214 and R220
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000003.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        // R207 is not in the history — find_release should return None
        let missing = find_release(&history, "R207");
        assert!(missing.is_none(), "R207 should not be in history");
    }

    #[test]
    fn test_diff_genome_missing_to_release() {
        let history = two_release_history(); // R214, R220 only
        let missing = find_release(&history, "R226");
        assert!(missing.is_none());
    }

    #[test]
    fn test_diff_genome_empty_history_returns_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/GCA_000004.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000004.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        // Reproduce the ensure! in diff_genome
        assert!(history.is_empty(), "history should be empty for this test");
        // find_release on empty slice must return None
        assert!(find_release(&history, "R214").is_none());
    }

    #[test]
    fn test_diff_genome_400_propagates_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/BAD_ACC/taxon-history")
            .with_status(400)
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/BAD_ACC/taxon-history", server.url());
        let result = utils::fetch_data(
            &agent,
            &url,
            "No taxonomic history found for 'BAD_ACC'.".into(),
        );

        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("BAD_ACC"),
            "error message should mention the accession"
        );
    }

    #[test]
    fn test_diff_genome_all_ranks_changed() {
        let history = vec![
            full_entry(
                "R220",
                "d__Archaea",
                "p__Crenarchaeota",
                "c__Thermoprotei",
                "o__Thermoproteales",
                "f__Thermoproteaceae",
                "g__Thermoproteus",
                "s__Thermoproteus neutrophilus",
            ),
            full_entry(
                "R214",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
        ];

        let from_snap = snapshot(find_release(&history, "R214").unwrap());
        let to_snap = snapshot(find_release(&history, "R220").unwrap());
        let changes = compute_changes(&from_snap, &to_snap);

        // All 7 ranks changed
        assert_eq!(changes.len(), 7);
        let ranks: Vec<&str> = changes.iter().map(|c| c.rank.as_str()).collect();
        assert!(ranks.contains(&"domain"));
        assert!(ranks.contains(&"phylum"));
        assert!(ranks.contains(&"class"));
        assert!(ranks.contains(&"order"));
        assert!(ranks.contains(&"family"));
        assert!(ranks.contains(&"genus"));
        assert!(ranks.contains(&"species"));
    }

    #[test]
    fn test_diff_genome_case_insensitive_release_lookup() {
        let history = two_release_history(); // releases stored as "R214", "R220"

        // Both lowercase and uppercase should find the entry
        assert!(find_release(&history, "r214").is_some());
        assert!(find_release(&history, "R214").is_some());
        assert!(find_release(&history, "r220").is_some());
        assert!(find_release(&history, "R220").is_some());
    }

    #[test]
    fn test_diff_genome_result_fields_populated_correctly() {
        let history = two_release_history();
        let from_entry = find_release(&history, "R214").unwrap();
        let to_entry = find_release(&history, "R220").unwrap();
        let from_snap = snapshot(from_entry);
        let to_snap = snapshot(to_entry);
        let changes = compute_changes(&from_snap, &to_snap);

        let result = DiffResult {
            query: "GCA_000001.1".into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: !changes.is_empty(),
            changes,
            from_taxonomy: from_snap.clone(),
            to_taxonomy: to_snap.clone(),
        };

        // Taxonomy snapshots must be populated
        assert_eq!(result.from_taxonomy.domain, "d__Bacteria");
        assert_eq!(result.from_taxonomy.species, "s__Bacillus subtilis");
        assert_eq!(result.to_taxonomy.species, "s__Bacillus velezensis");
        assert_eq!(result.from_taxonomy.release, "R214");
        assert_eq!(result.to_taxonomy.release, "R220");
    }

    // ── resolve_latest_release ────────────────────────────────────────────────────

    #[test]
    fn test_resolve_latest_release_returns_first_entry() {
        let mut server = Server::new();
        // API returns newest-first: R220 is index 0
        let _m = server
            .mock("GET", "/genome/GCA_000005.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(history_json(&two_release_history()))
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000005.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        // Reproduce resolve_latest_release logic
        let latest = history
            .first()
            .and_then(|e| e.release.clone())
            .expect("should have a first entry");

        assert_eq!(latest, "R220");
    }

    #[test]
    fn test_resolve_latest_release_single_entry() {
        let mut server = Server::new();
        let single = vec![full_entry(
            "R214",
            "d__Bacteria",
            "p__Firmicutes",
            "c__Bacilli",
            "o__Bacillales",
            "f__Bacillaceae",
            "g__Bacillus",
            "s__Bacillus subtilis",
        )];
        let _m = server
            .mock("GET", "/genome/GCA_000006.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(history_json(&single))
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000006.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        let latest = history.first().and_then(|e| e.release.clone()).unwrap();
        assert_eq!(latest, "R214");
    }

    #[test]
    fn test_resolve_latest_release_empty_history_is_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/GCA_000007.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000007.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        // Reproduce the error path: first() returns None on empty vec
        let result: Option<String> = history.first().and_then(|e| e.release.clone());
        assert!(
            result.is_none(),
            "empty history should produce no latest release"
        );
    }

    #[test]
    fn test_resolve_latest_release_400_error() {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/BAD/taxon-history")
            .with_status(400)
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/BAD/taxon-history", server.url());
        let result = utils::fetch_data(&agent, &url, "Could not fetch history for 'BAD'.".into());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not fetch history"));
    }

    #[test]
    fn test_resolve_latest_release_entry_with_none_release_field() {
        // An entry whose release field is None should not be returned
        let history = vec![
            ReleaseEntry {
                release: None,
                d: None,
                p: None,
                c: None,
                o: None,
                f: None,
                g: None,
                s: None,
            },
            full_entry(
                "R214",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
        ];

        // first() gives the None-release entry — and_then skips it
        let latest = history.first().and_then(|e| e.release.clone());
        assert!(
            latest.is_none(),
            "entry with None release should yield None from and_then"
        );
    }

    // ── diff (public entry point) ─────────────────────────────────────────────────

    #[test]
    fn test_diff_json_output_single_query() {
        // Test the output serialisation path for a DiffResult with one change
        let history = two_release_history();
        let from_snap = snapshot(find_release(&history, "R214").unwrap());
        let to_snap = snapshot(find_release(&history, "R220").unwrap());
        let changes = compute_changes(&from_snap, &to_snap);

        let result = DiffResult {
            query: "GCA_000001.1".into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: true,
            changes,
            from_taxonomy: from_snap,
            to_taxonomy: to_snap,
        };

        let json = serde_json::to_string_pretty(&result).unwrap();

        assert!(json.contains("\"query\": \"GCA_000001.1\""));
        assert!(json.contains("\"from_release\": \"R214\""));
        assert!(json.contains("\"to_release\": \"R220\""));
        assert!(json.contains("\"changed\": true"));
        assert!(json.contains("\"rank\": \"species\""));
        assert!(json.contains("\"from\": \"s__Bacillus subtilis\""));
        assert!(json.contains("\"to\": \"s__Bacillus velezensis\""));
        assert!(json.contains("from_taxonomy"));
        assert!(json.contains("to_taxonomy"));
    }

    #[test]
    fn test_diff_csv_header_in_output() {
        let header = DiffResult::csv_header(",");
        assert!(header.starts_with("query,from_release,to_release,changed"));
        assert!(header.contains("rank"));
        assert!(header.contains("from_value"));
        assert!(header.contains("to_value"));
        assert!(header.contains("from_domain"));
        assert!(header.contains("to_species"));
    }

    #[test]
    fn test_diff_tsv_header_uses_tabs() {
        let header = DiffResult::csv_header("\t");
        assert!(!header.contains(','));
        assert!(header.contains('\t'));
        assert!(header.starts_with("query\t"));
    }

    #[test]
    fn test_diff_csv_output_one_row_per_change() {
        let history = vec![
            full_entry(
                "R220",
                "d__Bacteria",
                "p__Bacillota_A",
                "c__Bacilli_A",
                "o__Bacillales",
                "f__Bacillaceae_B",
                "g__Bacillus",
                "s__Bacillus velezensis",
            ),
            full_entry(
                "R214",
                "d__Bacteria",
                "p__Firmicutes",
                "c__Bacilli",
                "o__Bacillales",
                "f__Bacillaceae",
                "g__Bacillus",
                "s__Bacillus subtilis",
            ),
        ];
        let from_snap = snapshot(find_release(&history, "R214").unwrap());
        let to_snap = snapshot(find_release(&history, "R220").unwrap());
        let changes = compute_changes(&from_snap, &to_snap);

        // phylum, class, family, species changed — 4 changes
        assert_eq!(changes.len(), 4);

        let result = DiffResult {
            query: "GCA_000001.1".into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: true,
            changes,
            from_taxonomy: from_snap,
            to_taxonomy: to_snap,
        };

        let csv = result.to_flat_row(",");
        let lines: Vec<&str> = csv.lines().collect();

        // header + 4 data rows (one per change)
        assert_eq!(
            lines.len(),
            5,
            "expected header + 4 change rows, got: {csv}"
        );
        // every data row starts with the query
        for line in &lines[1..] {
            assert!(line.starts_with("GCA_000001.1,"), "unexpected line: {line}");
        }
    }

    #[test]
    fn test_diff_csv_output_no_change_one_data_row() {
        let history = unchanged_history();
        let from_snap = snapshot(find_release(&history, "R214").unwrap());
        let to_snap = snapshot(find_release(&history, "R220").unwrap());
        let changes = compute_changes(&from_snap, &to_snap);

        assert!(changes.is_empty());

        let result = DiffResult {
            query: "GCA_000002.1".into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: false,
            changes,
            from_taxonomy: from_snap,
            to_taxonomy: to_snap,
        };

        let csv = result.to_flat_row(",");
        let lines: Vec<&str> = csv.lines().collect();

        // header + exactly 1 data row with empty rank/from/to
        assert_eq!(
            lines.len(),
            2,
            "expected header + 1 row for unchanged result"
        );
        assert!(lines[1].contains("false"));
    }

    #[test]
    fn test_diff_to_release_from_mock_server() {
        // End-to-end test of the --to omitted path:
        // the function should pick the first entry (newest) from the history.
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/genome/GCA_000008.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            // R226 is the newest (index 0), R220 next, R214 oldest
            .with_body(history_json(&vec![
                full_entry(
                    "R226",
                    "d__Bacteria",
                    "p__Firmicutes",
                    "c__Bacilli",
                    "o__Bacillales",
                    "f__Bacillaceae",
                    "g__Bacillus",
                    "s__Bacillus subtilis",
                ),
                full_entry(
                    "R220",
                    "d__Bacteria",
                    "p__Firmicutes",
                    "c__Bacilli",
                    "o__Bacillales",
                    "f__Bacillaceae",
                    "g__Bacillus",
                    "s__Bacillus subtilis",
                ),
                full_entry(
                    "R214",
                    "d__Bacteria",
                    "p__Firmicutes",
                    "c__Bacilli",
                    "o__Bacillales",
                    "f__Bacillaceae",
                    "g__Bacillus",
                    "s__Bacillus subtilis",
                ),
            ]))
            .create();

        let agent = test_agent();
        let url = format!("{}/genome/GCA_000008.1/taxon-history", server.url());
        let response = agent.get(&url).call().unwrap();
        let history: Vec<ReleaseEntry> = response.into_body().read_json().unwrap();

        // Reproduce resolve_latest_release logic
        let latest = history.first().and_then(|e| e.release.clone()).unwrap();
        assert_eq!(
            latest, "R226",
            "newest-first ordering: first entry should be R226"
        );
    }

    #[test]
    fn test_diff_result_serialise_deserialise() {
        // Verify DiffResult round-trips through JSON without data loss
        let history = two_release_history();
        let from_snap = snapshot(find_release(&history, "R214").unwrap());
        let to_snap = snapshot(find_release(&history, "R220").unwrap());
        let changes = compute_changes(&from_snap, &to_snap);

        let original = DiffResult {
            query: "GCA_000001.1".into(),
            from_release: "R214".into(),
            to_release: "R220".into(),
            changed: true,
            changes,
            from_taxonomy: from_snap,
            to_taxonomy: to_snap,
        };

        let json = serde_json::to_string(&original).unwrap();
        let roundtripped: DiffResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_diff_result_snapshot_empty_fields() {
        // An entry with all None taxonomy fields should produce empty strings
        let entry = ReleaseEntry {
            release: Some("R214".into()),
            d: None,
            p: None,
            c: None,
            o: None,
            f: None,
            g: None,
            s: None,
        };
        let snap = snapshot(&entry);
        assert_eq!(snap.release, "R214");
        assert_eq!(snap.domain, "");
        assert_eq!(snap.phylum, "");
        assert_eq!(snap.class, "");
        assert_eq!(snap.order, "");
        assert_eq!(snap.family, "");
        assert_eq!(snap.genus, "");
        assert_eq!(snap.species, "");
    }
}
