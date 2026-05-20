use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
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

    /// Show taxonomic differences between two GTDB releases
    Diff(DiffArgs),

    /// Generate shell completion scripts
    Completions(CompletionsArgs),
}

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
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

    /// Read queries from FILE, one per line. Use '-' to read from stdin.
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Output to FILE (format set by --outfmt)
    #[arg(short, long, value_name = "FILE", value_parser = is_existing)]
    pub out: Option<String>,

    /// Output format
    #[arg(short = 'O', long, value_name = "STR", default_value = "csv", value_parser = ["csv", "json", "tsv"])]
    pub outfmt: String,

    /// Write each result to a separate file named after the query/accession.
    /// Mutually exclusive with --out.
    #[arg(short = 's', long, conflicts_with = "out")]
    pub split: bool,

    /// Directory for per-item files when using --split (default: current directory)
    #[arg(long, value_name = "DIR", requires = "split")]
    pub split_dir: Option<String>,

    /// Target a specific GTDB release (e.g. R214, R220, R226).
    /// Defaults to the latest release. Note: not all endpoints support
    /// historical releases; unsupported endpoints will use the latest.
    #[arg(long, value_name = "RELEASE", value_parser = parse_release)]
    pub release: Option<String>,

    /// Disable SSL certificate verification
    #[arg(short = 'k')]
    pub insecure: bool,
}

#[derive(Args)]
pub struct GenomeArgs {
    /// Genome accession
    #[arg(conflicts_with = "file")]
    pub accession: Option<String>,

    /// Read accessions from FILE, one per line. Use '-' to read from stdin.
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Get genome taxon history
    #[arg(short = 'H', long)]
    pub history: bool,

    /// Get genome metadata
    #[arg(short, long, conflicts_with = "history")]
    pub metadata: bool,

    /// Output format
    #[arg(short = 'O', long, value_name = "STR", default_value = "json", value_parser = ["csv", "json", "tsv"])]
    pub outfmt: String,

    /// Output raw JSON
    #[arg(short, long, value_name = "FILE", value_parser = is_existing)]
    pub out: Option<String>,

    /// Write each result to a separate file named after the query/accession.
    /// Mutually exclusive with --out.
    #[arg(short = 's', long, conflicts_with = "out")]
    pub split: bool,

    /// Directory for per-item files when using --split (default: current directory)
    #[arg(long, value_name = "DIR", requires = "split")]
    pub split_dir: Option<String>,

    /// Target a specific GTDB release (e.g. R214, R220, R226).
    /// Defaults to the latest release. Note: not all endpoints support
    /// historical releases; unsupported endpoints will use the latest.
    #[arg(long, value_name = "RELEASE", value_parser = parse_release)]
    pub release: Option<String>,

    /// Disable SSL certificate verification
    #[arg(short = 'k')]
    pub insecure: bool,
}

#[derive(Args)]
pub struct TaxonArgs {
    /// Taxon name
    #[arg(value_parser = is_valid_taxon, conflicts_with = "file")]
    pub name: Option<String>,

    /// Read taxon names from FILE, one per line. Use '-' to read from stdin.
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Output to FILE (format set by --outfmt)
    #[arg(short, long, value_name = "FILE", value_parser = is_existing)]
    pub out: Option<String>,

    /// Match only whole words
    #[arg(short, long)]
    pub word: bool,

    /// Search for a taxon in current release
    #[arg(long)]
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

    /// Output format
    #[arg(short = 'O', long, value_name = "STR", default_value = "json", value_parser = ["csv", "json", "tsv"])]
    pub outfmt: String,

    /// Write each result to a separate file named after the query/accession.
    /// Mutually exclusive with --out.
    #[arg(short = 's', long, conflicts_with = "out")]
    pub split: bool,

    /// Directory for per-item files when using --split (default: current directory)
    #[arg(long, value_name = "DIR", requires = "split")]
    pub split_dir: Option<String>,

    /// Target a specific GTDB release (e.g. R214, R220, R226).
    /// Defaults to the latest release. Note: not all endpoints support
    /// historical releases; unsupported endpoints will use the latest.
    #[arg(long, value_name = "RELEASE", value_parser = parse_release)]
    pub release: Option<String>,

    /// Disable SSL certificate verification
    #[arg(short = 'k')]
    pub insecure: bool,
}

#[derive(Args)]
pub struct DiffArgs {
    /// Genome accession or taxon name to compare
    #[arg(conflicts_with = "file")]
    pub query: Option<String>,

