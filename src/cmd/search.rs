use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::api::GtdbApiRequest;
use crate::cli::SearchArgs;
use crate::utils::{self, OutputFormat, SearchField};

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
    /// # Example
    /// ```
    /// let search_result = SearchResult::default();
    /// assert_eq!(search_result.get_accession(), None);
    /// ```
    fn get_accession(&self) -> Option<&String> {
        self.accession.as_ref()
    }

    /// Get NCBI organism name
    /// # Example
    /// ```
    /// let search_result = SearchResult::default();
    /// assert_eq!(search_result.get_ncbi_org_name(), None);
    /// ```
    fn get_ncbi_org_name(&self) -> Option<&String> {
        self.ncbi_org_name.as_ref()
    }

    /// Get NCBI taxonomy name
    /// # Example
    /// ```
    /// let search_result = SearchResult::default();
    /// assert_eq!(search_result.get_ncbi_taxonomy(), None);
    /// ```
    fn get_ncbi_taxonomy(&self) -> Option<&String> {
        self.ncbi_taxonomy.as_ref()
    }

    /// Get GTDB taxonomy
    /// # Example
    /// ```
    /// let search_result = SearchResult::default();
    /// assert_eq!(search_result.get_gtdb_taxonomy(), None);
    /// ```
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
    /// # Example
    /// ```
    /// let search_results = SearchResults::default();
    /// assert_eq!(search_results.get_total_rows(), 0_u32);
    /// ```
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
            search_field: args.field.clone(),
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
                .map(|x| x.gid.clone())
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

/// Fetch all pages for a search query and return the accumulated SearchResults.
/// The first response has already been fetched and deserialized; subsequent pages
/// are fetched using the agent and the same search parameters, incrementing page number.
fn fetch_all_pages(
    agent: &ureq::Agent,
    first_page: SearchResults,
    args: &SearchArgs,
    query: &str,
) -> Result<SearchResults> {
    const ITEMS_PER_PAGE: u32 = 1000;

    let total = first_page.total_rows;
    let mut accumulated = first_page;

    if total <= ITEMS_PER_PAGE {
        return Ok(accumulated);
    }

    let total_pages = (total as f64 / ITEMS_PER_PAGE as f64).ceil() as u16;

    for page in 2..=total_pages {
        let search = GtdbApiRequest::Search {
            query: query.to_string(),
            search_field: args.field.clone(),
            gtdb_species_rep_only: args.rep,
            ncbi_type_material_only: args.r#type,
            output_format: "json".into(), // always JSON for pagination
            page,
            items_per_page: ITEMS_PER_PAGE,
            sort_by: "".into(),
            sort_desc: false,
            filter_text: "".into(),
        };

        let response = utils::fetch_data(
            agent,
            &search.to_url(),
            format!(
                "Failed to fetch page {}/{} for query '{}'. The GTDB API may be under load.",
                page, total_pages, query
            ),
        )?;

        let page_result: SearchResults = response.into_body().read_json()?;
        accumulated.rows.extend(page_result.rows);
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
/// # Example
/// ```
/// assert!(whole_taxon_match("d__domain; p__phylum; c__class; o__order; f__family; g__genus; s__species", "d__domain"));
/// assert!(!whole_taxon_match("d__domain; p__phylum; c__class; o__order; f__family; g__genus; s__species", "xgt"));
/// ```
fn whole_taxon_match(taxonomy: &str, taxon: &str) -> bool {
    taxonomy.split("; ").any(|tax| tax == taxon)
}

/// Perform whole word exact matching
/// # Example
/// ```
/// assert!(whole_word_match("bar bir ber bor", "bor"));
/// assert!(!whole_word_match("bar bir ber bor", "xgt"));
/// ```
fn whole_word_match(haystack: &str, needle: &str) -> bool {
    haystack.split_whitespace().any(|word| word == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::SearchArgs;
    use crate::utils::SearchField;

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
    fn test_partial_search_count() {
        let args = SearchArgs {
            query: Some("g__Azorhizobium".to_string()),
            word: false,
            field: String::from("all"),
            rep: false,
            r#type: false,
            id: false,
            count: true,
            file: None,
            outfmt: String::from("json"),
            out: Some("test.txt".to_string()),
            insecure: true,
            split: false,
            split_dir: None,
            release: None,
        };
        let res = search(&args);
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("23".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }
}
