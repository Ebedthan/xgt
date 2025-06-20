use clap::{Args, Parser, Subcommand};
use std::path::Path;

#[derive(Parser)]
#[command(name = "xgt")]
#[command(about = "Query and parse GTDB data", version)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Add verbosity to program
    #[arg(short = 'v', long, action = clap::ArgAction::SetTrue)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search a taxon on GTDB
    Search(SearchArgs),

    /// Information about a genome
    Genome(GenomeArgs),

    /// Information about a specific taxon
    Taxon(TaxonArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    /// A value (typically a species or genus name/taxon) used for searching
    #[arg(conflicts_with = "file")]
    pub query: Option<String>,

    /// Search field
    #[arg(short = 'F', long, value_name = "STR", default_value = "all", value_parser = ["all", "acc", "org", "gtdb", "ncbi"])]
    pub field: String,

    /// Match only whole words
    #[arg(short, long)]
    pub word: bool,

    /// Search GTDB representative species only
    #[arg(short, long)]
    pub rep: bool,

    /// Search NCBI type species only
    #[arg(short, long)]
    pub r#type: bool,

    /// Only print matched genomes ID
    #[arg(short, long)]
    pub id: bool,

    /// Only print a count of matched genomes
    #[arg(short, long)]
    pub count: bool,

    /// Only print a count of matched genomes
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Output to FILE
    #[arg(short, long, value_name = "FILE", value_parser = is_existing)]
    pub out: Option<String>,

    /// Output format
    #[arg(short = 'O', long, value_name = "STR", default_value = "csv", value_parser = ["csv", "json", "tsv"])]
    pub outfmt: String,

    /// Disable SSL certificate verification
    #[arg(short = 'k')]
    pub insecure: bool,
}

#[derive(Args)]
pub struct GenomeArgs {
    /// Genome accession
    #[arg(conflicts_with = "file")]
    pub accession: Option<String>,

    /// Search from name in FILE
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Get genome taxon history
    #[arg(short = 'H', long)]
    pub history: bool,

    /// Get genome metadata
    #[arg(short, long, conflicts_with = "history")]
    pub metadata: bool,

    /// Output raw JSON
    #[arg(short, long, value_name = "FILE", value_parser = is_existing)]
    pub out: Option<String>,

    /// Disable SSL certificate verification
    #[arg(short = 'k')]
    pub insecure: bool,
}

#[derive(Args)]
pub struct TaxonArgs {
    /// Taxon name
    #[arg(value_parser = is_valid_taxon, conflicts_with = "file")]
    pub name: Option<String>,

    /// Search from name in FILE
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Redirect output to FILE
    #[arg(short, long, value_name = "FILE", value_parser = is_existing)]
    pub out: Option<String>,

    /// Match only whole words
    #[arg(short, long)]
    pub word: bool,

    /// Search for a taxon in current release
    #[arg(short, long)]
    pub search: bool,

    /// Search for a taxon across all releases
    #[arg(long)]
    pub all: bool,

    /// Get V taxon genomes
    #[arg(short, long)]
    pub genomes: bool,

    /// Set taxon V genomes search to lookup reps seqs only
    #[arg(short, long)]
    pub reps: bool,

    /// Disable SSL certificate verification
    #[arg(short = 'k')]
    pub insecure: bool,
}

fn is_valid_taxon(s: &str) -> Result<String, String> {
    let prefixes = ["d__", "p__", "c__", "o__", "f__", "g__", "s__"];
    for prefix in &prefixes {
        if s.starts_with(prefix) {
            return Ok(s.to_string());
        }
    }
    Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
}

fn is_existing(s: &str) -> Result<String, String> {
    if !Path::new(s).exists() {
        Ok(s.to_string())
    } else {
        Err("file should not already exists".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_existing() {
        // Test with a non-existing file
        let result = is_existing("test/acc.txt");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "file should not already exists");

        // Test with an existing file
        let result = is_existing("non_existing_file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "non_existing_file.txt".to_string());
    }

    #[test]
    fn test_is_valid_taxon() {
        // Positive test cases
        assert_eq!(is_valid_taxon("d__Bacteria"), Ok("d__Bacteria".to_string()));
        assert_eq!(
            is_valid_taxon("g__Actinobacteria"),
            Ok("g__Actinobacteria".to_string())
        );
        assert_eq!(
            is_valid_taxon("s__Staphylococcus aureus"),
            Ok("s__Staphylococcus aureus".to_string())
        );

        // Negative test cases
        assert_eq!(
            is_valid_taxon("Bacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("d_"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("Actinobacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("__Actinobacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
        assert_eq!(
            is_valid_taxon("d_Actinobacteria"),
            Err("Taxon name must be in greengenes format, e.g. g__Foo".to_string())
        );
    }
}
