use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::{collections::HashMap, time::Duration};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::utils;
use crate::api;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    gid: Option<String>,
    accession: Option<String>,
    ncbi_org_name: Option<String>,
    ncbi_taxonomy: Option<String>,
    gtdb_taxonomy: Option<String>,
    is_gtdb_species_rep: Option<bool>,
    is_ncbi_type_material: Option<bool>,
}

impl SearchResult {
    fn get_gtdb_level(&self, level: &str) -> String {
        let mut fields: Vec<String> = Vec::new();
        let tax = self.gtdb_taxonomy.clone();
        if let Some(taxonomy) = tax {
            let tax: Vec<String> = taxonomy
                .split(';')
                .collect::<Vec<&str>>()
                .iter()
                .map(|x| x.to_string())
                .collect();
            for f in tax {
                fields.push(f);
            }
        } else {
            eprintln!("Failed to perform exact match as gtdb taxonomy is a null field");
            std::process::exit(1);
        }
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

pub fn search_gtdb(args: utils::SearchArgs) -> Result<()> {
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
            writeln!(
                io::stderr(),
                "error: file {} should not already exist",
                path.display()
            )?;
            std::process::exit(1);
        }
    }

    let needles = args.get_needle();
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .with_context(|| "Failed to create client to GTDB API".to_string())?;

    for needle in needles {
        // format the request
        let oneedle = needle.clone();
        let search_api = api::Search::new(needle, &options);

        let request_url = search_api.request();

        let response = client
            .get(&request_url)
            .send()
            .with_context(|| "Failed to send request to GTDB API".to_string())?;

        let genomes: SearchResults = response.json().with_context(|| {
            "Failed to deserialize request response to search result structure".to_string()
        })?;

        // Perfom partial match or not?
        match partial {
            true => {
                let genome_list = genomes.rows;
                if genome_list.is_empty() {
                    writeln!(io::stdout(), "No matching data found in GTDB")?;
                    std::process::exit(0);
                }

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
                            file.write_all(&genome_list.len().to_ne_bytes())
                                .with_context(|| format!("Failed to write to {path_clone}"))?;
                            file.write_all("\n".as_bytes())
                                .with_context(|| format!("Failed to write to {path_clone}"))?;
                        } else {
                            writeln!(io::stdout(), "{}", genome_list.len())?;
                        }
                    }

                    // Return only genome id?
                    false => match gid {
                        true => {
                            let list: Vec<Option<String>> =
                                genome_list.iter().map(|x| x.gid.clone()).collect();

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
                                    file.write_all(gid.unwrap_or("null".to_string()).as_bytes())
                                        .with_context(|| {
                                            format!("Failed to write to {path_clone}")
                                        })?;
                                    file.write_all("\n".as_bytes()).with_context(|| {
                                        format!("Failed to write to {path_clone}")
                                    })?;
                                }
                            } else {
                                for gid in list {
                                    writeln!(
                                        io::stdout(),
                                        "{}",
                                        gid.unwrap_or("null".to_string())
                                    )?;
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
                                    for genome in genome_list {
                                        let genome_string = serde_json::to_string(&genome)
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
                                    for genome in genome_list {
                                        let genome_string = serde_json::to_string(&genome)
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
                                    for genome in genome_list {
                                        let genome_string = serde_json::to_string_pretty(&genome)
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
                                    for genome in genome_list {
                                        let genome_string = serde_json::to_string_pretty(&genome)
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
                let genome_list = genomes.search_by_level(&args.get_level(), &oneedle);
                if genome_list.is_empty() {
                    writeln!(io::stdout(), "No matching data found in GTDB.")?;
                    std::process::exit(0);
                }

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
                            file.write_all(&genome_list.len().to_ne_bytes())
                                .with_context(|| format!("Failed to write to {path_clone}"))?;
                            file.write_all("\n".as_bytes())
                                .with_context(|| format!("Failed to write to {path_clone}"))?;
                        } else {
                            writeln!(io::stdout(), "{}", genome_list.len())?;
                        }
                    }

                    // Return only genome id?
                    false => {
                        match gid {
                            true => {
                                let list: Vec<Option<String>> =
                                    genome_list.iter().map(|x| x.gid.clone()).collect();

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
                                        file.write_all(
                                            gid.unwrap_or("null".to_string()).as_bytes(),
                                        )
                                        .with_context(
                                            || format!("Failed to write to {path_clone}"),
                                        )?;
                                        file.write_all("\n".as_bytes()).with_context(|| {
                                            format!("Failed to write to {path_clone}")
                                        })?;
                                    }
                                } else {
                                    let mut stdout_lock = io::stdout().lock();
                                    for gid in list {
                                        writeln!(
                                            stdout_lock,
                                            "{}",
                                            gid.unwrap_or("null".to_string())
                                        )?;
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
                                            for genome in genome_list {
                                                let genome_string = serde_json::to_string(&genome).with_context(|| "Failed to convert search result to json string".to_string())?;
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
                                            for genome in genome_list {
                                                let genome_string = serde_json::to_string(&genome).with_context(|| "Failed to convert search result to json string".to_string())?;
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
                                            for genome in genome_list {
                                                let genome_string = serde_json::to_string_pretty(
                                                    &genome,
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

                                            for genome in genome_list {
                                                let genome_string = serde_json::to_string_pretty(
                                                    &genome,
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

    #[test]
    fn test_genome_get_gtdb_level() {
        let genome = SearchResult {
            gid: Some("test".to_owned()),
            accession: Some("test".to_owned()),
            ncbi_org_name: Some("test".to_owned()),
            ncbi_taxonomy: Some("test".to_owned()),
            gtdb_taxonomy: Some("d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
        };

        let genome1 = SearchResult {
            gid: Some("test".to_owned()),
            accession: Some("test".to_owned()),
            ncbi_org_name: Some("test".to_owned()),
            ncbi_taxonomy: Some("test".to_owned()),
            gtdb_taxonomy: Some("".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
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
            gid: Some("test1".to_owned()),
            accession: Some("test1".to_owned()),
            ncbi_org_name: Some("test1".to_owned()),
            ncbi_taxonomy: Some("test1".to_owned()),
            gtdb_taxonomy: Some("d__Bacteria; p__Actinobacteriota; c__Actinobacteria; o__Actinomycetales; f__Streptomycetaceae; g__Streptomyces; s__".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
        };

        let genome2 = SearchResult {
            gid: Some("test2".to_owned()),
            accession: Some("test2".to_owned()),
            ncbi_org_name: Some("test2".to_owned()),
            ncbi_taxonomy: Some("test2".to_owned()),
            gtdb_taxonomy: Some("d__Bacteria; p__Firmicutes; c__Bacilli; o__Lactobacillales; f__Lactobacillaceae; g__Lactobacillus; s__".to_owned()),
            is_gtdb_species_rep: Some(false),
            is_ncbi_type_material: Some(false),
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
            out: Some(String::from("search2")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search2")).unwrap();
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
            out: Some(String::from("search2")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search2")).unwrap();
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
            out: Some(String::from("search2")),
        };

        assert!(search_gtdb(args).is_ok());
        std::fs::remove_file(Path::new("search2")).unwrap();
    }
}
