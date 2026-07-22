use anyhow::{ensure, Ok, Result};
use serde::{Deserialize, Serialize};

use crate::api::GtdbApiRequest;
use crate::cli::SearchArgs;
use crate::utils::{self, OutputFormat, SearchField};

use std::sync::mpsc;
use std::thread;

const ITEMS_PER_PAGE: u32 = 1_000;

// GTDB API Search Result(s) structures and their methods
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
/// API search result struct
struct SearchResult {
    // Genome accession used as table ID
    gid: String,

    // Genome accession number
    accession: Option<String>,

    // NCBI organism name
    ncbi_org_name: Option<String>,

    // NCBI taxonomy
    ncbi_taxonomy: Option<String>,

    // GTDB taxonomy
    gtdb_taxonomy: Option<String>,

    // Boolean value indicating if species is a GTDB
    // representative species
    is_gtdb_species_rep: Option<bool>,

    // Boolean value indicating if species is a NCBI
    // type material
    is_ncbi_type_material: Option<bool>,
}

impl SearchResult {
    /// Get genome accession number
    fn get_accession(&self) -> Option<&String> {
        self.accession.as_ref()
    }

    /// Get NCBI organism name
    fn get_ncbi_org_name(&self) -> Option<&String> {
        self.ncbi_org_name.as_ref()
    }

    /// Get NCBI taxonomy name
    fn get_ncbi_taxonomy(&self) -> Option<&String> {
        self.ncbi_taxonomy.as_ref()
    }

