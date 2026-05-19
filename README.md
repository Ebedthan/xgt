# xgt

**Fast and flexible GTDB querying from the command line, built in Rust.**

[![CI](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/xgt/graph/badge.svg?token=OFAOB6K5KB)](https://codecov.io/gh/Ebedthan/xgt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat)](https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat)](https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE)

## What is xgt?

`xgt` is a command-line tool for querying the [Genome Taxonomy Database (GTDB)](https://gtdb.ecogenomic.org/) directly from your terminal or scripts. It covers the core GTDB REST API, genome cards, metadata, taxonomic history, taxon lineages, and search, and adds features designed for real research workflows: batch input from files or stdin, automatic pagination, retry logic, flexible output formats (JSON, CSV, TSV), and per-item file splitting for large datasets.

It is written in Rust for speed, portability, and a single self-contained binary with no runtime dependencies.


## Installation

### Prebuilt binaries

Download the binary for your platform from the [releases page](https://github.com/Ebedthan/xgt/releases) and place it somewhere on your `$PATH`.

| Platform | Download |
|---|---|
| Linux x86\_64 | `xgt-vX.X.X-x86_64-unknown-linux-gnu.tar.xz` |
| Linux aarch64 | `xgt-vX.X.X-aarch64-unknown-linux-gnu.tar.xz` |
| macOS Apple Silicon | `xgt-vX.X.X-aarch64-apple-darwin.tar.xz` |
| macOS Intel | `xgt-vX.X.X-x86_64-apple-darwin.tar.xz` |
| Windows x86\_64 | `xgt-vX.X.X-x86_64-pc-windows-msvc.zip` |

SHA-256 checksums are provided alongside each archive.

### From source

Requires Rust 1.85 or later ([install via rustup](https://rustup.rs/)).

```bash
git clone https://github.com/Ebedthan/xgt.git
cd xgt
cargo build --release
# binary is at target/release/xgt
cp target/release/xgt ~/.local/bin/
```


## Quick start

```bash
# Search all genomes assigned to a genus
xgt search g__Escherichia

# Get the full card for a genome
xgt genome GCA_000005845.2

# Get the taxonomic classification of a taxon
xgt taxon g__Escherichia

# Compare a genome's taxonomy between two GTDB releases
xgt diff GCA_000005845.2 --from R214 --to R220
```

## Subcommands

### `search`: search genomes in GTDB

```
xgt search [OPTIONS] [QUERY]
```

Searches the GTDB for genomes matching a query string. The query can be a
taxon name, accession, organism name, or any field depending on `--field`.

**Options**

| Flag | Short | Description |
|---|---|---|
| `--field STR` | `-F` | Search field: `all` (default), `acc`, `org`, `gtdb`, `ncbi` |
| `--word` | `-w` | Match whole words only |
| `--rep` | `-r` | Restrict to GTDB species representatives |
| `--type` | `-t` | Restrict to NCBI type material |
| `--id` | `-i` | Print only genome accessions |
| `--count` | `-c` | Print only the count of matched genomes |
| `--file FILE` | `-f` | Read queries from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `csv` (default), `tsv`, `json` |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Search for a genus, output as JSON
xgt search g__Escherichia -O json

# Search from a file, output TSV to a file
xgt search -f genera.txt -O tsv -o results.tsv

# Read from stdin, count results per query
xgt search -f - --count

# Restrict to species representatives, print accessions only
xgt search g__Bacillus --rep --id

# Pipe to downstream tools
xgt search g__Salmonella --id | wc -l
```

The search automatically paginates through all results regardless of
dataset size. Queries on large genera like *Escherichia* return
complete results without truncation.

### `genome`: retrieve genome information

```
xgt genome [OPTIONS] [ACCESSION]
```

Fetches data for one or more genome accessions. By default returns the
full genome card (taxonomy, assembly statistics, CheckM quality, NCBI
metadata). Use `--metadata` for a lightweight metadata-only response,
or `--history` for the full taxonomic history across all GTDB releases.

**Options**

| Flag | Short | Description |
|---|---|---|
| `--metadata` | `-m` | Retrieve genome metadata instead of full card |
| `--history` | `-H` | Retrieve taxonomic history across all releases |
| `--file FILE` | `-f` | Read accessions from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `json` (default), `csv`, `tsv` |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Full genome card
xgt genome GCA_000005845.2

# Lightweight metadata in CSV
xgt genome GCA_000005845.2 --metadata -O csv

# Taxonomic history across all GTDB releases
xgt genome GCA_000005845.2 --history

# Batch: process 200 accessions from a file, write CSV
xgt genome -f accessions.txt -O csv -o results.csv

# Batch: read accessions from stdin
cat accessions.txt | xgt genome -f -

# Batch: write one JSON file per accession
xgt genome -f accessions.txt --split --split-dir genome_cards/
```

When processing a file of accessions, a progress bar is shown on stderr
so that stdout output can be safely piped to downstream tools.


### `taxon`: explore GTDB taxonomy

```
xgt taxon [OPTIONS] [NAME]
```

Retrieves information about a GTDB taxon. The taxon name must use the
standard rank prefix format (e.g. `g__Escherichia`, `s__Escherichia coli`).
Valid prefixes are `d__`, `p__`, `c__`, `o__`, `f__`, `g__`, `s__`.

**Options**

| Flag | Short | Description |
|---|---|---|
| `--search` | `-s` | Search for a taxon name in the current release |
| `--all` | | Search for a taxon name across all releases |
| `--genomes` | `-g` | List genomes assigned to the taxon |
| `--reps` | `-r` | With `--genomes`, return species representatives only |
| `--word` | `-w` | Match whole words only |
| `--file FILE` | `-f` | Read taxon names from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `json` (default), `csv`, `tsv` |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Full taxonomic lineage for a genus
xgt taxon g__Escherichia

# Search for a taxon name in the current release
xgt taxon --search g__Escherichia

# Search across all GTDB releases
xgt taxon --all g__Escherichia

# List all genomes in a taxon
xgt taxon g__Escherichia --genomes

# List only species representatives
xgt taxon g__Escherichia --genomes --reps

# Output genome list as CSV
xgt taxon g__Escherichia --genomes -O csv -o genomes.csv
```

### `diff`: compare taxonomy between releases

```
xgt diff [OPTIONS] [ACCESSION] --from RELEASE
```

Shows how the taxonomic classification of a genome changed between two
GTDB releases. Requires `--from`; if `--to` is omitted, the latest
available release for the genome is used. Release identifiers use the
format `R<number>` (e.g. `R214`, `R220`, `R226`).

**Options**

| Flag | Short | Description |
|---|---|---|
| `--from RELEASE` | | Source release to compare from (required) |
| `--to RELEASE` | | Target release to compare to (default: latest) |
| `--file FILE` | `-f` | Read accessions from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `json` (default), `csv`, `tsv` |
| `--split` | `-s` | Write one file per accession |
| `--split-dir DIR` | | Directory for per-item files (requires `--split`) |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Compare a single genome between two releases
xgt diff GCA_000005845.2 --from R214 --to R220

# Compare against the latest release
xgt diff GCA_000005845.2 --from R207

# Batch comparison, CSV output
xgt diff -f accessions.txt --from R214 --to R220 -O csv -o changes.csv

# One diff file per accession
xgt diff -f accessions.txt --from R214 --to R220 --split --split-dir diffs/
```

**Output (JSON)**

```json
{
  "query": "GCA_000005845.2",
  "from_release": "R214",
  "to_release": "R220",
  "changed": true,
  "changes": [
    {
      "rank": "species",
      "from": "Escherichia coli",
      "to": "Escherichia coli_D"
    }
  ],
  "from_taxonomy": {
    "release": "R214",
    "domain": "d__Bacteria",
    "phylum": "p__Pseudomonadota",
    "class": "c__Gammaproteobacteria",
    "order": "o__Enterobacterales",
    "family": "f__Enterobacteriaceae",
    "genus": "g__Escherichia",
    "species": "s__Escherichia coli"
  },
  "to_taxonomy": { "..." : "..." }
}
```

## Common patterns

### Batch processing with stdin

```bash
# Generate an accession list from another tool and pipe directly
grep "Escherichia" my_study_metadata.tsv | cut -f1 | xgt genome -f - -O csv
```

### Combining subcommands

```bash
# Get all accessions in a genus, then fetch their cards
xgt search g__Rhizobium --id > accessions.txt
xgt genome -f accessions.txt -O csv -o rhizobium_cards.csv
```

### Tracking taxonomic changes in a dataset

```bash
# Find which genomes in your study changed between R214 and R220
xgt diff -f study_accessions.txt --from R214 --to R220 \
  -O csv -o taxonomy_changes.csv

# Filter to only changed genomes
awk -F',' '$4 == "true"' taxonomy_changes.csv
```

### Output to separate files per accession

```bash
# One JSON file per genome in a results directory
xgt genome -f accessions.txt --split --split-dir results/
```

## Output formats

All subcommands support `--outfmt csv|tsv|json` (except `search`, which
defaults to `csv`; all others default to `json`).

CSV and TSV output is suitable for loading directly into R, Python, or
spreadsheet tools. JSON output preserves all nested fields and is
recommended when the full genome card is needed.

All output goes to stdout by default. Use `--out FILE` to write to a
file, or `--split` to write one file per query when processing batches.


## Reporting issues

Found a bug or want to request a feature? [Open an issue](https://github.com/Ebedthan/xgt/issues).

Please include:
- OS and architecture
- xgt version (`xgt --version`)
- The command you ran and the output or error you received

## License

Dual-licensed under the [MIT License](LICENSE-MIT) and the
[Apache 2.0 License](LICENSE-APACHE). You may use either at your option.


## Developer notes

- Minimum Rust version: **1.85**
- Follows [Semantic Versioning](https://semver.org/)
- Contributions welcome! Please open an issue before submitting large changes
