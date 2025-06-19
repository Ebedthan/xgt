use crate::api::genome::GenomeAPI;
use crate::api::genome::GenomeRequestType;
use crate::cli::genome::GenomeArgs;
use crate::utils;

use anyhow::anyhow;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};

use ureq::Agent;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
/// GenomeCard API query result struct
pub struct GenomeCard {
    // Genome struct
    genome: Genome,

    // MetadataNucleotide struct
    metadata_nucleotide: MetadataNucleotide,

    // MetadataGene struct
    metadata_gene: MetadataGene,

    // MetadataNCBI struct
    metadata_ncbi: MetadataNCBI,

    // MetadataTypeMaterial struct
    metadata_type_material: MetadataTypeMaterial,

    // MetadataTaxonomy struct
    #[serde(alias = "metadataTaxonomy")]
    metadata_taxonomy: MetadataTaxonomy,

    // String to specify if it is a type material or not
    // for example: "not type material"
    #[serde(alias = "gtdbTypeDesignation")]
    gtdb_type_designation: Option<String>,

    subunit_summary: Option<String>,

    // Representative species name of this genome
    // for example: "GCA_000010525.1"
    #[serde(alias = "speciesRepName")]
    species_rep_name: Option<String>,

    #[serde(alias = "speciesClusterCount")]
    species_cluster_count: Option<i32>,

    // Link to Genome page on LPSN if any
    // for example: "https://lpsn.dsmz.de/species/azorhizobium-caulinodans"
    #[serde(alias = "lpsnUrl")]
    lpsn_url: Option<String>,

    // Parsed link to NCBI Taxonomy of Genome if any
    // for example: "<a target=\"_blank\" href=\"https://www.ncbi.nlm.nih.gov/data-hub/taxonomy/2/\">d__Bacteria</a>; <a target=\"_blank\" href=\"https://www.ncbi.nlm.nih.gov/data-hub/taxonomy/1224/\">p__Pseudomonadota</a>; c__; o__; f__; g__; s__"
    link_ncbi_taxonomy: Option<String>,

    // Raw link to NCBI Taxonomy of Genome if any
    // for example: "<a target=\"_blank\" href=\"https://www.ncbi.nlm.nih.gov/data-hub/taxonomy/2/\">d__Bacteria</a>; <a target=\"_blank\" href=\"https://www.ncbi.nlm.nih.gov/data-hub/taxonomy/1224/\">p__Pseudomonadota</a>; <a target=\"_blank\" href=\"https://www.ncbi.nlm.nih.gov/data-hub/taxonomy/81684/\">x__unclassified Pseudomonadota</a>; <a target=\"_blank\" href=\"https://www.ncbi.nlm.nih.gov/data-hub/taxonomy/1977087/\">s__Pseudomonadota bacterium</a>"
    link_ncbi_taxonomy_unfiltered: Option<String>,

    // Parsed NCBI taxonomy as a Vec of Taxon struct
    #[serde(alias = "ncbiTaxonomyFiltered")]
    ncbi_taxonomy_filtered: Vec<Taxon>,

