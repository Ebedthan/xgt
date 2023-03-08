use std::io;
use std::io::Write;

use anyhow::Result;
use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
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
        let fields: Vec<&str> = self.gtdb_taxonomy.split(";").collect();

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

pub fn search_gtdb(needle: &str, level: &str, exact: bool, count: bool) -> Result<(), Error> {
    let request_url = format!("https://api.gtdb.ecogenomic.org/search/gtdb?search={needle}&page=1&itemsPerPage=100&searchField=gtdb_tax&gtdbSpeciesRepOnly=false&ncbiTypeMaterialOnly=false", needle = needle);

    let response = reqwest::blocking::get(&request_url)?;

    let genomes: SearchResult = response.json()?;
    let mut genome_list = Vec::new();

    match exact {
        true => genome_list = genomes.search_by_level(level, needle),
        false => genome_list = genomes.rows,
    }

    match count {
        true => writeln!(io::stdout(), "{}", genome_list.len()).unwrap(),
        false => {
            for genome in genome_list {
                let g = serde_json::to_string(&genome).unwrap();
                writeln!(io::stdout(), "{g}").unwrap();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gtdb_level() {
        let str = "d__Bacteria; p__Proteobacteria; c__Alphaproteobacteria; o__Rhizobiales; f__Xanthobacteraceae; g__Bradyrhizobium; s__Bradyrhizobium sp003020075".to_string();
        let g = Genome {
            gid: String::from(""),
            accession: String::from(""),
            ncbi_org_name: String::from(""),
            ncbi_taxonomy: String::from(""),
            gtdb_taxonomy: str,
            is_gtdb_species_rep: false,
            is_ncbi_type_material: true,
        };
        assert_eq!(g.get_gtdb_level("domain"), "Bacteria".to_string());
        assert_eq!(g.get_gtdb_level("phylum"), "Proteobacteria".to_string());
        assert_eq!(g.get_gtdb_level("class"), "Alphaproteobacteria".to_string());
        assert_eq!(g.get_gtdb_level("order"), "Rhizobiales".to_string());
        assert_eq!(g.get_gtdb_level("family"), "Xanthobacteraceae".to_string());
        assert_eq!(g.get_gtdb_level("genus"), "Bradyrhizobium".to_string());
        assert_eq!(
            g.get_gtdb_level("species"),
            "Bradyrhizobium sp003020075".to_string()
        );
    }
}
