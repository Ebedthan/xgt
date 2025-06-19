use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::api::search::SearchAPI;
use crate::cli;
use crate::utils::{self, is_valid_taxonomy, OutputFormat, SearchField};

const INTO_STRING_LIMIT: usize = 20 * 1_024 * 1_024;

/*----- GTDB API Search Result(s) structures and their methods -----*/
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
pub fn search(args: cli::search::SearchArgs) -> Result<()> {
    let agent = utils::get_agent(args.disable_certificate_verification())?;

    for needle in args.get_needles() {
        let search_api = SearchAPI::from(needle, &args);
        let request_url = search_api.request();

        let response = agent.get(&request_url).call().map_err(|e| match e {
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

        let output_result = if args.is_only_print_ids() || args.is_only_num_entries() {
            handle_id_or_count_response(response, needle, &args)
        } else {
            match args.get_outfmt() {
                OutputFormat::Json => handle_json_response(response, needle, &args),
                _ => handle_xsv_response(response, needle, &args),
            }
        };

        utils::write_to_output(output_result?.as_bytes(), args.get_output().clone())?;
    }

    Ok(())
}

// If -c or -i just use JSON output format to count entries or
// return ids list as converting using into_string can
// throw an error of too big to convert to string especially
// when querying data related to large genus like Escherichia
// See cli/search.rs#L166-L178
fn handle_id_or_count_response(
    response: ureq::Response,
    needle: &str,
    args: &cli::search::SearchArgs,
) -> Result<String> {
    process_response(response, needle, args, |search_result| {
        if args.is_only_num_entries() {
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
    response: ureq::Response,
    needle: &str,
    args: &cli::search::SearchArgs,
    format_fn: F,
) -> Result<String>
where
    F: FnOnce(&SearchResults) -> Result<String>,
{
    let mut search_result: SearchResults = response.into_json()?;
    if args.is_whole_words_matching() {
        search_result.filter_json(needle.to_string(), args.get_search_field());
    }
    ensure!(
        search_result.get_total_rows() != 0,
        "No matching data found in GTDB"
    );
    format_fn(&search_result)
}

fn handle_json_response(
    response: ureq::Response,
    needle: &str,
    args: &cli::search::SearchArgs,
) -> Result<String> {
    process_response(response, needle, args, |search_result| {
        serde_json::to_string_pretty(&search_result.rows).map_err(Into::into)
    })
}

fn handle_xsv_response(
    response: ureq::Response,
    needle: &str,
    args: &cli::search::SearchArgs,
) -> Result<String> {
    process_xsv_response(response, needle, args, |result, needle| {
        filter_xsv(result, needle, args.get_search_field(), args.get_outfmt());
    })
}

fn process_xsv_response<F>(
    response: ureq::Response,
    needle: &str,
    args: &cli::search::SearchArgs,
    process_fn: F,
) -> Result<String>
where
    F: FnOnce(&mut String, &str),
{
    let mut buf: Vec<u8> = vec![];
    response
        .into_reader()
        .take((INTO_STRING_LIMIT + 1) as u64)
        .read_to_end(&mut buf)?;
    if buf.len() > INTO_STRING_LIMIT {
        return Err(anyhow!("GTDB response is too big (> 20 MB) to convert to string. Please use JSON output format (-O json)"));
    }
    let mut result = String::from_utf8_lossy(&buf).to_string();

    if args.is_whole_words_matching() {
        process_fn(&mut result, needle);
    }
    Ok(result)
}

/// Filter CSV/TSV API query result by search field value
fn filter_xsv(result: &mut String, needle: &str, search_field: SearchField, outfmt: OutputFormat) {
    // Move content out of `result` to avoid borrowing issues
    let content = std::mem::take(result);

    // Split the content into lines and parse the header
    let mut lines = content.lines();

    // Check presence of CSV/TSV header
    let header = lines.next().expect("Input should have a header");

    let split_pat = if outfmt == OutputFormat::Csv {
        ","
    } else {
        "\t"
    };

    // Filter lines based on the determined matcher
    let filtered_lines: Vec<&str> = if search_field == SearchField::All {
        lines
            .filter(|line| {
                let fields: Vec<&str> = line.split(split_pat).collect();
                all_match(fields, needle)
            })
            .collect()
    } else {
        // Get the CSV/TSV column which will be subjected to filtering
        let sfield = match search_field {
            SearchField::NcbiId => "accession".to_string(),
            SearchField::NcbiOrg => "ncbi_organism_name".to_string(),
            SearchField::NcbiTax => "ncbi_taxonomy".to_string(),
            _ => "gtdb_taxonomy".to_string(),
        };
        let headers: Vec<&str> = header.split(split_pat).collect();
        let index = headers.iter().position(|&field| field == sfield);
        if index.is_none() {
            std::io::stdout()
                .write_all(b"Warning: missing header in the output")
                .unwrap();
        }
        lines
            .filter(|line| {
                let fields: Vec<&str> = line.split(split_pat).collect();
                if let Some(idx) = index {
                    if let Some(field) = fields.get(idx) {
                        return if is_valid_taxonomy(field) {
                            println!(
                                "Field: {}, Needle: {}, Result: {}",
                                field,
                                needle,
                                whole_taxon_match(field, needle)
                            );
                            whole_taxon_match(field, needle)
                        } else {
                            whole_word_match(field, needle)
                        };
                    }
                }
                false
            })
            .collect()
    };

    // Modify the original result string
    result.clear();
    result.push_str(header);
    result.push_str("\r\n");
    for line in filtered_lines {
        result.push_str(line);
        result.push_str("\r\n");
    }
}

/// Perform a match on all `SearchResult` fields
/// # Example
/// ```
/// let input = ["GCA00000.1", "org name", "d__d1; p__p1; c__c1; o__o1; f__f1; g__g1; s__s1", "d__d2; p__p2; c__c2; o__o2; f__f2; g__g2; s__s2"];
/// assert!(all_match(input, "d__d1"));
/// assert!(all_match(input, "org name"));
/// assert!(!all_match(input, "xgt"));
/// ```
fn all_match(haystack: Vec<&str>, needle: &str) -> bool {
    haystack
        .iter()
        .take(4)
        .any(|field| whole_word_match(field, needle) || whole_taxon_match(field, needle))
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
    use crate::search::SearchResult;
    use crate::utils::SearchField;
    use cli::search::SearchArgs;
    use ureq::Response;

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
    fn test_filter_xsv_csv_accession_field() {
        let mut input =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\nGCA_000020265.1,Rhizobium etli CIAT 652,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium etli,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium phaseoli,False,True".to_string();
        let needle = "GCA_000016265.1";
        let search_field = SearchField::NcbiId;
        let outfmt = OutputFormat::Csv;

        let expected_output =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\n".to_string();
        filter_xsv(&mut input, needle, search_field, outfmt);

        assert_eq!(input, expected_output);
    }

    #[test]
    fn test_filter_xsv_csv_all_fields() {
        let mut input =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\nGCA_000020265.1,Rhizobium etli CIAT 652,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium etli,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium phaseoli,False,True".to_string();
        let needle = "Agrobacterium";
        let search_field = SearchField::All;
        let outfmt = OutputFormat::Csv;

        let expected_output =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\n".to_string();
        filter_xsv(&mut input, needle, search_field, outfmt);

        assert_eq!(input, expected_output);
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
    fn test_search_id() {
        let mut args = SearchArgs::new();
        args.add_needle("g__Azorhizobium");
        args.set_id(true);
        args.set_output(Some("test3.txt".to_string()));
        args.set_outfmt("json".to_string());
        args.set_disable_certificate_verification(true);
        let res = search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test3.txt").unwrap();
        assert_eq!(
            r#"GCA_002279595.1
GCA_002280795.1
GCA_002280945.1
GCA_002281175.1
GCA_002282175.1
GCA_023405075.1
GCA_023448105.1
GCF_000010525.1
GCF_000473085.1
GCF_004364705.1
GCF_014635325.1
GCF_036600855.1
GCF_036600875.1
GCF_036600895.1
GCF_036600915.1
GCF_943371865.1"#
                .to_string(),
            expected
        );
        std::fs::remove_file("test3.txt").unwrap();
    }

    #[test]
    fn test_partial_search_count() {
        let mut args = cli::search::SearchArgs::new();
        args.add_needle("g__Azorhizobium");
        args.set_count(true);
        args.set_disable_certificate_verification(true);
        args.set_output(Some("test.txt".to_string()));
        args.set_outfmt("json".to_string());
        let res = search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("16".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_all_match() {
        let line = "GCA_001512625.1,Clostridiales bacterium DTU036,d__Bacteria; p__Bacillota; c__Clostridia; o__Eubacteriales; f__; g__; s__,d__Bacteria; p__Bacillota_A; c__Clostridia; o__Peptostreptococcales; f__Acidaminobacteraceae; g__DTU036; s__DTU036 sp001512625,True,False";
        let fields: Vec<&str> = line.split(",").collect();
        assert!(all_match(fields, "c__Clostridia"));
    }

    // Dummy ureq::Response-like type
    struct MockResponse {
        body: Vec<u8>,
    }

    impl MockResponse {
        fn new_from_str(s: &str) -> Self {
            Self {
                body: s.as_bytes().to_vec(),
            }
        }

        fn to_ureq_response(self) -> Response {
            // `ureq::Response` is not mockable directly; simulate using `ureq::Response::into_reader()`
            ureq::Response::new(200, "OK", std::str::from_utf8(&self.body).unwrap()).unwrap()
        }
    }

    #[test]
    fn test_process_xsv_response_too_large() {
        let big_str = "a".repeat(INTO_STRING_LIMIT + 1);
        let response = MockResponse::new_from_str(&big_str).to_ureq_response();

        let args = cli::search::SearchArgs {
            is_whole_words_matching: true,
            ..Default::default()
        };

        let result = process_xsv_response(response, "ACC123", &args, |_, _| {});
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("GTDB response is too big"));
    }
}