    /// Read queries from FILE, one per line. Use '-' to read from stdin.
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<String>,

    /// Source release to compare from (e.g. R214)
    #[arg(long, value_name = "RELEASE", value_parser = parse_release)]
    pub from: String,

    /// Target release to compare to (e.g. R220). Defaults to latest.
    #[arg(long, value_name = "RELEASE", value_parser = parse_release)]
    pub to: Option<String>,

    /// Output format
    #[arg(short = 'O', long, value_name = "STR", default_value = "json",
          value_parser = ["csv", "tsv", "json"])]
    pub outfmt: String,

    /// Output to FILE (format set by --outfmt)
    #[arg(short, long, value_name = "FILE", value_parser = is_existing)]
    pub out: Option<String>,

    /// Write each result to a separate file named after the query
    #[arg(short = 's', long, conflicts_with = "out")]
    pub split: bool,

    /// Directory for per-item files when using --split
    #[arg(long, value_name = "DIR", requires = "split")]
    pub split_dir: Option<String>,

    /// Disable SSL certificate verification
    #[arg(short = 'k')]
    pub insecure: bool,
}

fn parse_release(s: &str) -> Result<String, String> {
    // Accept R<number> or r<number>, normalize to uppercase
    let upper = s.to_uppercase();
    if upper.starts_with('R') && upper[1..].parse::<u32>().is_ok() {
        Ok(upper)
    } else {
        Err(format!(
            "'{}' is not a valid release identifier. \
             Use the format R<number>, e.g. R214, R220, R226.",
            s
        ))
    }
}

fn is_valid_taxon(s: &str) -> Result<String, String> {
    let prefixes = ["d__", "p__", "c__", "o__", "f__", "g__", "s__"];
    for prefix in &prefixes {
        if s.starts_with(prefix) {
            return Ok(s.to_string());
        }
    }
    Err(format!(
        "'{}' is not a valid taxon name. Use a rank prefix followed by the name \
         (e.g. g__Escherichia, s__Escherichia coli). \
         Valid prefixes: d__, p__, c__, o__, f__, g__, s__",
        s
    ))
}

fn is_existing(s: &str) -> Result<String, String> {
    if !Path::new(s).exists() {
        Ok(s.to_string())
    } else {
        Err(format!(
            "Output file '{}' already exists. Choose a different path or remove the existing file.",
            s
        ))
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
        assert_eq!(result.err().unwrap(), "Output file 'test/acc.txt' already exists. Choose a different path or remove the existing file.");

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
            Err("'Bacteria' is not a valid taxon name. Use a rank prefix followed by the name (e.g. g__Escherichia, s__Escherichia coli). Valid prefixes: d__, p__, c__, o__, f__, g__, s__".to_string())
        );
        assert_eq!(
            is_valid_taxon("d_"),
            Err("'d_' is not a valid taxon name. Use a rank prefix followed by the name (e.g. g__Escherichia, s__Escherichia coli). Valid prefixes: d__, p__, c__, o__, f__, g__, s__".to_string())
        );
        assert_eq!(
            is_valid_taxon("Actinobacteria"),
            Err("'Actinobacteria' is not a valid taxon name. Use a rank prefix followed by the name (e.g. g__Escherichia, s__Escherichia coli). Valid prefixes: d__, p__, c__, o__, f__, g__, s__".to_string())
        );
        assert_eq!(
            is_valid_taxon("__Actinobacteria"),
            Err("'__Actinobacteria' is not a valid taxon name. Use a rank prefix followed by the name (e.g. g__Escherichia, s__Escherichia coli). Valid prefixes: d__, p__, c__, o__, f__, g__, s__".to_string())
        );
        assert_eq!(
            is_valid_taxon("d_Actinobacteria"),
            Err("'d_Actinobacteria' is not a valid taxon name. Use a rank prefix followed by the name (e.g. g__Escherichia, s__Escherichia coli). Valid prefixes: d__, p__, c__, o__, f__, g__, s__".to_string())
        );
    }

    #[test]
    fn test_parse_release_valid() {
        // tested via cli validator
        assert!(parse_release("R214").is_ok());
        assert!(parse_release("r220").is_ok()); // lowercase accepted, normalized
        assert_eq!(parse_release("r220").unwrap(), "R220");
    }

    #[test]
    fn test_parse_release_invalid() {
        assert!(parse_release("214").is_err());
        assert!(parse_release("release214").is_err());
        assert!(parse_release("R").is_err());
        assert!(parse_release("Rabc").is_err());
    }
}
