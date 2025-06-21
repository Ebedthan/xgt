use crate::api::GtdbApiRequest;
use crate::cli::GenomeArgs;
use crate::utils;

use crate::api::GenomeRequestType;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{self, Write},
};
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
) -> Result<()> {
    let accessions = utils::load_input(args, "No accession or file provided".to_string())?;
    let agent: Agent = utils::get_agent(args.insecure)?;
    for accession in accessions {
        let request_url = if args.metadata {
            let genome = GtdbApiRequest::Genome {
                accession: accession.to_string(),
                request_type: crate::api::GenomeRequestType::Metadata,
            };
            genome.to_url()
        } else {
            let genome = GtdbApiRequest::Genome {
                accession: accession.to_string(),
                request_type: crate::api::GenomeRequestType::Card,
            };
            genome.to_url()
        };
        let response = utils::fetch_data(
            &agent,
            &request_url,
            "The server returned an unexpected status code (400)".into(),
        )?;
        let genome_data: T = response.into_json()?;
        let genome_string = serde_json::to_string_pretty(&genome_data)?;
        if let Some(path) = &args.out {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .with_context(|| format!("Failed to create file {}", path))?;
            writeln!(file, "{}", genome_string)
                .with_context(|| format!("Failed to write to {}", path))?;
        } else {
            writeln!(io::stdout(), "{}", genome_string)?;
        }
    }
    Ok(())
}

pub fn get_genome_metadata(args: &GenomeArgs) -> Result<()> {
    fetch_and_save_genome_data::<GenomeMetadata>(args)
}

pub fn get_genome_card(args: &GenomeArgs) -> Result<()> {
    fetch_and_save_genome_data::<GenomeCard>(args)
}

pub fn get_genome_taxon_history(args: &GenomeArgs) -> Result<()> {
    let accessions = utils::load_input(args, "No accession or file provided".into())?;
    let agent = utils::get_agent(args.insecure)?;
    for acc in accessions {
        process_taxon_history(&acc, &agent, &args.out)?;
    }
    Ok(())
}

fn process_taxon_history(accession: &str, agent: &Agent, out: &Option<String>) -> Result<()> {
    let genome_api = GtdbApiRequest::Genome {
        accession: accession.into(),
        request_type: GenomeRequestType::TaxonHistory,
    };
    let url = genome_api.to_url();
    let response = utils::fetch_data(
        agent,
        &url,
        "The server returned unexpected response (400)".to_string(),
    )?;

    let records: Vec<History> = response.into_json()?;
    let changes = compute_taxonomic_changes(&records);

    if let Some(path) = out {
        write_csv_output(path, &records, &changes)?;
    } else {
        print_timeline(accession, &records, &changes);
    }

    Ok(())
}

fn compute_taxonomic_changes(records: &[History]) -> HashMap<String, Vec<String>> {
    let mut changes = HashMap::new();
    let mut prev: Option<&History> = None;

    for rec in records.iter().rev() {
        if let Some(last) = prev {
            let mut notes = Vec::new();
            compare_field(&last.d, &rec.d, "Domain", &mut notes);
            compare_field(&last.p, &rec.p, "Phylum", &mut notes);
            compare_field(&last.f, &rec.f, "Family", &mut notes);
            compare_field(&last.s, &rec.s, "Species", &mut notes);

            if !notes.is_empty() {
                if let Some(release) = &rec.release {
                    changes.insert(release.clone(), notes);
                }
            }
        }
        prev = Some(rec);
    }

    changes
}

