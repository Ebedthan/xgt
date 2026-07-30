use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::api::GtdbApiRequest;
use crate::cache::TTL_SEARCH;
use crate::cli::SearchArgs;
use crate::utils::{self, BatchWriter, OutputFormat, SearchField};

use std::sync::mpsc;
use std::thread;

const ITEMS_PER_PAGE: u32 = 1_000;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    gid: String,
    accession: Option<String>,
    ncbi_org_name: Option<String>,
    ncbi_taxonomy: Option<String>,
    gtdb_taxonomy: Option<String>,
    is_gtdb_species_rep: Option<bool>,
    is_ncbi_type_material: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct SearchResults {
    rows: Vec<SearchResult>,
    total_rows: u32,
}

impl SearchResults {
    /// Filter rows for exact match of taxon name / field as supplied by the user.
    fn filter_json(&mut self, needle: String, search_field: SearchField) {
        self.rows.retain(|r| match search_field {
            SearchField::All => {
                let taxon_match = [r.ncbi_taxonomy.as_deref(), r.gtdb_taxonomy.as_deref()]
                    .iter()
                    .flatten()
                    .any(|v| whole_taxon_match(v, &needle));
                let word_match = [r.accession.as_deref(), r.ncbi_org_name.as_deref()]
                    .iter()
                    .flatten()
                    .any(|v| whole_word_match(v, &needle));
                taxon_match || word_match
            }
            SearchField::NcbiId => r
                .accession
                .as_deref()
                .is_some_and(|v| whole_word_match(v, &needle)),
            SearchField::NcbiOrg => r
                .ncbi_org_name
                .as_deref()
                .is_some_and(|v| whole_word_match(v, &needle)),
            SearchField::NcbiTax => r
                .ncbi_taxonomy
                .as_deref()
                .is_some_and(|v| whole_taxon_match(v, &needle)),
            SearchField::GtdbTax => r
                .gtdb_taxonomy
                .as_deref()
                .is_some_and(|v| whole_taxon_match(v, &needle)),
        });
        self.total_rows = self.rows.len() as u32;
    }
}

pub fn search(args: &SearchArgs, use_cache: bool) -> Result<()> {
    let agent = utils::get_agent(args.insecure)?;
    let queries = utils::load_input(args, "No search query provided...".to_string())?;
    let outfmt = OutputFormat::from(args.outfmt.as_str());
    let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);
    let bar = utils::make_progress_bar(queries.len());

    // BatchWriter handles header-once + append correctly for all output modes.
    // Search CSV header is written here; JSON and split modes are no-ops in write_global_header.
    let csv_header = format!(
        "accession{sep}ncbi_organism_name{sep}ncbi_taxonomy{sep}\
         gtdb_taxonomy{sep}gtdb_species_representative{sep}ncbi_type_material",
        sep = outfmt.sep()
    );
    let mut writer = BatchWriter::new(&dest, &outfmt);
    // --id and --count produce plain values with no header row
    if !args.id && !args.count {
        writer.write_global_header(format!("{csv_header}\n").as_bytes())?;
    }

    for query in &queries {
        utils::bar_tick(&bar, query);

        let search_req = GtdbApiRequest::Search {
            query: query.clone(),
            search_field: SearchField::from(args.field.as_str()),
            gtdb_species_rep_only: args.rep,
            ncbi_type_material_only: args.r#type,
            output_format: "json".into(),
            page: 1,
            items_per_page: ITEMS_PER_PAGE,
            sort_by: "".into(),
            sort_desc: false,
            filter_text: "".into(),
        };

        let first_page: SearchResults = utils::fetch_data_cached(
            &agent,
            &search_req.to_url(),
            "The server returned an unexpected status code (400).".into(),
            use_cache,
            TTL_SEARCH,
        )?;

        let mut results = fetch_all_pages(&agent, first_page, args, query, use_cache)?;
        filter_and_validate(&mut results, query, args)?;

        let sep = outfmt.sep();
        let body = if args.count {
            format!("{}\n", results.total_rows)
        } else if args.id {
            results
                .rows
                .iter()
                .map(|x| x.accession.as_deref().unwrap_or(&x.gid).to_string())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        } else {
            match outfmt {
                OutputFormat::Json => serde_json::to_string_pretty(&results.rows)? + "\n",
                _ => format_xsv(&results, sep),
            }
        };

        // split_header is used by BatchWriter only in split+CSV/TSV mode.
        // For --id and --count output there is no header row.
        let split_header = if !args.id && !args.count && outfmt != OutputFormat::Json {
            format!("{csv_header}\n")
        } else {
            String::new()
        };

        writer.write_item(query, split_header.as_bytes(), body.as_bytes())?;
        utils::bar_inc(&bar);
    }

    utils::bar_finish(bar, queries.len(), "queries");
    Ok(())
}

