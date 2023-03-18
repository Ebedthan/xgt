use super::utils::{self, GenomeArgs};
use crate::api::GenomeApi;
use crate::api::GenomeRequestType;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GenomeResult {
    genome: Genome,
    metadata_nucleotide: MetadataNucleotide,
    metadata_gene: MetadataGene,
    metadata_ncbi: MetadataNcbi,
    metadata_type_material: MetadataTypeMaterial,

    #[serde(alias = "metadataTaxonomy")]
    metadata_taxonomy: MetadataTaxonomy,
    #[serde(alias = "gtdbTypeDesignation", deserialize_with = "utils::parse_gtdb")]
    gtdb_type_designation: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    subunit_summary: String,
    #[serde(alias = "speciesRepName", deserialize_with = "utils::parse_gtdb")]
    species_rep_name: String,
    #[serde(alias = "speciesClusterCount")]
    species_cluster_count: i32,
    #[serde(alias = "lpsnUrl", deserialize_with = "utils::parse_gtdb")]
    lpsn_url: String,

    #[serde(deserialize_with = "utils::parse_gtdb")]
    link_ncbi_taxonomy: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    link_ncbi_taxonomy_unfiltered: String,

    #[serde(alias = "ncbiTaxonomyFiltered")]
    ncbi_taxonomy_filtered: Vec<Taxon>,
    #[serde(alias = "ncbiTaxonomyUnfiltered")]
    ncbi_taxonomy_unfiltered: Vec<Taxon>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Genome {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    accession: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadata_nucleotide")]
pub struct MetadataNucleotide {
    trna_aa_count: i32,
    contig_count: i32,
    n50_contigs: i32,
    longest_contig: i32,
    scaffold_count: i32,
    n50_scaffolds: i32,
    longest_scaffold: i64,
    genome_size: i64,
    gc_percentage: f64,
    ambiguous_bases: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadata_gene")]
pub struct MetadataGene {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    checkm_completeness: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    checkm_contamination: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    checkm_strain_heterogeneity: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    lsu_5s_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ssu_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    lsu_23s_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    protein_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    coding_density: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadata_ncbi")]
pub struct MetadataNcbi {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_genbank_assembly_accession: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_strain_identifiers: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_assembly_level: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_assembly_name: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_assembly_type: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_bioproject: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_biosample: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_country: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_date: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_genome_category: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_isolate: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_isolation_source: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_lat_lon: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_molecule_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_cds_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_refseq_category: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_seq_rel_date: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_spanned_gaps: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_species_taxid: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_ssu_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_submitter: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_taxid: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_total_gap_length: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_translation_table: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_trna_count: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_unspanned_gaps: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_version_status: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_wgs_master: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", rename = "metadata_type_material")]
pub struct MetadataTypeMaterial {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    gtdb_type_designation: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    gtdb_type_designation_sources: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    lpsn_type_designation: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    dsmz_type_designation: String,
    lpsn_priority_year: i32,
    gtdb_type_species_of_genus: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename = "metadataTaxonomy")]
pub struct MetadataTaxonomy {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_taxonomy: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_taxonomy_unfiltered: String,
    gtdb_representative: bool,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    gtdb_genome_representative: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    ncbi_type_material_designation: String,

    #[serde(alias = "gtdbDomain", deserialize_with = "utils::parse_gtdb")]
    gtdb_domain: String,
    #[serde(alias = "gtdbPhylum", deserialize_with = "utils::parse_gtdb")]
    gtdb_phylum: String,
    #[serde(alias = "gtdbClass", deserialize_with = "utils::parse_gtdb")]
    gtdb_class: String,
    #[serde(alias = "gtdbOrder", deserialize_with = "utils::parse_gtdb")]
    gtdb_order: String,
    #[serde(alias = "gtdbFamily", deserialize_with = "utils::parse_gtdb")]
    gtdb_family: String,
    #[serde(alias = "gtdbGenus", deserialize_with = "utils::parse_gtdb")]
    gtdb_genus: String,
    #[serde(alias = "gtdbSpecies", deserialize_with = "utils::parse_gtdb")]
    gtdb_species: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Taxon {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    taxon: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    taxon_id: String,
}

// GTDB Genome metadata API Struct
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GenomeMetadata {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    accession: String,
    #[serde(alias = "isNcbiSurveillance")]
    is_ncbi_surveillance: bool,
}

// GTDB Genome history API structs
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct History {
    #[serde(deserialize_with = "utils::parse_gtdb")]
    release: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    d: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    p: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    c: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    o: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    f: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    g: String,
    #[serde(deserialize_with = "utils::parse_gtdb")]
    s: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct TaxonHistory {
    data: Vec<History>,
}

pub fn genome_gtdb(args: GenomeArgs) -> Result<()> {
    // format the request
    let genome_api: Vec<GenomeApi> = args
        .get_accession()
        .iter()
        .map(|x| GenomeApi::from(x.to_string()))
        .collect();
    let request_type = args.get_request_type();
    let raw = args.get_raw();

    if let Some(filename) = args.get_output() {
        let path = Path::new(&filename);
        if path.exists() {
            writeln!(io::stderr(), "error: file should not already exist")?;
            std::process::exit(1);
        }
    }

    for accession in genome_api {
        let request_url = accession.request(request_type);

        let response = reqwest::blocking::get(request_url)
            .with_context(|| "Failed to get response from GTDB API".to_string())?;

        if request_type == GenomeRequestType::Metadata {
            let genome: GenomeMetadata = response.json().with_context(|| {
                "Failed to convert request response to genome metadata structure".to_string()
            })?;

            match raw {
                true => {
                    let genome_string = serde_json::to_string(&genome).with_context(|| {
                        "Failed to convert genome metadata structure to json string".to_string()
                    })?;
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
                    let genome_string =
                        serde_json::to_string_pretty(&genome).with_context(|| {
                            "Failed to convert genome metadata structure to json string".to_string()
                        })?;
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
        } else if request_type == GenomeRequestType::TaxonHistory {
            let genome: TaxonHistory = response.json().with_context(|| {
                "Failed to convert request response to genome metadata structure".to_string()
            })?;
            match raw {
                true => {
                    let genome_string = serde_json::to_string(&genome).with_context(|| {
                        "Failed to convert genome metadata structure to json string".to_string()
                    })?;
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
                    let genome_string =
                        serde_json::to_string_pretty(&genome).with_context(|| {
                            "Failed to convert genome metadata structure to json string".to_string()
                        })?;
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
        } else {
            let genome: GenomeResult = response.json().with_context(|| {
                "Failed to convert genome card structure to json string".to_string()
            })?;

            match raw {
                true => {
                    let genome_string = serde_json::to_string(&genome).with_context(|| {
                        "Failed to convert genome card structure to json string".to_string()
                    })?;
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
                    let genome_string =
                        serde_json::to_string_pretty(&genome).with_context(|| {
                            "Failed to convert genome card structure to json string".to_string()
                        })?;
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
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_genome_gtdb_card_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Card,
            raw: false,
            output: None,
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_card_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Card,
            raw: true,
            output: None,
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Metadata,
            raw: false,
            output: None,
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_out() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().as_os_str().to_str().unwrap();
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Metadata,
            raw: false,
            output: Some(String::from(path)),
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_out_1() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().as_os_str().to_str().unwrap();
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Metadata,
            raw: true,
            output: Some(String::from(path)),
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_card_out_1() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().as_os_str().to_str().unwrap();
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Card,
            raw: true,
            output: Some(String::from(path)),
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_card_out_2() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().as_os_str().to_str().unwrap();
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Card,
            raw: false,
            output: Some(String::from(path)),
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_tx_out_1() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().as_os_str().to_str().unwrap();
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::TaxonHistory,
            raw: true,
            output: Some(String::from(path)),
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_tx_out_2() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().as_os_str().to_str().unwrap();
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::TaxonHistory,
            raw: false,
            output: Some(String::from(path)),
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::Metadata,
            raw: true,
            output: None,
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_taxon_history_1() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::TaxonHistory,
            raw: false,
            output: None,
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_taxon_history_2() {
        let args = utils::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            request_type: GenomeRequestType::TaxonHistory,
            raw: true,
            output: None,
        };
        assert!(genome_gtdb(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_4() {
        let args = utils::GenomeArgs {
            accession: vec!["".to_owned()],
            request_type: GenomeRequestType::Card,
            raw: true,
            output: None,
        };

        assert!(genome_gtdb(args).is_err())
    }

    #[test]
    fn test_response_failure() {
        let args = utils::GenomeArgs {
            accession: vec!["&&&&^^^^^||||".to_owned()],
            request_type: GenomeRequestType::Card,
            raw: true,
            output: None,
        };
        assert!(
            genome_gtdb(args).is_err(),
            "Failed to get response from GTDB API"
        );
    }

    #[test]
    fn test_genome_metadata_json_err() {
        let args = utils::GenomeArgs {
            accession: vec!["".to_owned()],
            request_type: GenomeRequestType::Metadata,
            raw: true,
            output: None,
        };

        assert!(
            genome_gtdb(args).is_err(),
            "Failed to convert genome metadata structure to json string"
        );
    }
}
