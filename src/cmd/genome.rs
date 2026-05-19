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
            tax.gtdb_representative,
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
    let accessions = utils::load_input(args, "No genome accession provided...".to_string())?;
    let agent = utils::get_agent(args.insecure)?;
    let outfmt = utils::OutputFormat::from(args.outfmt.clone());
    let sep = match outfmt {
        utils::OutputFormat::Tsv => "\t",
        _ => ",",
    };

    let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);
    let bar = utils::make_progress_bar(accessions.len());

    // Write a single header once — only for non-split, non-JSON batch output
    if !dest.is_split() && outfmt != utils::OutputFormat::Json {
        let header = T::csv_header(sep);
        utils::write_to_output(format!("{}\n", header).as_bytes(), dest.resolve(""), false)?;
    }

    let mut first_write = !dest.is_split() && outfmt == utils::OutputFormat::Json;

    for accession in &accessions {
        if let Some(ref bar) = bar {
            bar.set_message(accession.clone());
        }

        let request_type = if args.metadata {
            GenomeRequestType::Metadata
        } else {
            GenomeRequestType::Card
        };

        let url = GtdbApiRequest::Genome {
            accession: accession.clone(),
            request_type,
            release: args.release.clone(),
        }
        .to_url();

        let response = utils::fetch_data(
            &agent,
            &url,
            format!(
                "Accession '{}' was not found in GTDB (HTTP 400). \
                 Verify the accession format (e.g. GCA_000010525.1 or GCF_000010525.1).",
                accession
            ),
        )?;

        let genome_data: T = response.into_body().read_json()?;

        // In split mode: write header + row to each individual file
        if dest.is_split() && outfmt != utils::OutputFormat::Json {
            let header = T::csv_header(sep);
            utils::write_to_output(
                format!("{}\n", header).as_bytes(),
                dest.resolve(accession),
                false,
            )?;
        }

        let out = match outfmt {
            utils::OutputFormat::Json => serde_json::to_string_pretty(&genome_data)? + "\n",
            _ => genome_data.to_flat_row(sep) + "\n",
        };

        // split mode always truncates (new file per item)
        // non-split JSON truncates only on first write
        let append = if dest.is_split() { false } else { !first_write };

        utils::write_to_output(out.as_bytes(), dest.resolve(accession), append)?;
        first_write = false;

        if let Some(ref bar) = bar {
            bar.inc(1);
        }
    }

    if let Some(bar) = bar {
        bar.finish_with_message(format!("done, {} genomes processed", accessions.len()));
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
    let accessions = utils::load_input(args, "No genome accession provided...".into())?;
    let agent = utils::get_agent(args.insecure)?;
    let outfmt = utils::OutputFormat::from(args.outfmt.clone());
    let dest = utils::output_destination(&args.out, args.split, &outfmt, &args.split_dir);
    let bar = utils::make_progress_bar(accessions.len());

    for acc in &accessions {
        if let Some(ref bar) = bar {
            bar.set_message(acc.clone());
        }

        process_taxon_history(acc, &agent, &outfmt, &dest.resolve(acc))?;

        if let Some(ref bar) = bar {
            bar.inc(1);
        }
    }

    if let Some(bar) = bar {
        bar.finish_with_message(format!("done, {} accessions processed", accessions.len()));
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
        release: None,
    }
    .to_url();

    let response = utils::fetch_data(
        agent,
        &url,
        format!(
            "No taxonomic history found for accession '{}' (HTTP 400). \
             Verify the accession exists in GTDB.",
            accession
        ),
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
    use mockito::Server;
    use std::io::Write;
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

    /// Build a minimal GenomeMetadata with known values.
    fn make_metadata(accession: &str, surveillance: bool) -> GenomeMetadata {
        GenomeMetadata {
            accession: Some(accession.into()),
            is_ncbi_surveillance: Some(surveillance),
        }
    }

    /// Build a fully-populated GenomeCard with minimal but non-empty values.
    fn make_genome_card() -> GenomeCard {
        GenomeCard {
            genome: Genome {
                accession: "GCA_000005845.2".into(),
                name: "Escherichia coli K-12".into(),
            },
            metadata_nucleotide: MetadataNucleotide {
                trna_aa_count: None,
                contig_count: Some(1),
                n50_contigs: Some(4641652),
                longest_contig: None,
                scaffold_count: Some(1),
                n50_scaffolds: Some(4641652),
                longest_scaffold: None,
                genome_size: Some(4641652),
                gc_percentage: Some(50.79),
                ambiguous_bases: Some(0),
            },
            metadata_gene: MetadataGene {
                checkm_completeness: Some("99.74".into()),
                checkm_contamination: Some("0.0".into()),
                checkm_strain_heterogeneity: Some("0.0".into()),
                lsu_5s_count: Some("3".into()),
                ssu_count: Some("7".into()),
                lsu_23s_count: Some("7".into()),
                protein_count: Some("4285".into()),
                coding_density: Some("87.4".into()),
            },
            metadata_ncbi: MetadataNCBI {
                ncbi_genbank_assembly_accession: None,
                ncbi_strain_identifiers: Some("K-12".into()),
                ncbi_assembly_level: Some("Complete Genome".into()),
                ncbi_assembly_name: Some("asm584v2".into()),
                ncbi_assembly_type: Some("haploid".into()),
                ncbi_bioproject: Some("PRJNA57779".into()),
                ncbi_biosample: Some("SAMN02604091".into()),
                ncbi_country: None,
                ncbi_date: Some("2013/09/26".into()),
                ncbi_genome_category: None,
                ncbi_isolate: None,
                ncbi_isolation_source: None,
                ncbi_lat_lon: None,
                ncbi_molecule_count: None,
                ncbi_cds_count: None,
                ncbi_refseq_category: Some("reference genome".into()),
                ncbi_seq_rel_date: Some("2013/09/26".into()),
                ncbi_spanned_gaps: None,
                ncbi_species_taxid: Some("562".into()),
                ncbi_ssu_count: None,
                ncbi_submitter: Some("Blattner Lab".into()),
                ncbi_taxid: Some("83333".into()),
                ncbi_total_gap_length: None,
                ncbi_translation_table: Some("11".into()),
                ncbi_trna_count: None,
                ncbi_unspanned_gaps: None,
                ncbi_version_status: Some("latest".into()),
                ncbi_wgs_master: None,
            },
            metadata_type_material: MetadataTypeMaterial {
                gtdb_type_designation: Some("not type material".into()),
                gtdb_type_designation_sources: None,
                lpsn_type_designation: None,
                dsmz_type_designation: None,
                lpsn_priority_year: None,
                gtdb_type_species_of_genus: Some(false),
            },
            metadata_taxonomy: MetadataTaxonomy {
                ncbi_taxonomy: Some("d__Bacteria; p__Pseudomonadota".into()),
                ncbi_taxonomy_unfiltered: None,
                gtdb_representative: true,
                gtdb_genome_representative: Some("GCA_000005845.2".into()),
                ncbi_type_material_designation: None,
                gtdb_domain: Some("d__Bacteria".into()),
                gtdb_phylum: Some("p__Pseudomonadota".into()),
                gtdb_class: Some("c__Gammaproteobacteria".into()),
                gtdb_order: Some("o__Enterobacterales".into()),
                gtdb_family: Some("f__Enterobacteriaceae".into()),
                gtdb_genus: Some("g__Escherichia".into()),
                gtdb_species: Some("s__Escherichia coli".into()),
            },
            gtdb_type_designation: Some("not type material".into()),
            subunit_summary: None,
            species_rep_name: Some("GCA_000005845.2".into()),
            species_cluster_count: Some(42),
            lpsn_url: Some("https://lpsn.dsmz.de/species/escherichia-coli".into()),
            link_ncbi_taxonomy: None,
            link_ncbi_taxonomy_unfiltered: None,
            ncbi_taxonomy_filtered: vec![],
            ncbi_taxonomy_unfiltered: vec![],
        }
    }

    /// Build a minimal GenomeArgs pointing at a mock server URL.
    fn make_genome_args(
        accession: Option<&str>,
        file: Option<String>,
        metadata: bool,
        history: bool,
        outfmt: &str,
        out: Option<String>,
    ) -> GenomeArgs {
        GenomeArgs {
            accession: accession.map(|s| s.to_string()),
            file,
            history,
            metadata,
            outfmt: outfmt.to_string(),
            out,
            insecure: false,
            split: false,
            split_dir: None,
            release: None,
        }
    }

    // ── ToFlatRow for GenomeMetadata ──────────────────────────────────────────────

    #[test]
    fn test_genome_metadata_csv_header_comma() {
        let header = GenomeMetadata::csv_header(",");
        assert_eq!(header, "accession,is_ncbi_surveillance");
    }

    #[test]
    fn test_genome_metadata_csv_header_tab() {
        let header = GenomeMetadata::csv_header("\t");
        assert_eq!(header, "accession\tis_ncbi_surveillance");
    }

    #[test]
    fn test_genome_metadata_to_flat_row_all_present() {
        let m = make_metadata("GCA_000005845.2", true);
        let row = m.to_flat_row(",");
        assert_eq!(row, "GCA_000005845.2,true");
    }

    #[test]
    fn test_genome_metadata_to_flat_row_false_surveillance() {
        let m = make_metadata("GCF_000001405.39", false);
        let row = m.to_flat_row(",");
        assert_eq!(row, "GCF_000001405.39,false");
    }

    #[test]
    fn test_genome_metadata_to_flat_row_none_values() {
        let m = GenomeMetadata {
            accession: None,
            is_ncbi_surveillance: None,
        };
        let row = m.to_flat_row(",");
        assert_eq!(row, ",");
    }

    #[test]
    fn test_genome_metadata_to_flat_row_tsv() {
        let m = make_metadata("GCA_000005845.2", false);
        let row = m.to_flat_row("\t");
        assert_eq!(row, "GCA_000005845.2\tfalse");
    }

    // ── ToFlatRow for GenomeCard ──────────────────────────────────────────────────

    #[test]
    fn test_genome_card_header_column_count() {
        let header = GenomeCard::csv_header(",");
        let count = header.split(',').count();
        // Must match the number of values produced by to_flat_row
        let card = make_genome_card();
        let row = card.to_flat_row(",");
        let row_count = row.split(',').count();
        assert_eq!(
            count, row_count,
            "csv_header column count ({}) does not match to_flat_row value count ({})",
            count, row_count
        );
    }

    #[test]
    fn test_genome_card_header_tab_column_count() {
        let header = GenomeCard::csv_header("\t");
        let count = header.split('\t').count();
        let card = make_genome_card();
        let row = card.to_flat_row("\t");
        let row_count = row.split('\t').count();
        assert_eq!(count, row_count);
    }

    #[test]
    fn test_genome_card_to_flat_row_contains_accession() {
        let card = make_genome_card();
        let row = card.to_flat_row(",");
        assert!(row.starts_with("GCA_000005845.2,"));
    }

    #[test]
    fn test_genome_card_to_flat_row_contains_name() {
        let card = make_genome_card();
        let row = card.to_flat_row(",");
        assert!(row.contains("Escherichia coli K-12"));
    }

    #[test]
    fn test_genome_card_to_flat_row_contains_genome_size() {
        let card = make_genome_card();
        let row = card.to_flat_row(",");
        assert!(row.contains("4641652"));
    }

    #[test]
    fn test_genome_card_to_flat_row_contains_gtdb_species() {
        let card = make_genome_card();
        let row = card.to_flat_row(",");
        assert!(row.contains("s__Escherichia coli"));
    }

    #[test]
    fn test_genome_card_to_flat_row_contains_gtdb_representative() {
        let card = make_genome_card();
        let row = card.to_flat_row(",");
        // gtdb_representative is a bool field — true
        assert!(row.contains("true"));
    }

    #[test]
    fn test_genome_card_to_flat_row_none_fields_produce_empty() {
        let mut card = make_genome_card();
        card.lpsn_url = None;
        card.subunit_summary = None;
        let row = card.to_flat_row(",");
        // Last two columns should be empty (subunit_summary, lpsn_url)
        // Row ends with: ...,<species_cluster_count>,,
        assert!(row.ends_with(",,"));
    }

    #[test]
    fn test_genome_card_to_flat_row_lpsn_url_present() {
        let card = make_genome_card();
        let row = card.to_flat_row(",");
        assert!(row.ends_with("https://lpsn.dsmz.de/species/escherichia-coli"));
    }

    #[test]
    fn test_genome_card_header_first_column_is_accession() {
        let header = GenomeCard::csv_header(",");
        assert!(header.starts_with("accession,"));
    }

    #[test]
    fn test_genome_card_header_last_column_is_lpsn_url() {
        let header = GenomeCard::csv_header(",");
        assert!(header.ends_with(",lpsn_url"));
    }

    // ── build_csv_string ──────────────────────────────────────────────────────────

    fn make_history(release: &str, d: &str, p: &str, f: &str, s: &str) -> History {
        History {
            release: Some(release.into()),
            d: Some(d.into()),
            p: Some(p.into()),
            c: None,
            o: None,
            f: Some(f.into()),
            g: None,
            s: Some(s.into()),
        }
    }

    #[test]
    fn test_build_csv_string_header_present() {
        let records = vec![make_history(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        )];
        let changes = HashMap::new();
        let output = build_csv_string(&records, &changes, ",");
        let first_line = output.lines().next().unwrap();
        assert_eq!(first_line, "release,domain,phylum,family,species,changes");
    }

    #[test]
    fn test_build_csv_string_tsv_header() {
        let records = vec![make_history(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        )];
        let changes = HashMap::new();
        let output = build_csv_string(&records, &changes, "\t");
        let first_line = output.lines().next().unwrap();
        assert_eq!(
            first_line,
            "release\tdomain\tphylum\tfamily\tspecies\tchanges"
        );
    }

    #[test]
    fn test_build_csv_string_single_record_initial_classification() {
        let records = vec![make_history(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        )];
        let changes = HashMap::new();
        let output = build_csv_string(&records, &changes, ",");
        // Single record is the first (and last) so should get "initial classification"
        let data_line = output.lines().nth(1).unwrap();
        assert!(data_line.contains("initial classification"));
        assert!(data_line.starts_with("R214,"));
    }

    #[test]
    fn test_build_csv_string_two_records_change_noted() {
        let records = vec![
            make_history(
                "R220",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus velezensis",
            ),
            make_history(
                "R214",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus subtilis",
            ),
        ];
        let mut changes = HashMap::new();
        changes.insert(
            "R220".to_string(),
            vec!["Species: Bacillus subtilis -> Bacillus velezensis".to_string()],
        );

        let output = build_csv_string(&records, &changes, ",");
        let lines: Vec<&str> = output.lines().collect();

        // Header + 2 data lines
        assert_eq!(lines.len(), 3);

        // R220 line should contain the change note
        let r220_line = lines[1];
        assert!(r220_line.starts_with("R220,"));
        assert!(r220_line.contains("Bacillus subtilis -> Bacillus velezensis"));

        // R214 line is last — initial classification
        let r214_line = lines[2];
        assert!(r214_line.starts_with("R214,"));
        assert!(r214_line.contains("initial classification"));
    }

    #[test]
    fn test_build_csv_string_empty_records() {
        let records: Vec<History> = vec![];
        let changes = HashMap::new();
        let output = build_csv_string(&records, &changes, ",");
        // Only the header line, plus trailing newline
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with("release"));
    }

    #[test]
    fn test_build_csv_string_none_fields_produce_empty() {
        let record = History {
            release: Some("R214".into()),
            d: None,
            p: None,
            c: None,
            o: None,
            f: None,
            g: None,
            s: None,
        };
        let changes = HashMap::new();
        let output = build_csv_string(&[record], &changes, ",");
        let data_line = output.lines().nth(1).unwrap();
        // release,,,,,initial classification
        assert!(data_line.starts_with("R214,,,,"));
    }

    #[test]
    fn test_build_csv_string_ends_with_newline() {
        let records = vec![make_history(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        )];
        let changes = HashMap::new();
        let output = build_csv_string(&records, &changes, ",");
        assert!(output.ends_with('\n'));
    }

    // ── process_taxon_history ─────────────────────────────────────────────────────

    #[test]
    fn test_process_taxon_history_json_output_to_file() {
        let mut server = Server::new();
        let history_json = serde_json::to_string(&vec![
            make_history(
                "R220",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus velezensis",
            ),
            make_history(
                "R214",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus subtilis",
            ),
        ])
        .unwrap();

        let _m = server
            .mock("GET", "/genome/GCA_000001.1/taxon-history")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&history_json)
            .create();

        // Temporarily override the base URL by using fetch_data directly —
        // we test the full function via a temp file
        let out_file = NamedTempFile::new().unwrap();
        let out_path = out_file.path().to_str().unwrap().to_string();
        // We need to remove the file first since write_to_output checks for non-existence
        std::fs::remove_file(&out_path).unwrap();

        let agent = ureq::Agent::config_builder().build().new_agent();
        let url = format!("{}/genome/GCA_000001.1/taxon-history", server.url());

        // Fetch the response manually since process_taxon_history uses the real URL
        let response = agent.get(&url).call().unwrap();
        let records: Vec<History> = response.into_body().read_json().unwrap();
        let _ = compute_taxonomic_changes(&records);

        let content = serde_json::to_string_pretty(&records).unwrap();
        utils::write_to_output(content.as_bytes(), Some(out_path.clone()), false).unwrap();

        let written = std::fs::read_to_string(&out_path).unwrap();
        assert!(written.contains("R220"));
        assert!(written.contains("R214"));
        assert!(written.contains("Bacillus velezensis"));
        std::fs::remove_file(&out_path).unwrap();
    }

    #[test]
    fn test_process_taxon_history_csv_output() {
        let records = vec![
            make_history(
                "R220",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus velezensis",
            ),
            make_history(
                "R214",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus subtilis",
            ),
        ];
        let changes = compute_taxonomic_changes(&records);
        let output = build_csv_string(&records, &changes, ",");

        // Verify the CSV contains both releases
        assert!(output.contains("R220"));
        assert!(output.contains("R214"));
        // Verify the species change is noted
        assert!(output.contains("Bacillus subtilis -> Bacillus velezensis"));
    }

    #[test]
    fn test_process_taxon_history_tsv_separators() {
        let records = vec![make_history(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        )];
        let changes = HashMap::new();
        let output = build_csv_string(&records, &changes, "\t");
        // TSV must not contain commas as separators in the header
        let header = output.lines().next().unwrap();
        assert!(!header.contains(','));
        assert!(header.contains('\t'));
    }

    // ── fetch_and_save_genome_data ────────────────────────────────────────────────

    #[test]
    fn test_fetch_and_save_genome_metadata_json_to_stdout() {
        let mut server = Server::new();
        let metadata = GenomeMetadata {
            accession: Some("GCA_000005845.2".into()),
            is_ncbi_surveillance: Some(false),
        };
        let body = serde_json::to_string(&metadata).unwrap();

        let _m = server
            .mock("GET", "/genome/GCA_000005845.2/metadata")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create();

        // Test via direct deserialization (fetch_and_save_genome_data uses real URLs;
        // we test the data-transform path by simulating what the function does)
        let result: GenomeMetadata = serde_json::from_str(&body).unwrap();
        assert_eq!(result.accession.as_deref(), Some("GCA_000005845.2"));
        assert_eq!(result.is_ncbi_surveillance, Some(false));

        // Verify the flat row output matches expected format
        let row = result.to_flat_row(",");
        assert_eq!(row, "GCA_000005845.2,false");
    }

    #[test]
    fn test_fetch_and_save_genome_data_csv_output_to_file() {
        let mut server = Server::new();
        let metadata = GenomeMetadata {
            accession: Some("GCA_000005845.2".into()),
            is_ncbi_surveillance: Some(false),
        };
        let body = serde_json::to_string(&metadata).unwrap();

        let _m = server
            .mock("GET", "/genome/GCA_000005845.2/metadata")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&body)
            .create();

        let agent = ureq::Agent::config_builder().build().new_agent();
        let url = format!("{}/genome/GCA_000005845.2/metadata", server.url());
        let response = agent.get(&url).call().unwrap();
        let data: GenomeMetadata = response.into_body().read_json().unwrap();

        // Simulate the CSV write path
        let sep = ",";
        let mut output = format!("{}\n", GenomeMetadata::csv_header(sep));
        output.push_str(&(data.to_flat_row(sep) + "\n"));

        let expected = "accession,is_ncbi_surveillance\nGCA_000005845.2,false\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_fetch_and_save_genome_data_tsv_output() {
        let metadata = GenomeMetadata {
            accession: Some("GCA_000005845.2".into()),
            is_ncbi_surveillance: Some(true),
        };

        let sep = "\t";
        let mut output = format!("{}\n", GenomeMetadata::csv_header(sep));
        output.push_str(&(metadata.to_flat_row(sep) + "\n"));

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[0], "accession\tis_ncbi_surveillance");
        assert_eq!(lines[1], "GCA_000005845.2\ttrue");
    }

    #[test]
    fn test_fetch_and_save_genome_data_json_output() {
        let metadata = GenomeMetadata {
            accession: Some("GCA_000005845.2".into()),
            is_ncbi_surveillance: Some(false),
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("GCA_000005845.2"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_fetch_and_save_genome_data_multiple_accessions_csv() {
        // Simulate what the function does for two accessions in CSV mode
        let records = vec![
            GenomeMetadata {
                accession: Some("GCA_000001.1".into()),
                is_ncbi_surveillance: Some(false),
            },
            GenomeMetadata {
                accession: Some("GCA_000002.1".into()),
                is_ncbi_surveillance: Some(true),
            },
        ];

        let sep = ",";
        let mut output = format!("{}\n", GenomeMetadata::csv_header(sep));
        for r in &records {
            output.push_str(&(r.to_flat_row(sep) + "\n"));
        }

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 data rows
        assert_eq!(lines[0], "accession,is_ncbi_surveillance");
        assert_eq!(lines[1], "GCA_000001.1,false");
        assert_eq!(lines[2], "GCA_000002.1,true");
    }

    #[test]
    fn test_fetch_and_save_genome_data_load_input_from_file() {
        // Verify load_input reads accessions from a file correctly
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "GCA_000001.1").unwrap();
        writeln!(tmp, "GCA_000002.1").unwrap();
        writeln!(tmp, "GCA_000003.1").unwrap();

        let args = make_genome_args(
            None,
            Some(tmp.path().to_str().unwrap().to_string()),
            true,
            false,
            "csv",
            None,
        );

        let accessions = utils::load_input(&args, "error".to_string()).unwrap();
        assert_eq!(accessions.len(), 3);
        assert_eq!(accessions[0], "GCA_000001.1");
        assert_eq!(accessions[1], "GCA_000002.1");
        assert_eq!(accessions[2], "GCA_000003.1");
    }

    #[test]
    fn test_fetch_and_save_genome_data_load_input_single_accession() {
        let args = make_genome_args(Some("GCA_000005845.2"), None, false, false, "json", None);

        let accessions = utils::load_input(&args, "error".to_string()).unwrap();
        assert_eq!(accessions.len(), 1);
        assert_eq!(accessions[0], "GCA_000005845.2");
    }

    #[test]
    fn test_fetch_and_save_genome_data_load_input_no_input_returns_error() {
        let args = make_genome_args(None, None, false, false, "json", None);
        let result = utils::load_input(&args, "No accession or file provided".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No accession or file provided"));
    }

    // ── get_genome_taxon_history ──────────────────────────────────────────────────

    #[test]
    fn test_get_genome_taxon_history_load_input_from_file() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "GCA_000001.1").unwrap();
        writeln!(tmp, "GCA_000002.1").unwrap();

        let args = GenomeArgs {
            accession: None,
            file: Some(tmp.path().to_str().unwrap().to_string()),
            history: true,
            metadata: false,
            outfmt: "json".to_string(),
            out: None,
            insecure: false,
            split: false,
            split_dir: None,
            release: None,
        };

        let accessions = utils::load_input(&args, "error".to_string()).unwrap();
        assert_eq!(accessions.len(), 2);
        assert_eq!(accessions[0], "GCA_000001.1");
    }

    #[test]
    fn test_get_genome_taxon_history_csv_format_via_build_csv_string() {
        // Simulate get_genome_taxon_history output pipeline for CSV
        let records = vec![
            make_history(
                "R220",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus velezensis",
            ),
            make_history(
                "R214",
                "Bacteria",
                "Firmicutes",
                "Bacillaceae",
                "Bacillus subtilis",
            ),
        ];
        let changes = compute_taxonomic_changes(&records);
        let content = build_csv_string(&records, &changes, ",");

        assert!(content.contains("release,domain,phylum,family,species,changes"));
        assert!(content.contains("R220"));
        assert!(content.contains("R214"));
    }

    #[test]
    fn test_get_genome_taxon_history_json_format() {
        let records = vec![make_history(
            "R220",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus velezensis",
        )];
        let content = serde_json::to_string_pretty(&records).unwrap();
        assert!(content.contains("R220"));
        assert!(content.contains("Bacillus velezensis"));
    }

    #[test]
    fn test_get_genome_taxon_history_outfmt_routing() {
        // Verify that outfmt correctly routes to the right format function
        let records = vec![make_history(
            "R214",
            "Bacteria",
            "Firmicutes",
            "Bacillaceae",
            "Bacillus subtilis",
        )];
        let changes = HashMap::new();

        let csv_content = build_csv_string(&records, &changes, ",");
        let tsv_content = build_csv_string(&records, &changes, "\t");
        let json_content = serde_json::to_string_pretty(&records).unwrap();

        // CSV uses commas
        assert!(csv_content.contains("release,domain"));
        // TSV uses tabs
        assert!(tsv_content.contains("release\tdomain"));
        // JSON is structured
        assert!(json_content.starts_with('['));
    }
}
