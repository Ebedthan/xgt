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
}