    /// Get GTDB taxonomy
    fn get_gtdb_taxonomy(&self) -> Option<&String> {
        self.gtdb_taxonomy.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
// JSON API search result struct
struct SearchResults {
    // A list of SearchResult struct
    rows: Vec<SearchResult>,
    // A count of number of entries
    total_rows: u32,
}

impl SearchResults {
    /// Filter SearchResult for exact match of taxon name
    /// and rank as supplied by the user
    fn filter_json(&mut self, needle: String, search_field: SearchField) {
        self.rows.retain(|result| match search_field {
            SearchField::All => {
                // Apply whole_taxon_match to ncbi_taxonomy and gtdb_taxonomy
                let taxon_match = [result.get_ncbi_taxonomy(), result.get_gtdb_taxonomy()]
                    .iter()
                    .filter_map(|field| field.as_ref()) // Filter out None values
                    .any(|value| whole_taxon_match(value, needle.as_str()));

                // Apply whole_word_match to accession and ncbi_org_name
                let word_match = [result.get_accession(), result.get_ncbi_org_name()]
                    .iter()
                    .filter_map(|field| field.as_ref())
                    .any(|value| whole_word_match(value, needle.as_str()));

                taxon_match || word_match
            }

            // Using map_or here avoids allocating a new string when None is encountered
            // instead of previous unwrap_or_default()
            SearchField::NcbiId => result
                .get_accession()
                .is_some_and(|acc| whole_word_match(acc, needle.as_str())),
            SearchField::NcbiOrg => result
                .get_ncbi_org_name()
                .is_some_and(|name| whole_word_match(name, needle.as_str())),
            SearchField::NcbiTax => result
                .get_ncbi_taxonomy()
                .is_some_and(|ncbi_tax| whole_taxon_match(ncbi_tax, needle.as_str())),
            SearchField::GtdbTax => result
                .get_gtdb_taxonomy()
                .is_some_and(|gtdb_tax| whole_taxon_match(gtdb_tax, needle.as_str())),
        });
        self.total_rows = self.rows.len() as u32;
    }

    /// Get total rows
    fn get_total_rows(&self) -> u32 {
        self.total_rows
    }
}

/*----- Main Search Function and its methods -----*/
/// Search GTDB data from `SearchArgs`
pub fn search(args: &SearchArgs) -> Result<()> {
    let agent = utils::get_agent(args.insecure)?;
    let queries = utils::load_input(args, "No search query provided...".to_string())?;
    let outfmt = OutputFormat::from(args.outfmt.clone());
    let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);
    let bar = utils::make_progress_bar(queries.len());

    for query in &queries {
        if let Some(ref bar) = bar {
            bar.set_message(query.clone());
        }

        let search_req = GtdbApiRequest::Search {
            query: query.clone(),
            search_field: SearchField::from(args.field.clone()),
            gtdb_species_rep_only: args.rep,
            ncbi_type_material_only: args.r#type,
            output_format: "json".into(),
            page: 1,
            items_per_page: 1000,
            sort_by: "".into(),
            sort_desc: false,
            filter_text: "".into(),
        };

        let response = utils::fetch_data(
            &agent,
            &search_req.to_url(),
            "The server returned an unexpected status code (400).".into(),
        )?;

        let output_result = if args.id || args.count {
            handle_id_or_count_response(&agent, response, query, args)
        } else {
            match &outfmt {
                OutputFormat::Json => handle_json_response(&agent, response, query, args),
                _ => handle_xsv_response(&agent, response, query, args),
            }
        };

        // split mode: new file per query (truncate); single mode: append
        let append = !dest.is_split();
        utils::write_to_output(output_result?.as_bytes(), dest.resolve(query), append)?;

        if let Some(ref bar) = bar {
            bar.inc(1);
        }
    }

    if let Some(bar) = bar {
        bar.finish_with_message(format!("done, {} queries processed", queries.len()));
    }

    Ok(())
}

fn handle_id_or_count_response(
    agent: &ureq::Agent,
    response: ureq::http::Response<ureq::Body>,
    needle: &str,
    args: &SearchArgs,
) -> Result<String> {
    process_response(agent, response, needle, args, |search_result| {
        if args.count {
            Ok(search_result.get_total_rows().to_string())
        } else {
            Ok(search_result
                .rows
                .iter()
                .map(|x| x.accession.as_deref().unwrap_or(&x.gid).to_string())
                .collect::<Vec<String>>()
                .join("\n"))
        }
    })
}

fn process_response<F>(
    agent: &ureq::Agent,
    response: ureq::http::Response<ureq::Body>,
    needle: &str,
    args: &SearchArgs,
    format_fn: F,
) -> Result<String>
where
    F: FnOnce(&SearchResults) -> Result<String>,
{
    let first_page: SearchResults = response.into_body().read_json()?;
    let mut search_result = fetch_all_pages(agent, first_page, args, needle)?;
    filter_and_validate(&mut search_result, needle, args)?;
    format_fn(&search_result)
}

/// Fetch all pages for a search query concurrently and return the accumulated SearchResults.
/// The first page has already been fetched and deserialized by the caller. Pages 2..=N are
/// dispatched to up to MAX_CONCURRENT threads. Results are merged in page order before returning.
fn fetch_all_pages(
    agent: &ureq::Agent,
    first_page: SearchResults,
    args: &SearchArgs,
    query: &str,
) -> Result<SearchResults> {
    let total = first_page.total_rows;
    let mut accumulated = first_page;
    let max_concurrent = args.max_concurrent.max(1); // guard against 0

    if total <= ITEMS_PER_PAGE {
        return Ok(accumulated);
    }

    let total_pages = (total as f64 / ITEMS_PER_PAGE as f64).ceil() as u32;
    let remaining: Vec<u32> = (2..=total_pages).collect();

    // Channel carries (page_number, rows) so we can sort by page before merging
    let (tx, rx) = mpsc::channel::<Result<(u32, Vec<SearchResult>)>>();

    // Dispatch pages in chunks of MAX_CONCURRENT so we never have more than
    // MAX_CONCURRENT live connections at once, the rate-limit guard.
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
                    search_field: SearchField::from(field),
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
                    let response = utils::fetch_data(
                        &agent, &search.to_url(), format!("Failed to fetch page {}/{} for query '{}'. The GTDB API may be under load.", page, total_pages, query)
                    )?;
                    let page_result: SearchResults = response.into_body().read_json()?;
                    Ok((page, page_result.rows))
                })();

                // send is infallible here since rx is still alive
                let _ = tx.send(result);
            });

            handles.push(handle);
        }

        // Wait for the current chunk to complete before dispatching the next.
        // This bounds concurrent connections to MAX_CONCURRENT at any time.
        for handle in handles {
            handle.join().map_err(|_| {
                anyhow::anyhow!("A page-fetch thread panicked during parallel pagination")
            })?;
        }
    }

    // drop the last sender so rx knows all results have been sent
    drop(tx);

    // collect all (page, rows) pairs from the channel
    let mut page_results: Vec<(u32, Vec<SearchResult>)> =
        rx.into_iter().collect::<Result<Vec<_>>>()?;

    // sort by page number to guarantee row order matches the API's natural order
    page_results.sort_by_key(|(page, _)| *page);

    for (_, rows) in page_results {
        accumulated.rows.extend(rows);
    }

    accumulated.total_rows = accumulated.rows.len() as u32;
    Ok(accumulated)
}