fn write_csv_output(
    path: &str,
    records: &[History],
    changes: &HashMap<String, Vec<String>>,
) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .with_context(|| format!("Failed to open output file: {}", path))?;
    writeln!(file, "release,domain,phylum,family,species,changes")?;

    for (i, rec) in records.iter().enumerate() {
        let is_first = i == records.len() - 1;
        let rel = rec.release.as_deref().unwrap_or("");
        let change_notes = if changes.contains_key(rel) {
            changes[rel].join("; ")
        } else if is_first {
            "initial classification".to_string()
        } else {
            String::new()
        };

        writeln!(
            file,
            "{},{},{},{},{},{}",
            rel,
            rec.d.as_deref().unwrap_or(""),
            rec.p.as_deref().unwrap_or(""),
            rec.f.as_deref().unwrap_or(""),
            rec.s.as_deref().unwrap_or(""),
            change_notes
        )?;
    }

    Ok(())
}

fn print_timeline(accession: &str, records: &[History], changes: &HashMap<String, Vec<String>>) {
    println!(
        "## Genome {} Classification Timeline (Newest → Oldest)\n",
        accession
    );

    for (i, rec) in records.iter().enumerate() {
        let is_first = i == records.len() - 1;
        let rel = rec.release.as_deref().unwrap_or("");
        let has_changes = changes.contains_key(rel);

        if is_first || has_changes {
            println!("### {}", rel);
            println!("- **Taxonomy**:");
            print_field("Domain", &rec.d);
            print_field("Phylum", &rec.p);
            print_field("Family", &rec.f);
            print_field("Species", &rec.s);

            if has_changes {
                println!("- **Changes**:");
                for note in &changes[rel] {
                    println!("  - {}", note);
                }
            } else if is_first {
                println!("- Initial classification.");
            }

            println!();
        }
    }
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
            notes.push(format!("{}: {} -> {}", name, prev_val, current_val));
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
        println!("  - {}: {}", name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compare_field_changes() {
        let mut notes = vec![];
        compare_field(
            &Some("A".to_string()),
            &Some("B".to_string()),
            "Domain",
            &mut notes,
        );
        assert_eq!(notes, vec!["Domain: A -> B"]);

        let mut notes = vec![];
        compare_field(&Some("A".to_string()), &None, "Domain", &mut notes);
        assert_eq!(notes, vec!["Domain removed (was A)"]);

        let mut notes = vec![];
        compare_field(&None, &Some("B".to_string()), "Domain", &mut notes);
        assert_eq!(notes, vec!["Domain added: B"]);

        let mut notes = vec![];
        compare_field(
            &Some("A".to_string()),
            &Some("A".to_string()),
            "Domain",
            &mut notes,
        );
        assert!(notes.is_empty());
    }

    #[test]
    fn test_compute_taxonomic_changes() {
        let records = vec![
            History {
                release: Some("R1".into()),
                d: Some("Bacteria".into()),
                p: Some("Firmicutes".into()),
                c: None,
                o: None,
                f: Some("Lactobacillaceae".into()),
                g: None,
                s: Some("SpeciesA".into()),
            },
            History {
                release: Some("R2".into()),
                d: Some("Bacteria".into()),
                p: Some("Firmicutes".into()),
                c: None,
                o: None,
                f: Some("Lactobacillaceae".into()),
                g: None,
                s: Some("SpeciesB".into()), // changed species
            },
        ];

        let changes = compute_taxonomic_changes(&records);
        assert!(changes.contains_key("R1")); // from R2 to R1 (in reverse)
        let notes = changes.get("R1").unwrap();
        assert_eq!(notes, &vec!["Species: SpeciesB -> SpeciesA"]);
    }

    #[test]
    fn test_write_csv_output() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let records = vec![History {
            release: Some("R1".into()),
            d: Some("Bacteria".into()),
            p: Some("Firmicutes".into()),
            c: None,
            o: None,
            f: Some("Lactobacillaceae".into()),
            g: None,
            s: Some("SpeciesA".into()),
        }];

        let changes: HashMap<String, Vec<String>> = HashMap::new();
        write_csv_output(&path, &records, &changes).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("release,domain,phylum,family,species,changes"));
        assert!(content.contains("initial classification"));
    }
}
