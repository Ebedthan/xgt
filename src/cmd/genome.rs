use super::utils::GenomeArgs;
use crate::api::genome_api::GenomeAPI;
use crate::api::genome_api::GenomeRequestType;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GenomeCard {
    genome: Genome,
    metadata_nucleotide: MetadataNucleotide,
    metadata_gene: MetadataGene,
    metadata_ncbi: MetadataNCBI,
    metadata_type_material: MetadataTypeMaterial,
    #[serde(alias = "metadataTaxonomy")]
    metadata_taxonomy: MetadataTaxonomy,
    #[serde(alias = "gtdbTypeDesignation")]
    gtdb_type_designation: Option<String>,
    subunit_summary: Option<String>,
    #[serde(alias = "speciesRepName")]
    species_rep_name: Option<String>,
    #[serde(alias = "speciesClusterCount")]
    species_cluster_count: Option<i32>,
    #[serde(alias = "lpsnUrl")]
    lpsn_url: Option<String>,
    link_ncbi_taxonomy: Option<String>,
    link_ncbi_taxonomy_unfiltered: Option<String>,
    #[serde(alias = "ncbiTaxonomyFiltered")]
    ncbi_taxonomy_filtered: Vec<Taxon>,
    #[serde(alias = "ncbiTaxonomyUnfiltered")]
    ncbi_taxonomy_unfiltered: Vec<Taxon>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Genome {
    accession: String,
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadata_nucleotide")]
pub struct MetadataNucleotide {
    trna_aa_count: Option<i32>,
    contig_count: Option<i32>,
    n50_contigs: Option<i32>,
    longest_contig: Option<i32>,
    scaffold_count: Option<i32>,
    n50_scaffolds: Option<i32>,
    longest_scaffold: Option<i64>,
    genome_size: Option<i64>,
    gc_percentage: Option<f64>,
    ambiguous_bases: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadata_gene")]
pub struct MetadataGene {
    checkm_completeness: Option<String>,
    checkm_contamination: Option<String>,
    checkm_strain_heterogeneity: Option<String>,
    lsu_5s_count: Option<String>,
    ssu_count: Option<String>,
    lsu_23s_count: Option<String>,
    protein_count: Option<String>,
    coding_density: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadata_ncbi")]
pub struct MetadataNCBI {
    ncbi_genbank_assembly_accession: Option<String>,
    ncbi_strain_identifiers: Option<String>,
    ncbi_assembly_level: Option<String>,
    ncbi_assembly_name: Option<String>,
    ncbi_assembly_type: Option<String>,
    ncbi_bioproject: Option<String>,
    ncbi_biosample: Option<String>,
    ncbi_country: Option<String>,
    ncbi_date: Option<String>,
    ncbi_genome_category: Option<String>,
    ncbi_isolate: Option<String>,
    ncbi_isolation_source: Option<String>,
    ncbi_lat_lon: Option<String>,
    ncbi_molecule_count: Option<String>,
    ncbi_cds_count: Option<String>,
    ncbi_refseq_category: Option<String>,
    ncbi_seq_rel_date: Option<String>,
    ncbi_spanned_gaps: Option<String>,
    ncbi_species_taxid: Option<String>,
    ncbi_ssu_count: Option<String>,
    ncbi_submitter: Option<String>,
    ncbi_taxid: Option<String>,
    ncbi_total_gap_length: Option<String>,
    ncbi_translation_table: Option<String>,
    ncbi_trna_count: Option<String>,
    ncbi_unspanned_gaps: Option<String>,
    ncbi_version_status: Option<String>,
    ncbi_wgs_master: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", rename = "metadata_type_material")]
pub struct MetadataTypeMaterial {
    gtdb_type_designation: Option<String>,
    gtdb_type_designation_sources: Option<String>,
    lpsn_type_designation: Option<String>,
    dsmz_type_designation: Option<String>,
    lpsn_priority_year: Option<i32>,
    gtdb_type_species_of_genus: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadataTaxonomy")]
pub struct MetadataTaxonomy {
    ncbi_taxonomy: Option<String>,
    ncbi_taxonomy_unfiltered: Option<String>,
    gtdb_representative: bool,
    gtdb_genome_representative: Option<String>,
    ncbi_type_material_designation: Option<String>,
    #[serde(alias = "gtdbDomain")]
    gtdb_domain: Option<String>,
    #[serde(alias = "gtdbPhylum")]
    gtdb_phylum: Option<String>,
    #[serde(alias = "gtdbClass")]
    gtdb_class: Option<String>,
    #[serde(alias = "gtdbOrder")]
    gtdb_order: Option<String>,
    #[serde(alias = "gtdbFamily")]
    gtdb_family: Option<String>,
    #[serde(alias = "gtdbGenus")]
    gtdb_genus: Option<String>,
    #[serde(alias = "gtdbSpecies")]
    gtdb_species: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Taxon {
    taxon: Option<String>,
    taxon_id: Option<String>,
}

// GTDB Genome metadata API Struct
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GenomeMetadata {
    accession: Option<String>,
    #[serde(alias = "isNcbiSurveillance")]
    is_ncbi_surveillance: Option<bool>,
}

// GTDB Genome history API structs
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct History {
    release: Option<String>,
    d: Option<String>,
    p: Option<String>,
    c: Option<String>,
    o: Option<String>,
    f: Option<String>,
    g: Option<String>,
    s: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct GenomeTaxonHistory {
    data: Vec<History>,
}

pub fn get_genome_metadata(args: GenomeArgs) -> Result<()> {
    let genome_api: Vec<GenomeAPI> = args
        .get_accession()
        .iter()
        .map(|x| GenomeAPI::from(x.to_string()))
        .collect();
    let raw = args.get_raw();

    let client = reqwest::blocking::Client::builder().build()?;

    for accession in genome_api {
        let request_url = accession.request(GenomeRequestType::Metadata);

        let response = client.get(request_url).send()?;

        let genome: GenomeMetadata = response.json()?;

        match raw {
            true => {
                let genome_string = serde_json::to_string(&genome)?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(genome_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{genome_string}")?;
                }
            }
            false => {
                let genome_string = serde_json::to_string_pretty(&genome)?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(genome_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{genome_string}")?;
                }
            }
        };
    }

    Ok(())
}

pub fn get_genome_card(args: GenomeArgs) -> Result<()> {
    let genome_api: Vec<GenomeAPI> = args
        .get_accession()
        .iter()
        .map(|x| GenomeAPI::from(x.to_string()))
        .collect();
    let raw = args.get_raw();

    let client = reqwest::blocking::Client::builder().build()?;

    for accession in genome_api {
        let request_url = accession.request(GenomeRequestType::Card);

        let response = client.get(request_url).send()?;

        let genome: GenomeCard = response.json()?;

        match raw {
            true => {
                let genome_string = serde_json::to_string(&genome)?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(genome_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{genome_string}")?;
                }
            }
            false => {
                let genome_string = serde_json::to_string_pretty(&genome)?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(genome_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{genome_string}")?;
                }
            }
        };
    }

    Ok(())
}

pub fn get_genome_taxon_history(args: GenomeArgs) -> Result<()> {
    let genome_api: Vec<GenomeAPI> = args
        .get_accession()
        .iter()
        .map(|x| GenomeAPI::from(x.to_string()))
        .collect();
    let raw = args.get_raw();

    let client = reqwest::blocking::Client::builder().build()?;

    for accession in genome_api {
        let request_url = accession.request(GenomeRequestType::TaxonHistory);

        let response = client.get(request_url).send()?;

        let genome: GenomeTaxonHistory = response.json()?;

        match raw {
            true => {
                let genome_string = serde_json::to_string(&genome)?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(genome_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{genome_string}")?;
                }
            }
            false => {
                let genome_string = serde_json::to_string_pretty(&genome)?;
                let output = args.get_output();
                if let Some(path) = output {
                    let path_clone = path.clone();
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path)
                        .with_context(|| format!("Failed to create file {path_clone}"))?;
                    file.write_all(genome_string.as_bytes())
                        .with_context(|| format!("Failed to write to {path_clone}"))?;
                } else {
                    writeln!(io::stdout(), "{genome_string}")?;
                }
            }
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use std::path::Path;

    #[test]
    fn test_genome_gtdb_card_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: false,
            output: None,
        };
        assert!(get_genome_card(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_card_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: true,
            output: None,
        };
        assert!(get_genome_card(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: false,
            output: None,
        };
        assert!(get_genome_metadata(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_out() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: false,
            output: Some(String::from("genome")),
        };
        assert!(get_genome_metadata(args).is_ok());
        std::fs::remove_file(Path::new("genome")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_metadata_out_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: true,
            output: Some(String::from("genome1")),
        };
        assert!(get_genome_metadata(args).is_ok());
        std::fs::remove_file(Path::new("genome1")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_card_out_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: true,
            output: Some(String::from("genome2")),
        };
        assert!(get_genome_card(args).is_ok());
        std::fs::remove_file(Path::new("genome2")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_card_out_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: false,
            output: Some(String::from("genome3")),
        };
        assert!(get_genome_card(args).is_ok());
        std::fs::remove_file(Path::new("genome3")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_tx_out_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: true,
            output: Some(String::from("genome4")),
        };
        assert!(get_genome_taxon_history(args).is_ok());
        std::fs::remove_file(Path::new("genome4")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_tx_out_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: false,
            output: Some(String::from("genome5")),
        };
        assert!(get_genome_taxon_history(args).is_ok());
        std::fs::remove_file(Path::new("genome5")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_metadata_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: true,
            output: None,
        };
        assert!(get_genome_metadata(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_taxon_history_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: false,
            output: None,
        };
        assert!(get_genome_taxon_history(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_taxon_history_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            raw: true,
            output: None,
        };
        assert!(get_genome_taxon_history(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_4() {
        let args = utils::GenomeArgs {
            accession: vec!["".to_owned()],
            raw: true,
            output: None,
        };

        assert!(get_genome_card(args).is_err())
    }

    #[test]
    fn test_response_failure() {
        let args = utils::GenomeArgs {
            accession: vec!["&&&&^^^^^||||".to_owned()],
            raw: true,
            output: None,
        };
        assert!(
            get_genome_card(args).is_err(),
            "Failed to get response from GTDB API"
        );
    }
}
