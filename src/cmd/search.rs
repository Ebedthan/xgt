use anyhow::{anyhow, bail, ensure, Result};
use serde::{Deserialize, Serialize};
use ureq::Agent;

use super::utils::{self};

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
    fn filter(&mut self, taxon_name: String, taxon_rank: String) {
        self.rows.retain(|result| {
            result.get_taxon_by_rank(&taxon_rank).unwrap()
                == (taxon_rank.clone() + "__" + &taxon_name)
        });
        self.total_rows = self.rows.len() as u32;
    }
    fn get_total_rows(&self) -> u32 {
        self.total_rows
    }
}

/// Search GTDB's using taxon name with partial match
pub fn partial_search(args: utils::SearchArgs) -> Result<()> {
    let agent: Agent = utils::get_agent(args.get_disable_certificate_verification())?;

    for taxon_name in args.get_taxon_names() {
        let search_api = SearchAPI::from(&taxon_name, &args);
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

        let search_result: SearchResults = response.into_json()?;
        ensure!(
            search_result.get_total_rows() != 0,
            "No matching data found in GTDB"
        );

        if args.get_count() {
            utils::write_to_output(
                search_result.get_total_rows().to_string().as_bytes(),
                args.get_out().clone(),
            )?;
        } else if args.get_gid() {
            let str = search_result
                .rows
                .iter()
                .map(|x| x.gid.clone())
                .collect::<Vec<String>>()
                .join("\n");
            utils::write_to_output(str.as_bytes(), args.get_out().clone())?;
        } else {
            let str = search_result
                .rows
                .iter()
                .map(|x| serde_json::to_string_pretty(x).unwrap())
                .collect::<Vec<String>>()
                .join("\n");
            utils::write_to_output(str.as_bytes(), args.get_out().clone())?;
        }
    }

    Ok(())
}

pub fn exact_search(args: utils::SearchArgs) -> Result<()> {
    let agent: Agent = utils::get_agent(args.get_disable_certificate_verification())?;

    for taxon_name in args.get_taxon_names() {
        let search_api = SearchAPI::from(&taxon_name, &args);
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

        let mut search_result: SearchResults = response.into_json()?;
        search_result.filter(taxon_name.clone(), args.get_taxon_rank(&taxon_name));

        ensure!(
            search_result.get_total_rows() != 0,
            "No matching data found in GTDB"
        );

        if args.get_count() {
            utils::write_to_output(
                search_result.get_total_rows().to_string().as_bytes(),
                args.get_out().clone(),
            )?;
        } else if args.get_gid() {
            let str = search_result
                .rows
                .iter()
                .map(|x| x.gid.clone())
                .collect::<Vec<String>>()
                .join("\n");
            utils::write_to_output(str.as_bytes(), args.get_out().clone())?;
        } else {
            let str = search_result
                .rows
                .iter()
                .map(|x| serde_json::to_string_pretty(x).unwrap())
                .collect::<Vec<String>>()
                .join("\n");
            utils::write_to_output(str.as_bytes(), args.get_out().clone())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        results.filter("Proteobacteria".to_string(), "p".to_string());
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
        args.add_taxon("g__Azorhizobium");
        args.set_count(true);
        args.set_disable_certificate_verification(true);
        args.set_out(Some("test.txt".to_string()));
        let res = exact_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!("6".to_string(), expected);
        std::fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_partial_search_count() {
        let mut args = utils::SearchArgs::new();
        args.add_taxon("g__Azorhizobium");
        args.set_count(true);
        args.set_disable_certificate_verification(true);
        args.set_out(Some("test1.txt".to_string()));
        let res = partial_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test1.txt").unwrap();
        assert_eq!("6".to_string(), expected);
        std::fs::remove_file("test1.txt").unwrap();
    }

    #[test]
    fn test_exact_search_id() {
        let mut args = utils::SearchArgs::new();
        args.add_taxon("g__Azorhizobium");
        args.set_id(true);
        args.set_disable_certificate_verification(true);
        args.set_out(Some("test2.txt".to_string()));
        let res = exact_search(args.clone());
        assert!(res.is_ok());
        let expected = std::fs::read_to_string("test2.txt").unwrap();
        assert_eq!("GCA_023405075.1\nGCA_023448105.1\nGCF_000010525.1\nGCF_000473085.1\nGCF_004364705.1\nGCF_014635325.1".to_string(), expected);
        std::fs::remove_file("test2.txt").unwrap();
    }

    #[test]
    fn test_partial_search_id() {
        let mut args = utils::SearchArgs::new();
        args.add_taxon("g__Azorhizobium");
        args.set_id(true);
        args.set_out(Some("test3.txt".to_string()));
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
        args.add_taxon("s__Azorhizobium doebereinerae");
        args.set_out(Some("test8.txt".to_string()));
        args.set_disable_certificate_verification(true);
        let res = exact_search(args.clone());
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
        args.add_taxon("s__Azorhizobium doebereinerae");
        args.set_out(Some("test9.txt".to_string()));
        args.set_disable_certificate_verification(true);
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
}