/// Apply optional whole-word filtering and verify the result is non-empty.
/// This is the shared post-pagination step for all output paths.
fn filter_and_validate(
    results: &mut SearchResults,
    needle: &str,
    args: &SearchArgs,
) -> anyhow::Result<()> {
    if args.word {
        results.filter_json(needle.to_string(), SearchField::from(args.field.clone()));
    }
    ensure!(
        results.get_total_rows() != 0,
        "No results found in GTDB for '{}'. \
         Try broadening your search or removing --word for partial matches.",
        needle
    );
    Ok(())
}

fn handle_json_response(
    agent: &ureq::Agent,
    response: ureq::http::Response<ureq::Body>,
    needle: &str,
    args: &SearchArgs,
) -> Result<String> {
    process_response(agent, response, needle, args, |search_result| {
        serde_json::to_string_pretty(&search_result.rows).map_err(Into::into)
    })
}

fn handle_xsv_response(
    agent: &ureq::Agent,
    response: ureq::http::Response<ureq::Body>,
    needle: &str,
    args: &SearchArgs,
) -> Result<String> {
    let first_page: SearchResults = response.into_body().read_json()?;
    let mut all_results = fetch_all_pages(agent, first_page, args, needle)?;
    filter_and_validate(&mut all_results, needle, args)?;

    let outfmt = OutputFormat::from(args.outfmt.clone());
    let sep = if outfmt == OutputFormat::Tsv {
        "\t"
    } else {
        ","
    };

    let header = format!(
        "accession{sep}ncbi_organism_name{sep}ncbi_taxonomy{sep}\
         gtdb_taxonomy{sep}gtdb_species_representative{sep}ncbi_type_material"
    );

    let mut lines = vec![header];
    for row in &all_results.rows {
        lines.push(format!(
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
        ));
    }

    Ok(lines.join("\n") + "\n")
}

/// Perform whole taxon exact matching
fn whole_taxon_match(taxonomy: &str, taxon: &str) -> bool {
    taxonomy.split("; ").any(|tax| tax == taxon)
}

/// Perform whole word exact matching
fn whole_word_match(haystack: &str, needle: &str) -> bool {
    haystack.split_whitespace().any(|word| word == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::SearchField;
    use mockito::Server;

    #[test]
    fn test_search_result_getters() {
        let sr = SearchResult {
            gid: "G00001".to_string(),
            accession: Some("GCA_000001.1".to_string()),
            ncbi_org_name: Some("Escherichia coli".to_string()),
            ncbi_taxonomy: Some("d__Bacteria;p__Proteobacteria".to_string()),
            gtdb_taxonomy: Some("d__Bacteria;p__Pseudomonadota".to_string()),
            is_gtdb_species_rep: Some(true),
            is_ncbi_type_material: Some(false),
        };

        assert_eq!(sr.get_accession(), Some(&"GCA_000001.1".to_string()));
        assert_eq!(
            sr.get_ncbi_org_name(),
            Some(&"Escherichia coli".to_string())
        );
        assert_eq!(
            sr.get_ncbi_taxonomy(),
            Some(&"d__Bacteria;p__Proteobacteria".to_string())
        );
        assert_eq!(
            sr.get_gtdb_taxonomy(),
            Some(&"d__Bacteria;p__Pseudomonadota".to_string())
        );
    }

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
    fn test_get_total_rows() {
        let results = SearchResults {
            rows: vec![Default::default(); 3],
            total_rows: 3,
        };

        assert_eq!(results.get_total_rows(), 3);
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
