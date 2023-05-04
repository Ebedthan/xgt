use std::fs::OpenOptions;
use std::io::{self, Write};

use anyhow::{anyhow, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use super::utils;

use crate::api::search_api::SearchAPI;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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
    fn search_by_level(&self, level: &str, needle: &str) -> Vec<SearchResult> {
        self.rows
            .clone()
            .into_iter()
            .filter(|x| x.get_gtdb_level(level).unwrap() == needle)
            .collect()
    }
    fn get_total_rows(&self) -> u32 {
        self.total_rows
    }
    fn get_rows(&self) -> Vec<SearchResult> {
        self.rows.clone()
    }
}

pub fn search_gtdb(args: utils::SearchArgs) -> Result<()> {
    // get args
    let gid = args.get_gid();
    let partial = args.get_partial();
    let count = args.get_count();
    let raw = args.get_raw();
    let output = args.get_out();

    let needles = args.get_needle();

    for needle in needles {
        // format the request
        let oneedle = needle.clone();
        let search_api = SearchAPI::from(needle, &args);

        let request_url = search_api.request();

        let response = reqwest::blocking::get(&request_url)
            .with_context(|| "Failed to send request to GTDB API".to_string())?;

        utils::check_status(&response)?;

        let search_result: SearchResults = response.json().with_context(|| {
            "Failed to deserialize request response to search result structure".to_string()
        })?;

        // Perfom partial match or not?
        match partial {
            true => {
                let search_result_list = search_result.get_rows();
                ensure!(
                    search_result.get_total_rows() != 0,
                    "No matching data found in GTDB"
                );

                // Return number of genomes?
                match count {
                    true => {
                        if let Some(path) = output.clone() {
                            let path_clone = path.clone();
                            let mut file = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(path)
                                .with_context(|| format!("Failed to create file {path_clone}"))?;
                            file.write_all(
                                format!("{}{}", &search_result.get_total_rows(), "\n").as_bytes(),
                            )
                            .with_context(|| format!("Failed to write to {path_clone}"))?;
                        } else {
                            writeln!(io::stdout(), "{}", search_result.get_total_rows())?;
                        }
                    }

                    // Return only genome id?
                    false => match gid {
                        true => {
                            let list: Vec<String> =
                                search_result_list.iter().map(|x| x.gid.clone()).collect();

                            if let Some(path) = output.clone() {
                                let path_clone = path.clone();
                                let mut file = OpenOptions::new()
                                    .append(true)
                                    .create(true)
                                    .open(path)
                                    .with_context(|| {
                                        format!("Failed to create file {path_clone}")
                                    })?;

                                for gid in list {
                                    file.write_all(format!("{}{}", gid, "\n").as_bytes())
                                        .with_context(|| {
                                            format!("Failed to write to {path_clone}")
                                        })?;
                                }
                            } else {
                                for gid in list {
                                    writeln!(io::stdout(), "{}", gid)?;
                                }
                            }
                        }
                        // Return all data in pretty print json?
                        false => match raw {
                            true => {
                                if let Some(path) = output.clone() {
                                    let path_clone = path.clone();
                                    let mut file = OpenOptions::new()
                                        .append(true)
                                        .create(true)
                                        .open(path)
                                        .with_context(|| {
                                            format!("Failed to create file {path_clone}")
                                        })?;
                                    for result in search_result_list {
                                        let genome_string = serde_json::to_string(&result)
                                            .with_context(|| {
                                                "Failed to convert search result to json string"
                                                    .to_string()
                                            })?;
                                        file.write_all(genome_string.as_bytes()).with_context(
                                            || format!("Failed to write to {path_clone}"),
                                        )?;
                                        file.write_all("\n".as_bytes()).with_context(|| {
                                            format!("Failed to write to {path_clone}")
                                        })?;
                                    }
                                } else {
                                    let mut stdout_lock = io::stdout().lock();
                                    for result in search_result_list {
                                        let genome_string = serde_json::to_string(&result)
                                            .with_context(|| {
                                                "Failed to convert search result to json string"
                                                    .to_string()
                                            })?;
                                        writeln!(stdout_lock, "{genome_string}")?;
                                    }
                                }
                            }
                            false => {
                                if let Some(path) = output.clone() {
                                    let path_clone = path.clone();
                                    let mut file = OpenOptions::new()
                                        .append(true)
                                        .create(true)
                                        .open(path)
                                        .with_context(|| {
                                            format!("Failed to create file {path_clone}")
                                        })?;
                                    for result in search_result_list {
                                        let genome_string = serde_json::to_string_pretty(&result)
                                            .with_context(|| {
                                            "Failed to convert search result to json string"
                                                .to_string()
                                        })?;
                                        file.write_all(genome_string.as_bytes()).with_context(
                                            || format!("Failed to write to {path_clone}"),
                                        )?;
                                        file.write_all("\n".as_bytes()).with_context(|| {
                                            format!("Failed to write to {path_clone}")
                                        })?;
                                    }
                                } else {
                                    let mut stdout_lock = io::stdout().lock();
                                    for result in search_result_list {
                                        let genome_string = serde_json::to_string_pretty(&result)
                                            .with_context(|| {
                                            "Failed to convert search result to json string"
                                                .to_string()
                                        })?;
                                        writeln!(stdout_lock, "{genome_string}")?;
                                    }
                                }
                            }
                        },
                    },
                }
            }
            false => {
                let search_result_list = search_result.search_by_level(&args.get_level(), &oneedle);
                ensure!(
                    search_result.get_total_rows() != 0,
                    "No matching data found in GTDB"
                );

                // Return number of genomes?
                match count {
                    true => {
                        if let Some(path) = output.clone() {
                            let path_clone = path.clone();
                            let mut file = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(path)
                                .with_context(|| format!("Failed to create file {path_clone}"))?;
                            file.write_all(
                                format!("{}{}", search_result.get_total_rows(), "\n").as_bytes(),
                            )
                            .with_context(|| format!("Failed to write to {path_clone}"))?;
                        } else {
                            writeln!(io::stdout(), "{}", search_result.get_total_rows())?;
                        }
                    }

                    // Return only genome id?
                    false => {
                        match gid {
                            true => {
                                let list: Vec<String> =
                                    search_result_list.iter().map(|x| x.gid.clone()).collect();

                                if let Some(path) = output.clone() {
                                    let path_clone = path.clone();
                                    let mut file = OpenOptions::new()
                                        .append(true)
                                        .create(true)
                                        .open(path)
                                        .with_context(|| {
                                            format!("Failed to create file {path_clone}")
                                        })?;
                                    for gid in list {
                                        file.write_all(format!("{}{}", gid, "\n").as_bytes())
                                            .with_context(|| {
                                                format!("Failed to write to {path_clone}")
                                            })?;
                                    }
                                } else {
                                    let mut stdout_lock = io::stdout().lock();
                                    for gid in list {
                                        writeln!(stdout_lock, "{}", gid)?;
                                    }
                                }
                            }
                            // Return all data in pretty print json?
                            false => {
                                match raw {
                                    true => {
                                        if let Some(path) = output.clone() {
                                            let path_clone = path.clone();
                                            let mut file = OpenOptions::new()
                                                .append(true)
                                                .create(true)
                                                .open(path)
                                                .with_context(|| {
                                                    format!("Failed to create file {path_clone}")
                                                })?;
                                            for result in search_result_list {
                                                let genome_string = serde_json::to_string(&result).with_context(|| "Failed to convert search result to json string".to_string())?;
                                                file.write_all(genome_string.as_bytes())
                                                    .with_context(|| {
                                                        format!("Failed to write to {path_clone}")
                                                    })?;
                                                file.write_all("\n".as_bytes()).with_context(
                                                    || format!("Failed to write to {path_clone}"),
                                                )?;
                                            }
                                        } else {
                                            let mut stdout_lock = io::stdout();
                                            for result in search_result_list {
                                                let genome_string = serde_json::to_string(&result).with_context(|| "Failed to convert search result to json string".to_string())?;
                                                writeln!(stdout_lock, "{genome_string}")?;
                                            }
                                        }
                                    }
                                    false => {
                                        if let Some(path) = output.clone() {
                                            let path_clone = path.clone();
                                            let mut file = OpenOptions::new()
                                                .append(true)
                                                .create(true)
                                                .open(path)
                                                .with_context(|| {
                                                    format!("Failed to create file {path_clone}")
                                                })?;
                                            for result in search_result_list {
                                                let genome_string = serde_json::to_string_pretty(
                                                    &result,
                                                )
                                                .with_context(|| {
                                                    "Failed to convert search result to json string"
                                                        .to_string()
                                                })?;
                                                file.write_all(genome_string.as_bytes())
                                                    .with_context(|| {
                                                        format!("Failed to write to {path_clone}")
                                                    })?;
                                                file.write_all("\n".as_bytes()).with_context(
                                                    || format!("Failed to write to {path_clone}"),
                                                )?;
                                            }
                                        } else {
                                            let mut stdout_lock = io::stdout();

                                            for result in search_result_list {
                                                let genome_string = serde_json::to_string_pretty(
                                                    &result,
                                                )
                                                .with_context(|| {
                                                    "Failed to convert search result to json string"
                                                        .to_string()
                                                })?;
                                                writeln!(stdout_lock, "{genome_string}")?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_genome_get_gtdb_level() {
        let genome = SearchResult {
            gid: "test".to_owned(),
            accession: Some("test".to_owned()),
            ncbi_org_name: Some("test".to_owned()),
            ncbi_taxonomy: Some("test".to_owned()),
            gtdb_taxonomy: Some("d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
        };

        let genome1 = SearchResult {
            gid: "test".to_owned(),
            accession: Some("test".to_owned()),
            ncbi_org_name: Some("test".to_owned()),
            ncbi_taxonomy: Some("test".to_owned()),
            gtdb_taxonomy: Some("".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
        };

        assert_eq!(genome.get_gtdb_level("domain").unwrap(), "Bacteria");
        assert_eq!(genome.get_gtdb_level("phylum").unwrap(), "Actinobacteriota");
        assert_eq!(genome.get_gtdb_level("class").unwrap(), "Actinobacteria");
        assert_eq!(genome.get_gtdb_level("order").unwrap(), "Actinomycetales");
        assert_eq!(
            genome.get_gtdb_level("family").unwrap(),
            "Streptomycetaceae"
        );
        assert_eq!(genome.get_gtdb_level("genus").unwrap(), "Streptomyces");
        assert_eq!(genome.get_gtdb_level("species").unwrap(), "".to_owned());
        assert_eq!(genome1.get_gtdb_level("genus").unwrap(), "".to_owned());
    }

    #[test]
    fn test_search_result_search_by_level() {
        let genome1 = SearchResult {
            gid: "test1".to_owned(),
            accession: Some("test1".to_owned()),
            ncbi_org_name: Some("test1".to_owned()),
            ncbi_taxonomy: Some("test1".to_owned()),
            gtdb_taxonomy: Some("d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
        };

        let genome2 = SearchResult {
            gid: "test2".to_owned(),
            accession: Some("test2".to_owned()),
            ncbi_org_name: Some("test2".to_owned()),
            ncbi_taxonomy: Some("test2".to_owned()),
            gtdb_taxonomy: Some("d__Bacteria; p__Firmicutes; c__Bacilli; o__Lactobacillales; f__Lactobacillaceae; g__Lactobacillus; s__".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
        };

        let search_result = SearchResults {
            rows: vec![genome1.clone(), genome2.clone()],
            total_rows: 2,
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

    #[test]
    fn test_search_gtdb_file_count_true() {
        let args = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: false,
            partial: false,
            rep: false,
            raw: false,
            type_material: false,
            count: true,
            out: Some(String::from("search")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search")).unwrap();
    }

    #[test]
    fn test_search_gtdb_file_count_false() {
        let args = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: false,
            partial: false,
            rep: false,
            raw: false,
            type_material: false,
            count: false,
            out: Some(String::from("search1")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search1")).unwrap();
    }

    #[test]
    fn test_search_gtdb_file_gid_true() {
        let args = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: true,
            partial: false,
            rep: false,
            raw: false,
            type_material: false,
            count: true,
            out: Some(String::from("search3")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search3")).unwrap();
    }

    #[test]
    fn test_search_gtdb_file_gid_false() {
        let args = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: false,
            partial: false,
            rep: false,
            raw: true,
            type_material: false,
            count: false,
            out: None,
        };

        assert!(search_gtdb(args).is_ok());
    }

    #[test]
    fn test_search_gtdb_file_gid_false_no_out() {
        let args = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: true,
            partial: false,
            rep: false,
            raw: true,
            type_material: false,
            count: false,
            out: Some(String::from("search4")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search4")).unwrap();
    }

    #[test]
    fn test_search_gtdb_file_gid_false_no_out_1() {
        let args = utils::SearchArgs {
            needle: vec!["Aminobacter".to_owned()],
            level: "genus".to_owned(),
            id: true,
            partial: false,
            rep: false,
            raw: false,
            type_material: false,
            count: false,
            out: Some(String::from("search5")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search5")).unwrap();
    }
}