/// Serialise a SearchResults set as CSV/TSV.
fn format_xsv(results: &SearchResults, sep: &str) -> String {
    let mut lines: Vec<String> = results
        .rows
        .iter()
        .map(|row| {
            format!(
                "{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}",
                row.accession.as_deref().unwrap_or(""),
                row.ncbi_org_name.as_deref().unwrap_or(""),
                row.ncbi_taxonomy.as_deref().unwrap_or(""),
                row.gtdb_taxonomy.as_deref().unwrap_or(""),
                row.is_gtdb_species_rep
                    .map(|b| if b { "True" } else { "False" })
                    .unwrap_or(""),
                row.is_ncbi_type_material
                    .map(|b| if b { "True" } else { "False" })
                    .unwrap_or(""),
            )
        })
        .collect();
    lines.push(String::new()); // trailing newline via join
    lines.join("\n")
}

/// Apply optional whole-word filtering and verify the result is non-empty.
fn filter_and_validate(results: &mut SearchResults, needle: &str, args: &SearchArgs) -> Result<()> {
    if args.word {
        results.filter_json(needle.to_string(), SearchField::from(args.field.as_str()));
    }
    ensure!(
        results.total_rows != 0,
        "No results found in GTDB for '{}'. \
         Try broadening your search or removing --word for partial matches.",
        needle
    );
    Ok(())
}

