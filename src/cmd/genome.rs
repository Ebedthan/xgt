use crate::api::GtdbApiRequest;
use crate::cli::GenomeArgs;
use crate::utils;

use crate::api::GenomeRequestType;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ureq::Agent;

use crate::utils::ToFlatRow;

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

impl ToFlatRow for GenomeCard {
    fn csv_header(sep: &str) -> String {
        format!(
            "accession{sep}name\
             {sep}genome_size{sep}gc_percentage{sep}contig_count{sep}n50_contigs\
             {sep}scaffold_count{sep}n50_scaffolds{sep}ambiguous_bases\
             {sep}checkm_completeness{sep}checkm_contamination{sep}checkm_strain_heterogeneity\
             {sep}protein_count{sep}coding_density\
             {sep}ssu_count{sep}lsu_5s_count{sep}lsu_23s_count\
             {sep}ncbi_assembly_level{sep}ncbi_assembly_name{sep}ncbi_assembly_type\
             {sep}ncbi_bioproject{sep}ncbi_biosample\
             {sep}ncbi_country{sep}ncbi_date{sep}ncbi_genome_category\
             {sep}ncbi_isolate{sep}ncbi_isolation_source{sep}ncbi_lat_lon\
             {sep}ncbi_refseq_category{sep}ncbi_seq_rel_date\
             {sep}ncbi_strain_identifiers{sep}ncbi_submitter\
             {sep}ncbi_taxid{sep}ncbi_species_taxid\
             {sep}ncbi_translation_table{sep}ncbi_version_status\
             {sep}gtdb_representative{sep}gtdb_genome_representative\
             {sep}gtdb_domain{sep}gtdb_phylum{sep}gtdb_class\
             {sep}gtdb_order{sep}gtdb_family{sep}gtdb_genus{sep}gtdb_species\
             {sep}ncbi_taxonomy{sep}ncbi_type_material_designation\
             {sep}gtdb_type_designation{sep}gtdb_type_species_of_genus\
             {sep}lpsn_priority_year{sep}species_rep_name{sep}species_cluster_count\
             {sep}subunit_summary{sep}lpsn_url"
        )
    }

    fn to_flat_row(&self, sep: &str) -> String {
        let n = &self.metadata_nucleotide;
        let g = &self.metadata_gene;
        let ncbi = &self.metadata_ncbi;
        let tax = &self.metadata_taxonomy;
        let tm = &self.metadata_type_material;

        format!(
            "{}{sep}{}\
             {sep}{}{sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}\
             {sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}\
             {sep}{}{sep}{}{sep}{}{sep}{}",
            // genome
            self.genome.accession,
            self.genome.name,
            // metadata_nucleotide
            n.genome_size.map(|v| v.to_string()).unwrap_or_default(),
            n.gc_percentage.map(|v| v.to_string()).unwrap_or_default(),
            n.contig_count.map(|v| v.to_string()).unwrap_or_default(),
            n.n50_contigs.map(|v| v.to_string()).unwrap_or_default(),
            n.scaffold_count.map(|v| v.to_string()).unwrap_or_default(),
            n.n50_scaffolds.map(|v| v.to_string()).unwrap_or_default(),
            n.ambiguous_bases.map(|v| v.to_string()).unwrap_or_default(),
            // metadata_gene
            g.checkm_completeness.as_deref().unwrap_or(""),
            g.checkm_contamination.as_deref().unwrap_or(""),
            g.checkm_strain_heterogeneity.as_deref().unwrap_or(""),
            g.protein_count.as_deref().unwrap_or(""),
            g.coding_density.as_deref().unwrap_or(""),
            g.ssu_count.as_deref().unwrap_or(""),
            g.lsu_5s_count.as_deref().unwrap_or(""),
            g.lsu_23s_count.as_deref().unwrap_or(""),
            // metadata_ncbi
            ncbi.ncbi_assembly_level.as_deref().unwrap_or(""),
            ncbi.ncbi_assembly_name.as_deref().unwrap_or(""),
            ncbi.ncbi_assembly_type.as_deref().unwrap_or(""),
            ncbi.ncbi_bioproject.as_deref().unwrap_or(""),
            ncbi.ncbi_biosample.as_deref().unwrap_or(""),
            ncbi.ncbi_country.as_deref().unwrap_or(""),
            ncbi.ncbi_date.as_deref().unwrap_or(""),
            ncbi.ncbi_genome_category.as_deref().unwrap_or(""),
            ncbi.ncbi_isolate.as_deref().unwrap_or(""),
            ncbi.ncbi_isolation_source.as_deref().unwrap_or(""),
            ncbi.ncbi_lat_lon.as_deref().unwrap_or(""),
            ncbi.ncbi_refseq_category.as_deref().unwrap_or(""),
            ncbi.ncbi_seq_rel_date.as_deref().unwrap_or(""),
            ncbi.ncbi_strain_identifiers.as_deref().unwrap_or(""),
            ncbi.ncbi_submitter.as_deref().unwrap_or(""),
            ncbi.ncbi_taxid.as_deref().unwrap_or(""),
            ncbi.ncbi_species_taxid.as_deref().unwrap_or(""),
            ncbi.ncbi_translation_table.as_deref().unwrap_or(""),
            ncbi.ncbi_version_status.as_deref().unwrap_or(""),
            // metadata_taxonomy
            tax.gtdb_representative.to_string(),
            tax.gtdb_genome_representative.as_deref().unwrap_or(""),
            tax.gtdb_domain.as_deref().unwrap_or(""),
            tax.gtdb_phylum.as_deref().unwrap_or(""),
            tax.gtdb_class.as_deref().unwrap_or(""),
            tax.gtdb_order.as_deref().unwrap_or(""),
            tax.gtdb_family.as_deref().unwrap_or(""),
            tax.gtdb_genus.as_deref().unwrap_or(""),
            tax.gtdb_species.as_deref().unwrap_or(""),
            tax.ncbi_taxonomy.as_deref().unwrap_or(""),
            tax.ncbi_type_material_designation.as_deref().unwrap_or(""),
            // metadata_type_material
            tm.gtdb_type_designation.as_deref().unwrap_or(""),
            tm.gtdb_type_species_of_genus
                .map(|v| v.to_string())
                .unwrap_or_default(),
            tm.lpsn_priority_year
                .map(|v| v.to_string())
                .unwrap_or_default(),
            // top-level GenomeCard fields
            self.species_rep_name.as_deref().unwrap_or(""),
            self.species_cluster_count
                .map(|v| v.to_string())
                .unwrap_or_default(),
            self.subunit_summary.as_deref().unwrap_or(""),
            self.lpsn_url.as_deref().unwrap_or(""),
        )
    }
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

impl ToFlatRow for GenomeMetadata {
    fn csv_header(sep: &str) -> String {
        format!("accession{sep}is_ncbi_surveillance")
    }

