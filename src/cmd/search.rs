use std::collections::HashMap;

use anyhow::Result;
use reqwest::Error;
use serde::{Deserialize, Serialize};

use super::utils;
use crate::api;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Genome {
    gid: String,
    accession: String,
    ncbi_org_name: String,
    ncbi_taxonomy: String,
    gtdb_taxonomy: String,
    is_gtdb_species_rep: bool,
    is_ncbi_type_material: bool,
}

impl Genome {
    fn get_gtdb_level(&self, level: &str) -> String {
        // Check for Undefined (Failed Quality Check) in gtdb_taxonomy field
        if self.gtdb_taxonomy != "Undefined (Failed Quality Check)" {
            let fields: Vec<&str> = self.gtdb_taxonomy.split(';').collect();

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
struct SearchResult {
    rows: Vec<Genome>,
}

impl SearchResult {
    fn search_by_level(&self, level: &str, needle: &str) -> Vec<Genome> {
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

    // format the request
    let search_api = api::Search::new(args.get_needle(), options);

    let request_url = search_api.request();

    let response = reqwest::blocking::get(&request_url)?;

    let genomes: SearchResult = response.json()?;

    // Perfom partial match or not?
    match partial {
        true => {
            let genome_list = genomes.rows;

            // Return number of genomes?
            match count {
                true => println!("{}", genome_list.len()),

                // Return only genome id?
                false => match gid {
                    true => {
                        let list: Vec<String> = genome_list.iter().map(|x| x.gid.clone()).collect();
                        for gid in list {
                            println!("{gid}");
                        }
                    }
                    // Pretty print json data?
                    false => match raw {
                        true => {
                            for genome in genome_list {
                                let g = serde_json::to_string(&genome).unwrap();
                                println!("{g}");
                            }
                        }
                        false => {
                            for genome in genome_list {
                                let g = serde_json::to_string_pretty(&genome).unwrap();
                                println!("{g}");
                            }
                        }
                    },
                },
            }
        }
        false => {
            let genome_list = genomes.search_by_level(&args.get_level(), &args.get_needle());

            // Return number of genomes?
            match count {
                true => println!("{}", genome_list.len()),

                // Return only genome id?
                false => match gid {
                    true => {
                        let list: Vec<String> = genome_list.iter().map(|x| x.gid.clone()).collect();
                        for gid in list {
                            println!("{gid}");
                        }
                    }
                    // Pretty print json data?
                    false => match raw {
                        true => {
                            for genome in genome_list {
                                let g = serde_json::to_string(&genome).unwrap();
                                println!("{g}");
                            }
                        }
                        false => {
                            for genome in genome_list {
                                let g = serde_json::to_string_pretty(&genome).unwrap();
                                println!("{g}");
                            }
                        }
                    },
                },
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
        let genome = Genome {
            gid: "test".to_owned(),
            accession: "test".to_owned(),
            ncbi_org_name: "test".to_owned(),
            ncbi_taxonomy: "test".to_owned(),
            gtdb_taxonomy: "d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned(),
            is_gtdb_species_rep: false,
            is_ncbi_type_material: false,
        };

        assert_eq!(genome.get_gtdb_level("domain"), "Bacteria");
        assert_eq!(genome.get_gtdb_level("phylum"), "Actinobacteriota");
        assert_eq!(genome.get_gtdb_level("class"), "Actinobacteria");
        assert_eq!(genome.get_gtdb_level("order"), "Actinomycetales");
        assert_eq!(genome.get_gtdb_level("family"), "Streptomycetaceae");
        assert_eq!(genome.get_gtdb_level("genus"), "Streptomyces");
        assert_eq!(genome.get_gtdb_level("species"), "");
        /*
        assert!(matches!(
            genome.get_gtdb_level("invalid"),
            _ if true // the `_ => unreachable!()` branch will be executed, so this should always panic
        ));*/
    }

    #[test]
    fn test_search_result_search_by_level() {
        let genome1 = Genome {
            gid: "test1".to_owned(),
            accession: "test1".to_owned(),
            ncbi_org_name: "test1".to_owned(),
            ncbi_taxonomy: "test1".to_owned(),
            gtdb_taxonomy: "d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned(),
            is_gtdb_species_rep: false,
            is_ncbi_type_material: false,
        };
        let genome2 = Genome {
            gid: "test2".to_owned(),
            accession: "test2".to_owned(),
            ncbi_org_name: "test2".to_owned(),
            ncbi_taxonomy: "test2".to_owned(),
            gtdb_taxonomy: "d__Bacteria; p__Firmicutes; c__Bacilli; o__Lactobacillales; f__Lactobacillaceae; g__Lactobacillus; s__".to_owned(),
            is_gtdb_species_rep: false,
            is_ncbi_type_material: false,
        };
        let search_result = SearchResult {
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
            vec![genome1.clone()]
        );
        assert_eq!(
            search_result.search_by_level("genus", "Lactobacillus"),
            vec![genome2]
        );
    }
}
