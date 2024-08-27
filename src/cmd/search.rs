use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};
use std::io::Read;

use crate::api::search::SearchAPI;
use crate::cli;
use crate::utils::{self, is_taxonomy_field, OutputFormat, SearchField};

const INTO_STRING_LIMIT: usize = 20 * 1_024 * 1_024;

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
    fn get_accession(&self) -> Option<String> {
        self.accession.clone()
    }

    /// Get NCBI organism name
    /// # Example
    /// ```
    /// let search_result = SearchResult::default();
    /// assert_eq!(search_result.get_ncbi_org_name(), None);
    /// ```
    fn get_ncbi_org_name(&self) -> Option<String> {
        self.ncbi_org_name.clone()
    }

    /// Get NCBI taxonomy name
    /// # Example
    /// ```
    /// let search_result = SearchResult::default();
    /// assert_eq!(search_result.get_ncbi_taxonomy(), None);
    /// ```
    fn get_ncbi_taxonomy(&self) -> Option<String> {
        self.ncbi_taxonomy.clone()
    }

    /// Get GTDB taxonomy
    /// # Example
    /// ```
    /// let search_result = SearchResult::default();
    /// assert_eq!(search_result.get_gtdb_taxonomy(), None);
    /// ```
    fn get_gtdb_taxonomy(&self) -> Option<String> {
        self.gtdb_taxonomy.clone()
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
            SearchField::All => [
                result.get_accession(),
                result.get_ncbi_org_name(),
                result.get_ncbi_taxonomy(),
                result.get_gtdb_taxonomy(),
            ]
            .iter()
            .all(|field| match field {
                Some(value) => value == &needle,
                None => false,
            }),
            SearchField::Acc => result.get_accession() == Some(needle.clone()),
            SearchField::Org => result.get_ncbi_org_name() == Some(needle.clone()),
            SearchField::Ncbi => result.get_ncbi_taxonomy() == Some(needle.clone()),
            SearchField::Gtdb => result.get_gtdb_taxonomy() == Some(needle.clone()),
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

/// Perform whole word exact matching
/// # Example
/// ```
/// assert!(whole_word_match("bar bir ber bor", "bor"));
/// assert!(!whole_word_match("bar bir ber bor", "xgt"));
/// ```
fn whole_word_match(haystack: &str, needle: &str) -> bool {
    haystack.split_whitespace().any(|word| word == needle)
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

/// Perform a match on all `SearchResult` fields
/// # Example
/// ```
/// let input = ["GCA00000.1", "org name", "d__d1; p__p1; c__c1; o__o1; f__f1; g__g1; s__s1", "d__d2; p__p2; c__c2; o__o2; f__f2; g__g2; s__s2"];
/// assert!(all_match(input, "d__d1"));
/// assert!(all_match(input, "org name"));
/// assert!(!all_match(input, "xgt"));
/// ```
fn all_match(haystack: Vec<&str>, needle: &str) -> bool {
    whole_word_match(haystack[0], needle) // Check word match in accession field
        || whole_word_match(haystack[1], needle) // Check word match in ncbi_org_name field
        || whole_taxon_match(haystack[2], needle) // Check word match in gtdb_taxonomy field
        || whole_taxon_match(haystack[3], needle) // Check word match in ncbi_taxonomy field
}

/// Filter CSV/TSV API query result by search field value
fn filter_xsv(
    result: String,
    needle: &str,
    search_field: SearchField,
    outfmt: OutputFormat,
) -> String {
    let split_pat = if outfmt == OutputFormat::Csv {
        ","
    } else {
        "\t"
    };
    let sfield = match search_field {
        SearchField::Acc => "accession".to_string(),
        SearchField::Org => "ncbi_organism_name".to_string(),
        SearchField::Ncbi => "ncbi_taxonomy".to_string(),
        _ => "gtdb_taxonomy".to_string(),
    };

    // Split the content into lines and parse the header
    let mut lines = result.trim_end().split("\r\n");

    let header = lines.next().expect("Input should have a header");

    // Determine the matching function based on the search field
    let matcher: Box<dyn Fn(&str) -> bool> = match search_field {
        // Dummy matcher for All, real logic is in all_match
        SearchField::All => Box::new(|_| false),
        _ => {
            if is_taxonomy_field(&search_field) {
                Box::new(|field| whole_taxon_match(field, needle))
            } else {
                Box::new(|field| whole_word_match(field, needle))
            }
        }
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
        let headers: Vec<&str> = header.split(split_pat).collect();
        let index = headers
            .iter()
            .position(|&field| field == sfield)
            .unwrap_or_else(|| panic!("{sfield} field not found in header"));
        lines
            .filter(|line| {
                let fields: Vec<&str> = line.split(split_pat).collect();
                fields.get(index).map_or(false, |&field| matcher(field))
            })
            .collect()
    };

    // Construct the final output
    let mut output = String::with_capacity(result.len());
    output.push_str(header);
    output.push_str("\r\n");
    for line in filtered_lines {
        output.push_str(line);
        output.push_str("\r\n");
    }

    output
}

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
                anyhow::anyhow!("There was an error making the request or receiving the response.")
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
    let mut search_result: SearchResults = response.into_json()?;
    if args.is_whole_words_matching() {
        search_result.filter_json(needle.to_string(), args.get_search_field());
    }

    ensure!(
        search_result.get_total_rows() != 0,
        "No matching data found in GTDB"
    );

    let result_str = if args.is_only_num_entries() {
        search_result.get_total_rows().to_string()
    } else {
        search_result
            .rows
            .iter()
            .map(|x| x.gid.clone())
            .collect::<Vec<String>>()
            .join("\n")
    };

    Ok(result_str)
}

fn handle_json_response(
    response: ureq::Response,
    needle: &str,
    args: &cli::search::SearchArgs,
) -> Result<String> {
    let mut search_result: SearchResults = response.into_json()?;
    if args.is_whole_words_matching() {
        search_result.filter_json(needle.to_string(), args.get_search_field());
    }

    ensure!(
        search_result.get_total_rows() != 0,
        "No matching data found in GTDB"
    );

    let result_str = search_result
        .rows
        .iter()
        .map(|x| serde_json::to_string_pretty(x).unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    Ok(result_str)
}

fn handle_xsv_response(
    response: ureq::Response,
    needle: &str,
    args: &cli::search::SearchArgs,
) -> Result<String> {
    let mut buf: Vec<u8> = vec![];
    response
        .into_reader()
        .take((INTO_STRING_LIMIT + 1) as u64)
        .read_to_end(&mut buf)?;
    if buf.len() > INTO_STRING_LIMIT {
        return Err(anyhow!("GTDB response is too big (> 20 MB) to convert to string. Please use JSON output format (-O json)"));
    }
    let result = String::from_utf8_lossy(&buf).to_string();
    if args.is_whole_words_matching() {
        filter_xsv(
            result.clone(),
            needle,
            args.get_search_field(),
            args.get_outfmt(),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_xsv_csv_accession_field() {
        let input =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\nGCA_000020265.1,Rhizobium etli CIAT 652,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium etli,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium phaseoli,False,True".to_string();
        let needle = "GCA_000016265.1";
        let search_field = SearchField::Acc;
        let outfmt = OutputFormat::Csv;

        let expected_output =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\n".to_string();
        let result = filter_xsv(input, needle, search_field, outfmt);

        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_filter_xsv_csv_all_fields() {
        let input =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\nGCA_000020265.1,Rhizobium etli CIAT 652,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium etli,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium phaseoli,False,True".to_string();
        let needle = "Agrobacterium";
        let search_field = SearchField::All;
        let outfmt = OutputFormat::Csv;

        let expected_output =
                "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_000016265.1,Agrobacterium radiobacter K84,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Rhizobiaceae; g__Agrobacterium; s__Agrobacterium tumefaciens,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__Rhizobium rhizogenes,False,True\r\n".to_string();
        let result = filter_xsv(input, needle, search_field, outfmt);

        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_get_total_rows() {
        let results = SearchResults {
            rows: vec![
                SearchResult::default(),
                SearchResult::default(),
                SearchResult::default(),
            ],
            total_rows: 3,
        };
        assert_eq!(results.get_total_rows(), 3);
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
        let mut args = cli::search::SearchArgs::new();
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
GCF_014635325.1"#
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
        assert_eq!("11".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }
}