    // Raw NCBI Taxonomy as a Vec of Taxon struct
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

fn fetch_and_save_genome_data<T: serde::de::DeserializeOwned + serde::Serialize>(
    args: &GenomeArgs,
    request_type: GenomeRequestType,
) -> Result<()> {
    let genome_api: Vec<GenomeAPI> = args
        .get_accession()
        .iter()
        .map(|x| GenomeAPI::from(x.to_string()))
        .collect();
    let agent: Agent = utils::get_agent(args.get_disable_certificate_verification())?;
    for accession in genome_api {
        let request_url = accession.request(request_type);
        let response = agent.get(&request_url).call().map_err(|e| match e {
            ureq::Error::Status(code, _) => {
                anyhow!("The server returned an unexpected status code ({})", code)
            }
            _ => anyhow!(
                "There was an error making the request or receiving the response\n{}",
                e
            ),
        })?;
        let genome_data: T = response.into_json()?;
        let genome_string = serde_json::to_string_pretty(&genome_data)?;
        if let Some(path) = args.get_output() {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .with_context(|| format!("Failed to create file {}", path))?;
            writeln!(file, "{}", genome_string)
                .with_context(|| format!("Failed to write to {}", path))?;
        } else {
            writeln!(io::stdout(), "{}", genome_string)?;
        }
    }
    Ok(())
}

pub fn get_genome_metadata(args: GenomeArgs) -> Result<()> {
    fetch_and_save_genome_data::<GenomeMetadata>(&args, GenomeRequestType::Metadata)
}

pub fn get_genome_card(args: GenomeArgs) -> Result<()> {
    fetch_and_save_genome_data::<GenomeCard>(&args, GenomeRequestType::Card)
}

pub fn get_genome_taxon_history(args: GenomeArgs) -> Result<()> {
    let genome_api: Vec<GenomeAPI> = args
        .get_accession()
        .iter()
        .map(|x| GenomeAPI::from(x.to_string()))
        .collect();
    let agent: Agent = utils::get_agent(args.get_disable_certificate_verification())?;
    for accession in genome_api {
        let request_url = accession.request(GenomeRequestType::TaxonHistory);
        let response = agent.get(&request_url).call().map_err(|e| match e {
            ureq::Error::Status(code, _) => {
                anyhow!("The server returned an unexpected status code ({})", code)
            }
            _ => anyhow!(
                "There was an error making the request or receiving the response\n{}",
                e
            ),
        })?;
        let genome_data: Vec<History> = response.into_json()?;
        let mut changes = HashMap::new();
        let mut prev_record: Option<&History> = None;
        for record in genome_data.iter().rev() {
            if let Some(prev) = prev_record {
                let mut change_notes = Vec::new();

                // Compare each taxonomic rank (only if both current and previous values exist)
                compare_field(&prev.d, &record.d, "Domain", &mut change_notes);
                compare_field(&prev.p, &record.p, "Phylum", &mut change_notes);
                compare_field(&prev.f, &record.f, "Family", &mut change_notes);
                compare_field(&prev.s, &record.s, "Species", &mut change_notes);

                if !change_notes.is_empty() {
                    if let Some(release) = &record.release {
                        changes.insert(release.clone(), change_notes);
                    }
                }
            }
            prev_record = Some(record);
        }
        if let Some(path) = args.get_output() {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .with_context(|| format!("Failed to create file {}", path))?;
            writeln!(file, "release,domain,phylum,family,species,changes")?;
            for (i, record) in genome_data.iter().enumerate() {
                let is_first = i == genome_data.len() - 1;
                let has_changes = record
                    .release
                    .as_ref()
                    .is_some_and(|r| changes.contains_key(r));

                if is_first || has_changes {
                    let changes_str = if has_changes {
                        changes
                            .get(&record.release.clone().unwrap())
                            .unwrap()
                            .join("; ")
                    } else if is_first {
                        "initial classification".to_string()
                    } else {
                        String::new()
                    };

                    writeln!(
                        file,
                        "{},{},{},{},{},{}",
                        record.release.as_deref().unwrap_or(""),
                        record.d.as_deref().unwrap_or(""),
                        record.p.as_deref().unwrap_or(""),
                        record.f.as_deref().unwrap_or(""),
                        record.s.as_deref().unwrap_or(""),
                        changes_str
                    )?;
                }
            }
        } else {
            // Generate timeline output (Markdown)
            println!(
                "## Genome {} Classification Timeline (Newest → Oldest)\n",
                accession.get_accession()
            );
            for (i, record) in genome_data.iter().enumerate() {
                if let Some(release) = &record.release {
                    let is_first = i == genome_data.len() - 1;
                    let has_changes = changes.contains_key(release);

                    if is_first || has_changes {
                        println!("### {}", release);
                        println!("- **Taxonomy**:");
                        print_field("Domain", &record.d);
                        print_field("Phylum", &record.p);
                        print_field("Family", &record.f);
                        print_field("Species", &record.s);

                        if has_changes {
                            println!("- **Changes**:");
                            for note in changes.get(release).unwrap() {
                                println!("  - {}", note);
                            }
                        } else if is_first {
                            println!("- Initial classification.");
                        }
                        println!();
                    }
                }
            }
        }
    }

    Ok(())
}

// Helper to compare fields
fn compare_field(
    prev: &Option<String>,
    current: &Option<String>,
    name: &str,
    notes: &mut Vec<String>,
) {
    match (prev, current) {
        (Some(prev_val), Some(current_val)) if prev_val != current_val => {
            notes.push(format!("{}: {} → {}", name, prev_val, current_val));
        }
        (Some(prev_val), None) => {
            notes.push(format!("{} removed (was {})", name, prev_val));
        }
        (None, Some(current_val)) => {
            notes.push(format!("{} added: {}", name, current_val));
        }
        _ => {}
    }
}

// Helper: Print a field if it exists
fn print_field(name: &str, field: &Option<String>) {
    if let Some(value) = field {
        println!("  - {}: `{}`", name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::genome;
    use std::path::Path;

    #[test]
    fn test_genome_gtdb_card_1() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };
        assert!(get_genome_card(args.clone()).is_ok());
    }

    #[test]
    fn test_genome_gtdb_card_2() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };
        assert!(get_genome_card(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_1() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };
        assert!(get_genome_metadata(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_metadata_out() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: Some(String::from("genome")),
            disable_certificate_verification: true,
        };
        assert!(get_genome_metadata(args).is_ok());
        std::fs::remove_file(Path::new("genome")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_metadata_out_1() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: Some(String::from("genome1")),
            disable_certificate_verification: true,
        };
        assert!(get_genome_metadata(args).is_ok());
        std::fs::remove_file(Path::new("genome1")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_card_out_1() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: Some(String::from("genome2")),
            disable_certificate_verification: true,
        };
        assert!(get_genome_card(args).is_ok());
        std::fs::remove_file(Path::new("genome2")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_card_out_2() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: Some(String::from("genome3")),
            disable_certificate_verification: true,
        };
        assert!(get_genome_card(args).is_ok());
        std::fs::remove_file(Path::new("genome3")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_tx_out_1() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: Some(String::from("genome4")),
            disable_certificate_verification: true,
        };
        assert!(get_genome_taxon_history(args).is_ok());
        std::fs::remove_file(Path::new("genome4")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_tx_out_2() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: Some(String::from("genome5")),
            disable_certificate_verification: true,
        };
        assert!(get_genome_taxon_history(args).is_ok());
        std::fs::remove_file(Path::new("genome5")).unwrap();
    }

    #[test]
    fn test_genome_gtdb_metadata_2() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };
        assert!(get_genome_metadata(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_taxon_history_1() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };
        assert!(get_genome_taxon_history(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_taxon_history_2() {
        let args = genome::GenomeArgs {
            accession: vec!["GCA_001512625.1".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };
        assert!(get_genome_taxon_history(args).is_ok());
    }

    #[test]
    fn test_genome_gtdb_4() {
        let args = genome::GenomeArgs {
            accession: vec!["".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };

        assert!(get_genome_card(args).is_err())
    }

    #[test]
    fn test_response_failure() {
        let args = genome::GenomeArgs {
            accession: vec!["&&&&^^^^^||||".to_owned()],
            output: None,
            disable_certificate_verification: true,
        };
        assert!(
            get_genome_card(args).is_err(),
            "Failed to get response from GTDB API"
        );
    }
}