    fn to_flat_row(&self, sep: &str) -> String {
        format!(
            "{}{sep}{}",
            self.accession.as_deref().unwrap_or(""),
            self.is_ncbi_surveillance
                .map(|v| v.to_string())
                .unwrap_or_default(),
        )
    }
}

fn fetch_and_save_genome_data<T>(args: &GenomeArgs) -> Result<()>
where
    T: serde::de::DeserializeOwned + serde::Serialize + ToFlatRow,
{
    let accessions = utils::load_input(args, "No accession or file provided".to_string())?;
    let agent = utils::get_agent(args.insecure)?;
    let outfmt = utils::OutputFormat::from(args.outfmt.clone());
    let sep = match outfmt {
        utils::OutputFormat::Tsv => "\t",
        _ => ",",
    };

    // Write header once, truncating any existing file content.
    // For JSON there is no header, but we still need to truncate on the first
    // data write. Handled via `first_write` below.
    if outfmt != utils::OutputFormat::Json {
        let header = T::csv_header(sep);
        utils::write_to_output(format!("{}\n", header).as_bytes(), args.out.clone(), false)?;
    }

    let mut first_write = outfmt == utils::OutputFormat::Json;

    for accession in accessions {
        let request_type = if args.metadata {
            GenomeRequestType::Metadata
        } else {
            GenomeRequestType::Card
        };
        let url = GtdbApiRequest::Genome {
            accession: accession.clone(),
            request_type,
        }
        .to_url();

        let response = utils::fetch_data(
            &agent,
            &url,
            "The server returned an unexpected status code (400)".into(),
        )?;

        let genome_data: T = response.into_body().read_json()?;

        let out = match outfmt {
            utils::OutputFormat::Json => serde_json::to_string_pretty(&genome_data)? + "\n",
            _ => genome_data.to_flat_row(sep) + "\n",
        };

        // Truncate only on the first JSON write; all subsequent writes append.
        let append = !first_write;
        utils::write_to_output(out.as_bytes(), args.out.clone(), append)?;
        first_write = false;
    }

    Ok(())
}

// GTDB Genome history API structs
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
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

pub fn get_genome_metadata(args: &GenomeArgs) -> Result<()> {
    fetch_and_save_genome_data::<GenomeMetadata>(args)
}

pub fn get_genome_card(args: &GenomeArgs) -> Result<()> {
    fetch_and_save_genome_data::<GenomeCard>(args)
}

pub fn get_genome_taxon_history(args: &GenomeArgs) -> Result<()> {
    let accessions = utils::load_input(args, "No accession or file provided".into())?;
    let agent = utils::get_agent(args.insecure)?;
    let outfmt = utils::OutputFormat::from(args.outfmt.clone());
    for acc in accessions {
        process_taxon_history(&acc, &agent, &outfmt, &args.out)?;
    }
    Ok(())
}

fn process_taxon_history(
    accession: &str,
    agent: &Agent,
    outfmt: &utils::OutputFormat,
    out: &Option<String>,
) -> Result<()> {
    let url = GtdbApiRequest::Genome {
        accession: accession.into(),
        request_type: GenomeRequestType::TaxonHistory,
    }
    .to_url();

    let response = utils::fetch_data(
        agent,
        &url,
        "The server returned unexpected response (400)".to_string(),
    )?;

    let records: Vec<History> = response.into_body().read_json()?;
    let changes = compute_taxonomic_changes(&records);

    let content = match outfmt {
        utils::OutputFormat::Json => serde_json::to_string_pretty(&records)?,
        utils::OutputFormat::Tsv => build_csv_string(&records, &changes, "\t"),
        utils::OutputFormat::Csv => build_csv_string(&records, &changes, ","),
    };

    utils::write_to_output(content.as_bytes(), out.clone(), true)
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

fn build_csv_string(
    records: &[History],
    changes: &HashMap<String, Vec<String>>,
    sep: &str,
) -> String {
    let mut lines = vec![format!(
        "release{sep}domain{sep}phylum{sep}family{sep}species{sep}changes"
    )];

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

        lines.push(format!(
            "{}{sep}{}{sep}{}{sep}{}{sep}{}{sep}{}",
            rel,
            rec.d.as_deref().unwrap_or(""),
            rec.p.as_deref().unwrap_or(""),
            rec.f.as_deref().unwrap_or(""),
            rec.s.as_deref().unwrap_or(""),
            change_notes,
        ));
    }

    lines.join("\n") + "\n"
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
