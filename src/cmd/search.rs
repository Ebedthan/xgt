use std::fs;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use reqwest::Error;
use serde::{Deserialize, Serialize};

use super::utils;
use crate::api;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    gid: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    accession: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_org_name: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_taxonomy: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    gtdb_taxonomy: String,
    is_gtdb_species_rep: bool,
    is_ncbi_type_material: bool,
}

impl SearchResult {
    fn get_gtdb_level(&self, level: &str) -> String {
        let fields: Vec<&str> = self.gtdb_taxonomy.split(';').collect();
        // Check for Undefined (Failed Quality Check) in gtdb_taxonomy field
        if fields.len() == 7 {
            match level {
                "domain" => fields[0].replace("d__", ""),
                "phylum" => fields[1].replace(" p__", ""),
                "class" => fields[2].replace(" c__", ""),
                "order" => fields[3].replace(" o__", ""),
                "family" => fields[4].replace(" f__", ""),
                "genus" => fields[5].replace(" g__", ""),
                "species" => fields[6].replace(" s__", ""),
                &_ => unreachable!("all fields have been taken into account"),
            }
        } else {
            String::from("")
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct SearchResults {
    rows: Vec<SearchResult>,
}

impl SearchResults {
    fn search_by_level(&self, level: &str, needle: &str) -> Vec<SearchResult> {
        self.rows
            .clone()
            .into_iter()
            .filter(|x| x.get_gtdb_level(level) == needle)
            .collect()
    }
}

pub fn search_gtdb(args: utils::SearchArgs) -> Result<(), Error> {
    // get args
    let mut options = HashMap::new();
    options.insert(
        "gtdb_species_rep_only".to_owned(),
        utils::bool_as_string(args.get_rep()),
    );
    options.insert(
        "ncbi_type_material_only".to_owned(),
        utils::bool_as_string(args.get_type_material()),
    );

    let gid = args.get_gid();
    let partial = args.get_partial();
    let count = args.get_count();
    let raw = args.get_raw();
    let output = args.get_out();

    if let Some(filename) = output.clone() {
        let path = Path::new(&filename);
        if path.exists() {
            eprintln!("error: file should not already exist");
            std::process::exit(1);
        }
    }

    let needles = args.get_needle();

    for needle in needles {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;

        // format the request
        let oneedle = needle.clone();
        let search_api = api::Search::new(needle, &options);

        let request_url = search_api.request();

        let response = client.get(&request_url).send()?;

        let genomes: SearchResults = response.json()?;

        // Perfom partial match or not?
        match partial {
            true => {
                let genome_list = genomes.rows;

                // Return number of genomes?
                match count {
                    true => {
                        if let Some(filename) = output.clone() {
                            let mut file =
                                fs::File::create(filename).expect("file path should be correct");
                            file.write_all(&genome_list.len().to_ne_bytes()).unwrap();
                        } else {
                            println!("{}", genome_list.len());
                        }
                    }

                    // Return only genome id?
                    false => match gid {
                        true => {
                            let list: Vec<String> =
                                genome_list.iter().map(|x| x.gid.clone()).collect();

                            if let Some(filename) = output.clone() {
                                let mut file = fs::File::create(filename)
                                    .expect("file path should be correct");
                                for gid in list {
                                    file.write_all(gid.as_bytes()).unwrap();
                                }
                            } else {
                                for gid in list {
                                    println!("{gid}");
                                }
                            }
                        }
                        // Return all data in pretty print json?
                        false => match raw {
                            true => {
                                if let Some(filename) = output.clone() {
                                    let file = fs::File::create(filename)
                                        .expect("file path should be correct");
                                    for genome in genome_list {
                                        let g = serde_json::to_string(&genome).unwrap();
                                        serde_json::to_writer(&file, &g).unwrap();
                                    }
                                } else {
                                    for genome in genome_list {
                                        let g = serde_json::to_string(&genome).unwrap();
                                        println!("{g}");
                                    }
                                }
                            }
                            false => {
                                if let Some(filename) = output.clone() {
                                    let file = fs::File::create(filename)
                                        .expect("file path should be correct");
                                    for genome in genome_list {
                                        serde_json::to_writer_pretty(&file, &genome).unwrap();
                                    }
                                } else {
                                    for genome in genome_list {
                                        let g = serde_json::to_string_pretty(&genome).unwrap();
                                        println!("{g}");
                                    }
                                }
                            }
                        },
                    },
                }
            }
            false => {
                let genome_list = genomes.search_by_level(&args.get_level(), &oneedle);

                // Return number of genomes?
                match count {
                    true => {
                        if let Some(filename) = output.clone() {
                            let mut file =
                                fs::File::create(filename).expect("file path should be correct");
                            file.write_all(&genome_list.len().to_ne_bytes()).unwrap();
                        } else {
                            println!("{}", genome_list.len());
                        }
                    }

                    // Return only genome id?
                    false => match gid {
                        true => {
                            let list: Vec<String> =
                                genome_list.iter().map(|x| x.gid.clone()).collect();

                            if let Some(filename) = output.clone() {
                                let mut file = fs::File::create(filename)
                                    .expect("file path should be correct");
                                for gid in list {
                                    file.write_all(gid.as_bytes()).unwrap();
                                }
                            } else {
                                for gid in list {
                                    println!("{gid}");
                                }
                            }
                        }
                        // Return all data in pretty print json?
                        false => match raw {
                            true => {
                                if let Some(filename) = output.clone() {
                                    let file = fs::File::create(filename)
                                        .expect("file path should be correct");
                                    for genome in genome_list {
                                        let g = serde_json::to_string(&genome).unwrap();
                                        serde_json::to_writer(&file, &g).unwrap();
                                    }
                                } else {
                                    for genome in genome_list {
                                        let g = serde_json::to_string(&genome).unwrap();
                                        println!("{g}");
                                    }
                                }
                            }
                            false => {
                                if let Some(filename) = output.clone() {
                                    let file = fs::File::create(filename)
                                        .expect("file path should be correct");
                                    for genome in genome_list {
                                        let g = serde_json::to_string_pretty(&genome).unwrap();
                                        serde_json::to_writer_pretty(&file, &g).unwrap();
                                    }
                                } else {
                                    for genome in genome_list {
                                        let g = serde_json::to_string_pretty(&genome).unwrap();
                                        println!("{g}");
                                    }
                                }
                            }
                        },
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
    fn test_genome_get_gtdb_level() {
        let genome = SearchResult {
            gid: "test".to_owned(),
            accession: "test".to_owned(),
            ncbi_org_name: "test".to_owned(),
            ncbi_taxonomy: "test".to_owned(),
            gtdb_taxonomy: "d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned(),
            is_gtdb_species_rep: false,
            is_ncbi_type_material: false,
        };

        let genome1 = SearchResult {
            gid: "test".to_owned(),
            accession: "test".to_owned(),
            ncbi_org_name: "test".to_owned(),
            ncbi_taxonomy: "test".to_owned(),
            gtdb_taxonomy: "".to_owned(),
            is_gtdb_species_rep: false,
            is_ncbi_type_material: false,
        };

        assert_eq!(genome.get_gtdb_level("domain"), "Bacteria");
        assert_eq!(genome.get_gtdb_level("phylum"), "Actinobacteriota");
        assert_eq!(genome.get_gtdb_level("class"), "Actinobacteria");
        assert_eq!(genome.get_gtdb_level("order"), "Actinomycetales");
        assert_eq!(genome.get_gtdb_level("family"), "Streptomycetaceae");
        assert_eq!(genome.get_gtdb_level("genus"), "Streptomyces");
        assert_eq!(genome.get_gtdb_level("species"), "".to_owned());
        assert_eq!(genome1.get_gtdb_level("genus"), "".to_owned());
    }

    #[test]
    fn test_search_result_search_by_level() {
        let genome1 = SearchResult {
            gid: "test1".to_owned(),
            accession: "test1".to_owned(),
            ncbi_org_name: "test1".to_owned(),
            ncbi_taxonomy: "test1".to_owned(),
            gtdb_taxonomy: "d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned(),
            is_gtdb_species_rep: false,
            is_ncbi_type_material: false,
        };
        let genome2 = SearchResult {
            gid: "test2".to_owned(),
            accession: "test2".to_owned(),
            ncbi_org_name: "test2".to_owned(),
            ncbi_taxonomy: "test2".to_owned(),
            gtdb_taxonomy: "d__Bacteria; p__Firmicutes; c__Bacilli; o__Lactobacillales; f__Lactobacillaceae; g__Lactobacillus; s__".to_owned(),
            is_gtdb_species_rep: false,
            is_ncbi_type_material: false,
        };
        let search_result = SearchResults {
            rows: vec![genome1.clone(), genome2.clone()],
        };

        assert_eq!(
            search_result.search_by_level("phylum", "Actinobacteriota"),
            vec![genome1.clone()]
        );
        assert_eq!(
            search_result.search_by_level("class", "Actinobacteria"),
            vec![genome1.clone()]
        );
        assert_eq!(
            search_result.search_by_level("genus", "Streptomyces"),
            vec![genome1]
        );
        assert_eq!(
            search_result.search_by_level("genus", "Lactobacillus"),
            vec![genome2]
        );
    }

    #[test]
    fn test_search_gtdb() {
        let args = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: false,
            partial: false,
            rep: false,
            raw: false,
            type_material: false,
            count: false,
            out: None,
        };

        let args1 = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: true,
            partial: true,
            rep: true,
            raw: true,
            type_material: true,
            count: true,
            out: None,
        };

        assert!(search_gtdb(args).is_ok());
        assert!(search_gtdb(args1).is_ok());
    }
}
