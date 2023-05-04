use anyhow::{anyhow, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use super::utils;

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
    fn get_gtdb_level(&self, level: &str) -> Result<String> {
        let tax = self.gtdb_taxonomy.clone().ok_or_else(|| {
            anyhow!("Failed to perform exact match as gtdb taxonomy is a null field")
        })?;

        let res = match level {
            "domain" => tax
                .replace("d__", "")
                .split("; ")
                .next()
                .unwrap_or("")
                .to_string(),
            "phylum" => tax
                .replace("p__", "")
                .split("; ")
                .nth(1)
                .unwrap_or("")
                .to_string(),
            "class" => tax
                .replace("c__", "")
                .split("; ")
                .nth(2)
                .unwrap_or("")
                .to_string(),
            "order" => tax
                .replace("o__", "")
                .split("; ")
                .nth(3)
                .unwrap_or("")
                .to_string(),
            "family" => tax
                .replace("f__", "")
                .split("; ")
                .nth(4)
                .unwrap_or("")
                .to_string(),
            "genus" => tax
                .replace("g__", "")
                .split("; ")
                .nth(5)
                .unwrap_or("")
                .to_string(),
            "species" => tax
                .replace("s__", "")
                .split("; ")
                .nth(6)
                .unwrap_or("")
                .to_string(),
            &_ => unreachable!("all fields have been taken into account"),
        };

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
    fn retain(&mut self, l: &str, c: &str) {
        self.rows.retain(|x| x.get_gtdb_level(l).unwrap() == c);
        self.total_rows = self.rows.len() as u32;
    }
    fn get_total_rows(&self) -> u32 {
        self.total_rows
    }
    fn get_rows(&self) -> Vec<SearchResult> {
        self.rows.clone()
    }
}

pub fn partial_search(args: utils::SearchArgs) -> Result<()> {
    // get args
    let gid = args.get_gid();
    let count = args.get_count();
    let raw = args.get_raw();
    let output = args.get_out();

    let needles = args.get_needle();

    for needle in needles {
        let search_api = SearchAPI::from(&needle, &args);

        let request_url = search_api.request();

        let response = reqwest::blocking::get(&request_url)
            .with_context(|| "Failed to send request to GTDB API".to_string())?;

        utils::check_status(&response)?;

        let search_result: SearchResults = response.json().with_context(|| {
            "Failed to deserialize request response to search result structure".to_string()
        })?;

        let search_result_list = search_result.get_rows();
        ensure!(
            search_result.get_total_rows() != 0,
            "No matching data found in GTDB"
        );

        // Return number of genomes?
        match count {
            true => {
                utils::write_to_output(
                    format!("{}{}", &search_result.get_total_rows(), "\n"),
                    output.clone(),
                )?;
            }

            // Return only genome id?
            false => match gid {
                true => {
                    let list: Vec<String> =
                        search_result_list.iter().map(|x| x.gid.clone()).collect();

                    for gid in list {
                        utils::write_to_output(format!("{}{}", gid, "\n"), output.clone())?;
                    }
                }
                // Return all data in pretty print json?
                false => match raw {
                    true => {
                        for result in search_result_list {
                            let genome_string =
                                serde_json::to_string(&result).with_context(|| {
                                    "Failed to convert search result to json string".to_string()
                                })?;
                            utils::write_to_output(
                                format!("{}{}", genome_string, "\n"),
                                output.clone(),
                            )?;
                        }
                    }
                    false => {
                        for result in search_result_list {
                            let genome_string = serde_json::to_string_pretty(&result)
                                .with_context(|| {
                                    "Failed to convert search result to json string".to_string()
                                })?;
                            utils::write_to_output(
                                format!("{}{}", genome_string, "\n"),
                                output.clone(),
                            )?;
                        }
                    }
                },
            },
        }
    }

    Ok(())
}

pub fn exact_search(args: utils::SearchArgs) -> Result<()> {
    // get args
    let gid = args.get_gid();
    let count = args.get_count();
    let raw = args.get_raw();
    let output = args.get_out();

    let needles = args.get_needle();

    for needle in needles {
        let oneedle = needle.clone();
        let search_api = SearchAPI::from(&oneedle, &args);

        let request_url = search_api.request();

        let response = reqwest::blocking::get(&request_url)
            .with_context(|| "Failed to send request to GTDB API".to_string())?;

        utils::check_status(&response)?;

        let mut search_result: SearchResults = response.json().with_context(|| {
            "Failed to deserialize request response to search result structure".to_string()
        })?;
        search_result.retain(&args.get_level(), &needle);
        ensure!(
            search_result.get_total_rows() != 0,
            "No matching data found in GTDB"
        );

        // Return number of genomes?
        match count {
            true => {
                utils::write_to_output(
                    format!("{}{}", search_result.get_total_rows(), "\n"),
                    output.clone(),
                )?;
            }

            // Return only genome id?
            false => {
                match gid {
                    true => {
                        let list: Vec<String> =
                            search_result.rows.iter().map(|x| x.gid.clone()).collect();

                        for gid in list {
                            utils::write_to_output(format!("{}{}", gid, "\n"), output.clone())?;
                        }
                    }
                    // Return all data in pretty print json?
                    false => match raw {
                        true => {
                            for result in search_result.rows {
                                let genome_string =
                                    serde_json::to_string(&result).with_context(|| {
                                        "Failed to convert search result to json string".to_string()
                                    })?;
                                utils::write_to_output(
                                    format!("{}{}", genome_string, "\n"),
                                    output.clone(),
                                )?;
                            }
                        }
                        false => {
                            for result in search_result.rows {
                                let genome_string = serde_json::to_string_pretty(&result)
                                    .with_context(|| {
                                        "Failed to convert search result to json string".to_string()
                                    })?;
                                utils::write_to_output(
                                    format!("{}{}", genome_string, "\n"),
                                    output.clone(),
                                )?;
                            }
                        }
                    },
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retain() {
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
        results.retain("phylum", "Proteobacteria");
        assert_eq!(results.get_rows().len(), 2);
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
        assert_eq!(results.get_rows().len(), 3);
    }

    #[test]
    fn test_get_gtdb_level() {
        let search_result = SearchResult {
            gid: "1".into(),
            gtdb_taxonomy: Some("d__Bacteria; p__Firmicutes; c__Bacilli; o__Bacillales; f__Bacillaceae; g__Bacillus; s__".into()),
            ..Default::default()
        };
        assert_eq!(
            search_result.get_gtdb_level("phylum").unwrap(),
            "Firmicutes"
        );
        assert_eq!(search_result.get_gtdb_level("class").unwrap(), "Bacilli");
        assert_eq!(search_result.get_gtdb_level("species").unwrap(), "");
    }

    #[test]
    fn test_exact_search_count() {
        let mut args = utils::SearchArgs::new();
        args.set_needle(vec!["Azorhizobium".to_string()]);
        args.set_count(true);
        args.set_out(Some("test.txt".to_string()));
        let res = exact_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("6\n".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_partial_search_count() {
        let mut args = utils::SearchArgs::new();
        args.set_needle(vec!["Azorhizobium".to_string()]);
        args.set_count(true);
        args.set_out(Some("test.txt".to_string()));
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("11\n".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_exact_search_id() {
        let mut args = utils::SearchArgs::new();
        args.set_needle(vec!["Azorhizobium".to_string()]);
        args.set_id(true);
        args.set_out(Some("test.txt".to_string()));
        let res = exact_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("GCA_023405075.1\nGCA_023448105.1\nGCF_000010525.1\nGCF_000473085.1\nGCF_004364705.1\nGCF_014635325.1\n".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_partial_search_id() {
        let mut args = utils::SearchArgs::new();
        args.set_needle(vec!["Azorhizobium".to_string()]);
        args.set_id(true);
        args.set_out(Some("test.txt".to_string()));
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("GCA_002279595.1\nGCA_002280795.1\nGCA_002280945.1\nGCA_002281175.1\nGCA_002282175.1\nGCA_023405075.1\nGCA_023448105.1\nGCF_000010525.1\nGCF_000473085.1\nGCF_004364705.1\nGCF_014635325.1\n".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_exact_search_raw() {
        let mut args = utils::SearchArgs::new();
        args.set_needle(vec!["Azorhizobium".to_string()]);
        args.set_raw(true);
        args.set_id(true);
        args.set_out(Some("test.txt".to_string()));
        let res = exact_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("GCA_023405075.1\nGCA_023448105.1\nGCF_000010525.1\nGCF_000473085.1\nGCF_004364705.1\nGCF_014635325.1\n".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_partial_search_raw() {
        let mut args = utils::SearchArgs::new();
        args.set_needle(vec!["Azorhizobium".to_string()]);
        args.set_raw(true);
        args.set_id(true);
        args.set_out(Some("test.txt".to_string()));
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("GCA_002279595.1\nGCA_002280795.1\nGCA_002280945.1\nGCA_002281175.1\nGCA_002282175.1\nGCA_023405075.1\nGCA_023448105.1\nGCF_000010525.1\nGCF_000473085.1\nGCF_004364705.1\nGCF_014635325.1\n".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }
}
