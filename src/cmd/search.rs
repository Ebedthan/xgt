use std::collections::HashMap;

use anyhow::{anyhow, bail, ensure, Result};
use serde::{Deserialize, Serialize};
use ureq::Agent;

use super::utils::{self, OutputFormat, SearchField};

use crate::api::search_api::SearchAPI;

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

impl SearchResult {
    fn get_ncbi_taxonomy(&self) -> Option<String> {
        self.ncbi_taxonomy.clone()
    }
    fn get_gtdb_taxonomy(&self) -> Option<String> {
        self.gtdb_taxonomy.clone()
    }
    fn get_taxon_by_rank(&self, rank: &str) -> Result<String> {
        let tax = self.gtdb_taxonomy.clone().ok_or_else(|| {
            anyhow!("Failed to perform exact match as gtdb taxonomy is a null field")
        })?;

        let index = match rank {
            "d" => 0,
            "p" => 1,
            "c" => 2,
            "o" => 3,
            "f" => 4,
            "g" => 5,
            "s" => 6,
            _ => return Err(anyhow!("Invalid level specified")),
        };

        let res = tax
            .split("; ")
            .nth(index)
            .unwrap_or("")
            .trim_start_matches(char::is_whitespace)
            .to_string();

        Ok(res)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SearchResults {
    rows: Vec<SearchResult>,
    total_rows: u32,
}

impl SearchResults {
    /// Filter SearchResult for exact match of taxon name
    /// and rank as supplied by the user
    fn filter_json(&mut self, needle: String, search_field: SearchField) {
        match search_field {
            SearchField::NCBI => {
                self.rows
                    .retain(|result| result.get_ncbi_taxonomy().unwrap().contains(&needle));
                self.total_rows = self.rows.len() as u32;
            }
            SearchField::GTDB => {
                self.rows
                    .retain(|result| result.get_gtdb_taxonomy().unwrap().contains(&needle));
                self.total_rows = self.rows.len() as u32;
            }
        }
    }
    fn get_total_rows(&self) -> u32 {
        self.total_rows
    }
}

fn whole_taxon_rank_match(haystack: &str, needle: &str) -> bool {
    haystack
        .split("; ")
        .collect::<Vec<&str>>()
        .iter()
        .any(|word| word.split("__").nth(1).unwrap_or("") == needle)
}

fn filter_xsv(
    result: String,
    needle: &str,
    search_field: SearchField,
    outfmt: OutputFormat,
) -> String {
    let split_pat = if outfmt == OutputFormat::CSV {
        ","
    } else {
        "\t"
    };
    let sfield = if search_field == SearchField::NCBI {
        "ncbi_taxonomy"
    } else {
        "gtdb_taxonomy"
    };

    // Split the content into lines and parse the header
    let mut lines = result.trim_end().split("\r\n");
    let header = lines.next().expect("Input should have a header");

    let headers: Vec<&str> = header.split(split_pat).collect();
    let taxonomy_index = headers
        .iter()
        .position(|&field| field == sfield)
        .expect(format!("{sfield} field not found in header").as_str());

    // Filter lines based on the taxonomy field
    let filtered_lines: Vec<&str> = lines
        .filter(|line| {
            let fields: Vec<&str> = line.split(split_pat).collect();
            fields
                .get(taxonomy_index)
                .map_or(false, |&field| whole_taxon_rank_match(field, needle))
        })
        .collect();

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

/// Search GTDB's using taxon name with partial match.
///
/// This function will write either to stdout or to a file the result of
/// the partial search.
///
/// # Args
/// args: SearArgs structure containing the provided CLI arguments from user
///
pub fn partial_search(args: utils::SearchArgs) -> Result<()> {
    let agent: Agent = utils::get_agent(args.disable_certificate_verification())?;

    for needle in args.get_needles() {
        let search_api = SearchAPI::from(&needle, &args);
        let request_url = search_api.request();

        let response = match agent.get(&request_url).call() {
            Ok(r) => r,
            Err(ureq::Error::Status(code, _)) => {
                bail!("The server returned an unexpected status code ({})", code)
            }
            Err(_) => {
                bail!("There was an error making the request or receiving the response.");
            }
        };

        let handle_response = |result: String| -> Result<()> {
            if args.is_only_num_entries() {
                utils::write_to_output(
                    result
                        .trim_end()
                        .split("\r\n")
                        .skip(1)
                        .count()
                        .to_string()
                        .as_bytes(),
                    args.get_output().clone(),
                )
            } else if args.is_only_print_ids() {
                let ids = result
                    .split("\r\n")
                    .skip(1)
                    .map(|l| {
                        l.split(
                            if args.get_outfmt() == OutputFormat::from("tsv".to_string()) {
                                '\t'
                            } else {
                                ','
                            },
                        )
                        .next()
                        .unwrap_or("")
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");
                utils::write_to_output(ids.as_bytes(), args.get_output().clone())
            } else {
                utils::write_to_output(result.as_bytes(), args.get_output().clone())
            }
        };

        if args.get_outfmt() == OutputFormat::from("json".to_string()) {
            let search_result: SearchResults = response.into_json()?;
            ensure!(
                search_result.get_total_rows() != 0,
                "No matching data found in GTDB"
            );

            let result_str = if args.is_only_num_entries() {
                search_result.get_total_rows().to_string()
            } else if args.is_only_print_ids() {
                search_result
                    .rows
                    .iter()
                    .map(|x| x.gid.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            } else {
                search_result
                    .rows
                    .iter()
                    .map(|x| serde_json::to_string_pretty(x).unwrap())
                    .collect::<Vec<String>>()
                    .join("\n")
            };

            utils::write_to_output(result_str.as_bytes(), args.get_output().clone())?;
        } else {
            let result = response.into_string()?;
            handle_response(result)?;
        }
    }

    Ok(())
}

pub fn search(args: utils::SearchArgs) -> Result<()> {
    let agent: Agent = utils::get_agent(args.disable_certificate_verification())?;

    for needle in args.get_needles() {
        let search_api = SearchAPI::from(&needle, &args);
        let request_url = search_api.request();

        let response = match agent.get(&request_url).call() {
            Ok(r) => r,
            Err(ureq::Error::Status(code, _)) => {
                bail!("The server returned an unexpected status code ({})", code);
            }
            Err(_) => {
                bail!("There was an error making the request or receiving the response.");
            }
        };

        let handle_response = |result: String| -> Result<()> {
            if args.is_only_num_entries() {
                utils::write_to_output(
                    result
                        .trim_end()
                        .split("\r\n")
                        .skip(1)
                        .count()
                        .to_string()
                        .as_bytes(),
                    args.get_output().clone(),
                )
            } else if args.is_only_print_ids() {
                let ids = result
                    .split("\r\n")
                    .skip(1)
                    .map(|l| {
                        l.split(
                            if args.get_outfmt() == OutputFormat::from("tsv".to_string()) {
                                '\t'
                            } else {
                                ','
                            },
                        )
                        .next()
                        .unwrap_or("")
                    })
                    .collect::<Vec<&str>>()
                    .join("\n");
                utils::write_to_output(ids.as_bytes(), args.get_output().clone())
            } else {
                utils::write_to_output(result.as_bytes(), args.get_output().clone())
            }
        };

        if args.get_outfmt() == OutputFormat::from("json".to_string()) {
            let mut search_result: SearchResults = response.into_json()?;
            if args.is_partial_search {
                search_result.filter_json(needle.clone(), args.get_search_field());
            }

            ensure!(
                search_result.get_total_rows() != 0,
                "No matching data found in GTDB"
            );

            let result_str = if args.is_only_num_entries() {
                search_result.get_total_rows().to_string()
            } else if args.is_only_print_ids() {
                search_result
                    .rows
                    .iter()
                    .map(|x| x.gid.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            } else {
                search_result
                    .rows
                    .iter()
                    .map(|x| serde_json::to_string_pretty(x).unwrap())
                    .collect::<Vec<String>>()
                    .join("\n")
            };
            utils::write_to_output(result_str.as_bytes(), args.get_output().clone())?;
        } else {
            let result = response.into_string()?;
            if args.is_partial_search() {
                filter_xsv(
                    result.clone(),
                    &needle,
                    args.get_search_field(),
                    args.get_outfmt(),
                );
            }
            handle_response(result)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_xsv() {
        let test =  "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_001512625.1,Clostridiales bacterium DTU036,d__Bacteria; p__Bacillota; c__Clostridia; o__Eubacteriales; f__; g__; s__,d__Bacteria; p__Bacillota_A; c__Clostridia; o__Peptostreptococcales; f__Acidaminobacteraceae; g__DTU036; s__DTU036 sp001512625,True,True\r\nGCA_001604435.1,Synergistales bacterium Syner_01,d__Bacteria; p__Synergistota; c__Synergistia; o__Synergistales; f__; g__; s__,d__Bacteria; p__Synergistota; c__Synergistia; o__Synergistales; f__Aminobacteriaceae; g__JAAYLU01; s__JAAYLU01 sp001604435,True,True\r\nGCA_001724355.1,Mesorhizobium sp. SCN 65-20,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Phyllobacteriaceae; g__Mesorhizobium; s__,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Aminobacter; s__Aminobacter sp001724355,True,True\r\nGCA_001897605.1,Clostridiales bacterium 38-18,d__Bacteria; p__Bacillota; c__Clostridia; o__Eubacteriales; f__; g__; s__,d__Bacteria; p__Bacillota_A; c__Clostridia; o__Peptostreptococcales; f__Acidaminobacteraceae; g__Fusibacter_C; s__Fusibacter_C sp001897605,True,True\r\nGCA_002029975.1,delta proteobacterium ML8_F1,d__Bacteria; p__; c__Deltaproteobacteria; o__; f__; g__; s__,d__Bacteria; p__Bacillota_A; c__Clostridia; o__Peptostreptococcales; f__Acidaminobacteraceae; g__ML8-F1; s__ML8-F1 sp002029975,True,True\r\nGCA_002068355.1,Synergistetes bacterium ADurb.BinA166,d__Bacteria; p__Synergistota; c__; o__; f__; g__; s__,d__Bacteria; p__Synergistota; c__Synergistia; o__Synergistales; f__Aminobacteriaceae; g__JAAYLU01; s__JAAYLU01 sp002068355,False,True\r\n";
        let res = filter_xsv(
            test.to_string(),
            "JAAYLU01",
            SearchField::GTDB,
            OutputFormat::CSV,
        );
        let expected = "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_001604435.1,Synergistales bacterium Syner_01,d__Bacteria; p__Synergistota; c__Synergistia; o__Synergistales; f__; g__; s__,d__Bacteria; p__Synergistota; c__Synergistia; o__Synergistales; f__Aminobacteriaceae; g__JAAYLU01; s__JAAYLU01 sp001604435,True,True\r\nGCA_002068355.1,Synergistetes bacterium ADurb.BinA166,d__Bacteria; p__Synergistota; c__; o__; f__; g__; s__,d__Bacteria; p__Synergistota; c__Synergistia; o__Synergistales; f__Aminobacteriaceae; g__JAAYLU01; s__JAAYLU01 sp002068355,False,True";
        assert_eq!(res, expected);
    }

    #[test]
    fn test_filter() {
        let mut results = SearchResults {
            rows: vec![
                SearchResult {
                    gid: "1".into(),
                    gtdb_taxonomy: Some("d__Bacteria; p__Proteobacteria; c__Gammaproteobacteria; o__Alteromonadales; f__Alteromonadaceae; g__Alteromonas; s__".into()),
                    ..Default::default()
                },
                SearchResult {
                    gid: "2".into(),
                    gtdb_taxonomy: Some("d__Bacteria; p__Firmicutes; c__Bacilli; o__Bacillales; f__Bacillaceae; g__Bacillus; s__".into()),
                    ..Default::default()
                },
                SearchResult {
                    gid: "3".into(),
                    gtdb_taxonomy: Some("d__Bacteria; p__Proteobacteria; c__Alphaproteobacteria; o__Rhizobiales; f__Rhizobiaceae; g__Rhizobium; s__".into()),
                    ..Default::default()
                },
            ],
            total_rows: 3,
        };
        results.filter_json("Proteobacteria".to_string(), SearchField::default());
        assert_eq!(results.rows.len(), 2);
        assert_eq!(results.get_total_rows(), 2);
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
    fn test_get_taxon_by_rank() {
        let search_result = SearchResult {
            gid: "1".into(),
            gtdb_taxonomy: Some("d__Bacteria; p__Firmicutes; c__Bacilli; o__Bacillales; f__Bacillaceae; g__Bacillus; s__".into()),
            ..Default::default()
        };
        assert_eq!(
            search_result.get_taxon_by_rank("p").unwrap(),
            "p__Firmicutes"
        );
        assert_eq!(search_result.get_taxon_by_rank("c").unwrap(), "c__Bacilli");
        assert_eq!(search_result.get_taxon_by_rank("s").unwrap(), "s__");
    }

    #[test]
    fn test_exact_search_count() {
        let mut args = utils::SearchArgs::new();
        args.add_needle("g__Azorhizobium");
        args.set_count(true);
        args.set_disable_certificate_verification(true);
        args.set_output(Some("test.txt".to_string()));
        args.set_outfmt("json".to_string());
        let res = search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("6".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_partial_search_count() {
        let mut args = utils::SearchArgs::new();
        args.add_needle("g__Azorhizobium");
        args.set_count(true);
        args.set_disable_certificate_verification(true);
        args.set_output(Some("test1.txt".to_string()));
        args.set_outfmt("json".to_string());
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test1.txt").unwrap();
        assert_eq!("6".to_string(), expected);
        std::fs::remove_file("test1.txt").unwrap();
    }

    #[test]
    fn test_exact_search_id() {
        let mut args = utils::SearchArgs::new();
        args.add_needle("g__Azorhizobium");
        args.set_id(true);
        args.set_disable_certificate_verification(true);
        args.set_output(Some("test2.txt".to_string()));
        args.set_outfmt("json".to_string());
        let res = search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test2.txt").unwrap();
        assert_eq!("GCA_023405075.1\nGCA_023448105.1\nGCF_000010525.1\nGCF_000473085.1\nGCF_004364705.1\nGCF_014635325.1".to_string(), expected);
        std::fs::remove_file("test2.txt").unwrap();
    }

    #[test]
    fn test_partial_search_id() {
        let mut args = utils::SearchArgs::new();
        args.add_needle("g__Azorhizobium");
        args.set_id(true);
        args.set_output(Some("test3.txt".to_string()));
        args.set_outfmt("json".to_string());
        args.set_disable_certificate_verification(true);
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test3.txt").unwrap();
        assert_eq!("GCA_023405075.1\nGCA_023448105.1\nGCF_000010525.1\nGCF_000473085.1\nGCF_004364705.1\nGCF_014635325.1".to_string(), expected);
        std::fs::remove_file("test3.txt").unwrap();
    }

    #[test]
    fn test_exact_search_pretty() {
        let mut args = utils::SearchArgs::new();
        args.add_needle("s__Azorhizobium doebereinerae");
        args.set_output(Some("test8.txt".to_string()));
        args.set_outfmt("json".to_string());
        args.set_disable_certificate_verification(true);
        let res = search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test8.txt").unwrap();
        let actual = r#"{
  "gid": "GCF_000473085.1",
  "accession": "GCF_000473085.1",
  "ncbiOrgName": "Azorhizobium doebereinerae UFLA1-100",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium doebereinerae",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium doebereinerae",
  "isGtdbSpeciesRep": true,
  "isNcbiTypeMaterial": true
}"#;
        assert_eq!(actual, expected);
        std::fs::remove_file("test8.txt").unwrap();
    }

    #[test]
    fn test_partial_search_pretty() {
        let mut args = utils::SearchArgs::new();
        args.add_needle("s__Azorhizobium doebereinerae");
        args.set_output(Some("test9.txt".to_string()));
        args.set_disable_certificate_verification(true);
        args.set_outfmt("json".to_string());
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test9.txt").unwrap();
        let actual = r#"{
  "gid": "GCA_023405075.1",
  "accession": "GCA_023405075.1",
  "ncbiOrgName": "Pseudomonadota bacterium",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__; o__; f__; g__; s__",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans",
  "isGtdbSpeciesRep": false,
  "isNcbiTypeMaterial": true
}
{
  "gid": "GCA_023448105.1",
  "accession": "GCA_023448105.1",
  "ncbiOrgName": "Pseudomonadota bacterium",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__; o__; f__; g__; s__",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans",
  "isGtdbSpeciesRep": false,
  "isNcbiTypeMaterial": true
}
{
  "gid": "GCF_000010525.1",
  "accession": "GCF_000010525.1",
  "ncbiOrgName": "Azorhizobium caulinodans ORS 571",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans",
  "isGtdbSpeciesRep": true,
  "isNcbiTypeMaterial": true
}
{
  "gid": "GCF_000473085.1",
  "accession": "GCF_000473085.1",
  "ncbiOrgName": "Azorhizobium doebereinerae UFLA1-100",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium doebereinerae",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium doebereinerae",
  "isGtdbSpeciesRep": true,
  "isNcbiTypeMaterial": true
}
{
  "gid": "GCF_003989665.1",
  "accession": "GCF_003989665.1",
  "ncbiOrgName": "Azospirillum doebereinerae",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhodospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Azospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae",
  "isGtdbSpeciesRep": true,
  "isNcbiTypeMaterial": true
}
{
  "gid": "GCF_004364705.1",
  "accession": "GCF_004364705.1",
  "ncbiOrgName": "Azorhizobium sp. AG788",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans",
  "isGtdbSpeciesRep": false,
  "isNcbiTypeMaterial": true
}
{
  "gid": "GCF_014635325.1",
  "accession": "GCF_014635325.1",
  "ncbiOrgName": "Azorhizobium oxalatiphilum",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium oxalatiphilum",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium oxalatiphilum",
  "isGtdbSpeciesRep": true,
  "isNcbiTypeMaterial": true
}
{
  "gid": "GCF_022214805.1",
  "accession": "GCF_022214805.1",
  "ncbiOrgName": "Azospirillum doebereinerae",
  "ncbiTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhodospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae",
  "gtdbTaxonomy": "d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Azospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae",
  "isGtdbSpeciesRep": false,
  "isNcbiTypeMaterial": true
}"#;
        assert_eq!(actual, expected);
        std::fs::remove_file("test9.txt").unwrap();
    }

    #[test]
    fn test_partial_search_csv() {
        let mut args = utils::SearchArgs::new();
        args.add_needle("s__Azorhizobium doebereinerae");
        args.set_output(Some("test10.txt".to_string()));
        args.set_disable_certificate_verification(true);
        args.set_outfmt("csv".to_string());
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test10.txt").unwrap();
        let actual = "accession,ncbi_organism_name,ncbi_taxonomy,gtdb_taxonomy,gtdb_species_representative,ncbi_type_material\r\nGCA_023405075.1,Pseudomonadota bacterium,d__Bacteria; p__Pseudomonadota; c__; o__; f__; g__; s__,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans,False,True\r\nGCA_023448105.1,Pseudomonadota bacterium,d__Bacteria; p__Pseudomonadota; c__; o__; f__; g__; s__,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans,False,True\r\nGCF_000010525.1,Azorhizobium caulinodans ORS 571,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans,True,True\r\nGCF_000473085.1,Azorhizobium doebereinerae UFLA1-100,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium doebereinerae,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium doebereinerae,True,True\r\nGCF_003989665.1,Azospirillum doebereinerae,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhodospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Azospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae,True,True\r\nGCF_004364705.1,Azorhizobium sp. AG788,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium caulinodans,False,True\r\nGCF_014635325.1,Azorhizobium oxalatiphilum,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Hyphomicrobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium oxalatiphilum,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Azorhizobium; s__Azorhizobium oxalatiphilum,True,True\r\nGCF_022214805.1,Azospirillum doebereinerae,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Rhodospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae,d__Bacteria; p__Pseudomonadota; c__Alphaproteobacteria; o__Azospirillales; f__Azospirillaceae; g__Azospirillum; s__Azospirillum doebereinerae,False,True\r\n";
        assert_eq!(actual, expected);
        std::fs::remove_file("test10.txt").unwrap();
    }
}