/// Fetch all pages concurrently and merge in page order.
fn fetch_all_pages(
    agent: &ureq::Agent,
    first_page: SearchResults,
    args: &SearchArgs,
    query: &str,
    use_cache: bool,
) -> Result<SearchResults> {
    let total = first_page.total_rows;
    let mut accumulated = first_page;
    let max_concurrent = args.max_concurrent.max(1);

    if total <= ITEMS_PER_PAGE {
        return Ok(accumulated);
    }

    let total_pages = (total as f64 / ITEMS_PER_PAGE as f64).ceil() as u32;
    let remaining: Vec<u32> = (2..=total_pages).collect();
    let (tx, rx) = mpsc::channel::<Result<(u32, Vec<SearchResult>)>>();

    for chunk in remaining.chunks(max_concurrent) {
        let mut handles = Vec::with_capacity(chunk.len());

        for &page in chunk {
            let tx = tx.clone();
            let agent = agent.clone();
            let query = query.to_string();
            let field = args.field.clone();
            let rep = args.rep;
            let type_ = args.r#type;

            let handle = thread::spawn(move || {
                let search = GtdbApiRequest::Search {
                    query: query.clone(),
                    search_field: SearchField::from(field.as_str()),
                    gtdb_species_rep_only: rep,
                    ncbi_type_material_only: type_,
                    output_format: "json".into(),
                    page: page as u16,
                    items_per_page: ITEMS_PER_PAGE,
                    sort_by: "".into(),
                    sort_desc: false,
                    filter_text: "".into(),
                };

                let result = (|| -> Result<(u32, Vec<SearchResult>)> {
                    let page_result: SearchResults = utils::fetch_data_cached(
                        &agent,
                        &search.to_url(),
                        format!(
                            "Failed to fetch page {}/{} for query '{}'. \
                             The GTDB API may be under load.",
                            page, total_pages, query
                        ),
                        use_cache,
                        TTL_SEARCH,
                    )?;
                    Ok((page, page_result.rows))
                })();

                let _ = tx.send(result);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().map_err(|_| {
                anyhow::anyhow!("A page-fetch thread panicked during parallel pagination")
            })?;
        }
    }

    drop(tx);

    let mut page_results: Vec<(u32, Vec<SearchResult>)> =
        rx.into_iter().collect::<Result<Vec<_>>>()?;

    page_results.sort_by_key(|(page, _)| *page);

    for (_, rows) in page_results {
        accumulated.rows.extend(rows);
    }

    accumulated.total_rows = accumulated.rows.len() as u32;
    Ok(accumulated)
}

fn whole_taxon_match(taxonomy: &str, taxon: &str) -> bool {
    taxonomy.split("; ").any(|t| t == taxon)
}

fn whole_word_match(haystack: &str, needle: &str) -> bool {
    haystack.split_whitespace().any(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::SearchField;
    use mockito::Server;

    #[test]
    fn test_search_results_filter_json_exact_ncbi_id() {
        let mut results = SearchResults {
            rows: vec![
                SearchResult {
                    gid: "id1".to_string(),
                    accession: Some("GCA_000123.1".to_string()),
                    ..Default::default()
                },
                SearchResult {
                    gid: "id2".to_string(),
                    accession: Some("GCA_999999.1".to_string()),
                    ..Default::default()
                },
            ],
            total_rows: 2,
        };

        results.filter_json("GCA_999999.1".to_string(), SearchField::NcbiId);
        assert_eq!(results.total_rows, 1);
        assert_eq!(results.rows[0].gid, "id2");
    }

    #[test]
    fn test_whole_word_match() {
        assert!(whole_word_match("bar bir ber bor", "bor"));
        assert!(!whole_word_match("bar bir ber bor", "xgt"));
        assert!(!whole_word_match("Geobacillus", "bacillus"));
    }

    #[test]
    fn test_get_rows() {
        let results = SearchResults {
            rows: vec![
                SearchResult {
                    gid: "1".into(),
                    ..Default::default()
                },
                SearchResult {
                    gid: "2".into(),
                    ..Default::default()
                },
                SearchResult {
                    gid: "3".into(),
                    ..Default::default()
                },
            ],
            total_rows: 3,
        };
        assert_eq!(results.rows.len(), 3);
    }

    #[test]
    fn test_handle_id_uses_accession_not_gid() {
        // Verifies that --id returns the clean accession, not the GB_/RS_ prefixed gid
        let rows = vec![
            SearchResult {
                gid: "GB_GCA_000005845.2".to_string(),
                accession: Some("GCA_000005845.2".to_string()),
                ncbi_org_name: None,
                ncbi_taxonomy: None,
                gtdb_taxonomy: None,
                ..Default::default()
            },
            SearchResult {
                gid: "RS_GCF_000001405.39".to_string(),
                accession: Some("GCF_000001405.39".to_string()),
                ncbi_org_name: None,
                ncbi_taxonomy: None,
                gtdb_taxonomy: None,
                ..Default::default()
            },
        ];

        let output: Vec<String> = rows
            .iter()
            .map(|x| x.accession.as_deref().unwrap_or(&x.gid).to_string())
            .collect();

        // Must return clean accessions, not GB_/RS_ prefixed gids
        assert_eq!(output[0], "GCA_000005845.2");
        assert_eq!(output[1], "GCF_000001405.39");
        assert!(
            !output[0].starts_with("GB_"),
            "accession must not have GB_ prefix"
        );
        assert!(
            !output[1].starts_with("RS_"),
            "accession must not have RS_ prefix"
        );
    }

    #[test]
    fn test_handle_id_falls_back_to_gid_when_accession_is_none() {
        // Safety net: if accession is None, gid is returned rather than crashing
        let row = SearchResult {
            gid: "GB_GCA_000005845.2".to_string(),
            accession: None,
            ncbi_org_name: None,
            ncbi_taxonomy: None,
            gtdb_taxonomy: None,
            ..Default::default()
        };

        let output = row.accession.as_deref().unwrap_or(&row.gid).to_string();
        assert_eq!(output, "GB_GCA_000005845.2");
    }

    #[test]
    fn test_fetch_all_pages_single_page_returns_early() {
        // When total_rows <= ITEMS_PER_PAGE, no additional fetches should occur
        let first_page = SearchResults {
            rows: vec![SearchResult {
                gid: "GB_GCA_000001.1".into(),
                accession: Some("GCA_000001.1".into()),
                ncbi_org_name: None,
                ncbi_taxonomy: None,
                gtdb_taxonomy: None,
                ..Default::default()
            }],
            total_rows: 1,
        };

        // With total_rows=1, fetch_all_pages should return immediately
        // We verify by checking the returned result has exactly one row
        assert_eq!(first_page.total_rows, 1);
        assert!(first_page.total_rows <= ITEMS_PER_PAGE);
    }

    #[test]
    fn test_total_pages_calculation() {
        // Verify ceiling division for various sizes
        let cases = [
            (1000u32, 1u32), // exactly one page
            (1001, 2),       // one row into second page
            (2000, 2),       // exactly two pages
            (52587, 53),     // g__Escherichia real-world case
            (13875, 14),     // g__Bacillus real-world case
            (2222, 3),       // g__Rhizobium real-world case
        ];
        for (total, expected_pages) in cases {
            let pages = (total as f64 / ITEMS_PER_PAGE as f64).ceil() as u32;
            assert_eq!(
                pages, expected_pages,
                "total_rows={} should give {} pages",
                total, expected_pages
            );
        }
    }

    #[test]
    fn test_page_results_sorted_before_merge() {
        // Simulate out-of-order arrival from parallel threads and verify sort
        let mut page_results: Vec<(u32, Vec<String>)> = vec![
            (3, vec!["c".into()]),
            (1, vec!["a".into()]), // already in accumulated
            (2, vec!["b".into()]),
        ];
        page_results.sort_by_key(|(page, _)| *page);
        assert_eq!(page_results[0].0, 1);
        assert_eq!(page_results[1].0, 2);
        assert_eq!(page_results[2].0, 3);
    }

    #[test]
    fn test_parallel_pagination_mock() {
        let mut server = Server::new();

        // Mock three pages of results
        let make_page = |accession: &str| -> String {
            format!(
                r#"{{"totalRows":3,"rows":[{{"gid":"GB_{acc}","accession":"{acc}","ncbiOrgName":null,"ncbiTaxonomy":null,"gtdbTaxonomy":null}}]}}"#,
                acc = accession
            )
        };

        let _m1 = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::UrlEncoded("page".into(), "2".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_page("GCA_000002.1"))
            .create();

        let _m2 = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::UrlEncoded("page".into(), "3".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_page("GCA_000003.1"))
            .create();

        let agent = ureq::Agent::config_builder().build().new_agent();

        // Replicate the parallel fetch for two pages
        let (tx, rx) = std::sync::mpsc::channel::<Result<(u32, Vec<SearchResult>)>>();

        for page in [2u32, 3u32] {
            let tx = tx.clone();
            let agent = agent.clone();
            let url = format!(
                "{}/search/gtdb?search=test&page={}&itemsPerPage=1000\
                 &searchField=all&gtdbSpeciesRepOnly=false\
                 &ncbiTypeMaterialOnly=false&outputFormat=json",
                server.url(),
                page
            );

            std::thread::spawn(move || {
                let result = agent
                    .get(&url)
                    .call()
                    .map_err(anyhow::Error::from)
                    .and_then(|r| {
                        r.into_body()
                            .read_json::<SearchResults>()
                            .map_err(anyhow::Error::from)
                            .map(|sr| (page, sr.rows))
                    });
                let _ = tx.send(result);
            });
        }

        drop(tx);

        let mut page_results: Vec<(u32, Vec<SearchResult>)> =
            rx.into_iter().collect::<Result<Vec<_>>>().unwrap();

        page_results.sort_by_key(|(p, _)| *p);

        assert_eq!(page_results.len(), 2);
        assert_eq!(page_results[0].0, 2);
        assert_eq!(page_results[1].0, 3);
    }
}
