# xgt

**Fast and flexible GTDB querying from the command line, built in Rust.**

[![CI](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/xgt/graph/badge.svg?token=OFAOB6K5KB)](https://codecov.io/gh/Ebedthan/xgt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat)](https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat)](https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE)

## What is xgt?

`xgt` is a command-line tool for querying the [Genome Taxonomy Database (GTDB)](https://gtdb.ecogenomic.org/) directly from your terminal or scripts. It covers the core GTDB REST API, genome cards, metadata, taxonomic history, taxon lineages, and search, and adds features designed for real research workflows: batch input from files or stdin, automatic parallel pagination, retry logic, flexible output formats (JSON, CSV, TSV), per-item file splitting, a transparent local response cache, and cross-release taxonomic comparison.

It is written in Rust for speed, portability, and a single self-contained binary with no runtime dependencies beyond SQLite (bundled).


## Installation

### Prebuilt binaries

Download the binary for your platform from the [releases page](https://github.com/Ebedthan/xgt/releases) and place it somewhere on your `$PATH`.

| Platform | Download |
|---|---|
| Linux x86\_64 | `xgt-vX.X.X-x86_64-unknown-linux-gnu.tar.xz` |
| macOS Apple Silicon | `xgt-vX.X.X-aarch64-apple-darwin.tar.xz` |
| Windows x86\_64 | `xgt-vX.X.X-x86_64-pc-windows-msvc.zip` |

SHA-256 checksums are provided alongside each archive.

**macOS Intel / Linux aarch64:** build from source (see below), or run
the Apple Silicon binary under Rosetta 2 (installed by default on all
Apple Silicon Macs).

### From source

Requires Rust 1.85.1 or later ([install via rustup](https://rustup.rs/)).

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

# Search at species level (names with spaces are handled automatically)
xgt search 's__Escherichia coli' -F org --type --id

# Get the full card for a genome
xgt genome GCA_000005845.2

# Get the taxonomic classification of a taxon
xgt taxon g__Escherichia

# Compare a genome's taxonomy between two GTDB releases
xgt diff GCA_000005845.2 --from R214 --to R220

# Run a batch diff and cache results, repeat runs are instant
xgt diff -f accessions.txt --from R214 --to R232 -O csv -o changes.csv

# Check cache statistics
xgt --cache-info

# Check if a newer version of xgt is available
xgt --check-update
```

## Local cache

xgt includes a transparent response cache backed by SQLite. The cache
is enabled by default and stores API responses locally so repeated
queries return immediately without hitting the GTDB API.

**Cache location:**
- Linux / macOS: `~/.cache/xgt/cache.db`
- Windows: `%LOCALAPPDATA%\xgt\cache.db`

**Cache TTLs by endpoint:**

| Endpoint | TTL | Rationale |
|---|---|---|
| `diff` | 1 year | Results for fixed release pairs never change |
| `genome --history` | 90 days | History only grows, never changes |
| `genome card / metadata` | 30 days | Changes only on new GTDB releases |
| `taxon` | 30 days | Moderately stable between releases |
| `search` | 7 days | Changes as new genomes are added |

**Cache flags (global, work with any subcommand):**

| Flag | Description |
|---|---|
| `--no-cache` | Bypass the cache for this request, always fetch from the API |
| `--cache-info` | Print cache statistics (entry count, expired entries, size) and exit |
| `--clear-cache` | Delete all cached responses and exit |

```bash
# Force a fresh fetch even though the cache has a valid entry
xgt genome GCA_000005845.2 --no-cache

# See how much space the cache is using
xgt --cache-info

# Clear all cached data
xgt --clear-cache
```

The cache never returns data for requests that have not been made before.
On first run all requests go to the live API and are stored. On subsequent
runs the cache is checked first; expired entries are evicted automatically.
Pass `--no-cache` any time you need guaranteed fresh data.


## Subcommands

### `search`: search genomes in GTDB

```
xgt search [OPTIONS] [QUERY]
```

Searches GTDB for genomes matching a query string against one or more
metadata fields. Pagination is automatic and parallel, queries on large
genera like *Escherichia* (52,000+ genomes) return complete results
without truncation. Species-level queries with spaces in the name
(e.g. `'s__Escherichia coli'`) are handled correctly via automatic
URL encoding.

**Options**

| Flag | Short | Description |
|---|---|---|
| `--field STR` | `-F` | Search field: `all` (default), `acc`, `org`, `gtdb`, `ncbi` |
| `--word` | `-w` | Match whole words only |
| `--rep` | `-r` | Restrict to GTDB species representatives |
| `--type` | `-t` | Restrict to NCBI type material |
| `--id` | `-i` | Print only genome accessions, one per line |
| `--count` | `-c` | Print only the total count of matched genomes |
| `--max-concurrent N` | | Max concurrent page requests during pagination (default: 5) |
| `--file FILE` | `-f` | Read queries from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `csv` (default), `tsv`, `json` |
| `--split` | `-s` | Write one file per query |
| `--split-dir DIR` | | Directory for per-query files (requires `--split`) |
| `--no-cache` | | Bypass cache for this request |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Search for a genus, output as JSON
xgt search g__Escherichia -O json

# Search at species level, spaces in names are handled automatically
xgt search 's__Escherichia coli' -F org

# Search from a file, output TSV to a file
xgt search -f genera.txt -O tsv -o results.tsv

# Restrict to species representatives, print accessions only
xgt search g__Bacillus --rep --id

# Pipe accessions directly to xgt genome
xgt search g__Rhizobium --id | xgt genome -f - -O csv -o rhizobium.csv
```

### `genome`: retrieve genome information

```
xgt genome [OPTIONS] [ACCESSION]
```

Fetches data for one or more genome accessions. By default returns the
full genome card (taxonomy, assembly statistics, CheckM/CheckM2 quality,
NCBI metadata). Use `--metadata` for a lightweight response or
`--history` for the full taxonomic history across all GTDB releases.

**Options**

| Flag | Short | Description |
|---|---|---|
| `--metadata` | `-m` | Retrieve genome metadata instead of full card |
| `--history` | `-H` | Retrieve taxonomic history across all releases |
| `--release RELEASE` | | Target a specific GTDB release (e.g. `R214`) |
| `--file FILE` | `-f` | Read accessions from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `json` (default), `csv`, `tsv` |
| `--split` | `-s` | Write one file per accession |
| `--split-dir DIR` | | Directory for per-accession files (requires `--split`) |
| `--no-cache` | | Bypass cache for this request |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Full genome card
xgt genome GCA_000005845.2

# Taxonomic history across all GTDB releases
xgt genome GCA_000005845.2 --history

# Batch: process accessions from a file, write CSV
xgt genome -f accessions.txt -O csv -o results.csv

# Batch: write one JSON file per accession
xgt genome -f accessions.txt --split --split-dir genome_cards/

# Target a specific GTDB release
xgt genome GCA_000005845.2 --release R214

# Force a fresh fetch, bypassing the cache
xgt genome GCA_000005845.2 --no-cache
```


### `taxon`: explore GTDB taxonomy

```
xgt taxon [OPTIONS] [NAME]
```

Retrieves information about a GTDB taxon. Names must use the standard
rank prefix format (e.g. `g__Escherichia`, `s__Escherichia coli`).
Valid prefixes: `d__`, `p__`, `c__`, `o__`, `f__`, `g__`, `s__`.

**Options**

| Flag | Short | Description |
|---|---|---|
| `--search` | `-s` | Search for a taxon name in the current release, returning partial matches |
| `--all` | | Search across all GTDB releases and NCBI (requires `--search`) |
| `--genomes` | `-g` | List genome accessions assigned to the taxon |
| `--reps` | `-r` | With `--genomes`, return species representatives only |
| `--word` | `-w` | Restrict `--search` results to exact matches only |
| `--file FILE` | `-f` | Read taxon names from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `json` (default), `csv`, `tsv` |
| `--no-cache` | | Bypass cache for this request |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Full taxonomic record for a genus
xgt taxon g__Escherichia

# Species-level query with spaces in the name
xgt taxon 's__Escherichia coli'

# Search for a taxon name in the current release
xgt taxon --search Escherichia

# Search across all GTDB releases and NCBI
xgt taxon --search --all Escherichia

# List all genomes in a taxon
xgt taxon g__Escherichia --genomes

# List only species representatives
xgt taxon g__Escherichia --genomes --reps
```


### `diff`: compare taxonomy between releases

```
xgt diff [OPTIONS] [ACCESSION] --from RELEASE
```

Computes per-rank taxonomic changes for a genome between two GTDB
releases. Requires `--from`; if `--to` is omitted, the latest available
release for the genome is used automatically. Release identifiers follow
the format `R<number>` (e.g. `R214`, `R220`, `R232`).

Results for a given accession and release pair are cached for one year
by default, so large batch diff runs are fast on subsequent executions.

**Options**

| Flag | Short | Description |
|---|---|---|
| `--from RELEASE` | | Source release (required) |
| `--to RELEASE` | | Target release (default: latest available) |
| `--file FILE` | `-f` | Read accessions from FILE, one per line; use `-` for stdin |
| `--out FILE` | `-o` | Write output to FILE instead of stdout |
| `--outfmt STR` | `-O` | Output format: `json` (default), `csv`, `tsv` |
| `--split` | `-s` | Write one file per accession |
| `--split-dir DIR` | | Directory for per-accession files (requires `--split`) |
| `--no-cache` | | Bypass cache for this request |
| `--insecure` | `-k` | Disable SSL certificate verification |

**Examples**

```bash
# Compare a genome between two releases
xgt diff GCA_000005845.2 --from R214 --to R220

# Compare against the latest release
xgt diff GCA_000005845.2 --from R214

# Batch comparison, CSV output, cached on subsequent runs
xgt diff -f accessions.txt --from R214 --to R232 -O csv -o changes.csv
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
      "from": "s__Escherichia coli",
      "to": "s__G047199095 sp047199095"
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
  "to_taxonomy": { "...": "..." }
}
```

## Common patterns

### Tracking taxonomic changes in a dataset

```bash
# Find which genomes changed between two GTDB releases
# Results are cached, the second run returns instantly
xgt diff -f study_accessions.txt --from R214 --to R220 \
  -O csv -o taxonomy_changes.csv

# Filter to only changed genomes
awk -F',' '$4 == "true"' taxonomy_changes.csv
```

### Pipeline composition

```bash
# Get all representative accessions for a genus, then fetch genome cards
xgt search g__Rhizobium --rep --id | xgt genome -f - -O csv -o rhizobium.csv
```

### Per-accession output files

```bash
# One JSON file per genome
xgt genome -f accessions.txt --split --split-dir results/

# One diff file per accession
xgt diff -f accessions.txt --from R214 --split --split-dir diffs/
```

### Cache management

```bash
# Check what is cached and how much space it uses
xgt --cache-info

# Clear the cache entirely (e.g. after a new GTDB release)
xgt --clear-cache

# Force fresh data for a single query without clearing the whole cache
xgt genome GCA_000005845.2 --no-cache
```

## Output formats

All subcommands support `--outfmt csv|tsv|json`. The default is `csv`
for `search` and `json` for `genome`, `taxon`, and `diff`.

CSV and TSV output is suitable for loading directly into R, Python, or
spreadsheet tools. JSON preserves all nested fields and is recommended
when the full genome card is needed.

Output goes to stdout by default. Use `--out` to redirect to a single
file, or `--split` to write one file per query or accession.


## Shell completions

```bash
# Bash
xgt completions bash > ~/.local/share/bash-completion/completions/xgt

# Zsh (ensure ~/.zfunc is in your fpath)
xgt completions zsh > ~/.zfunc/_xgt

# Fish
xgt completions fish > ~/.config/fish/completions/xgt.fish
```

## Citation

If you use xgt in your research, please cite:

Ebou AET, Koua DK, Zézé A. xgt: a command-line interface for the Genome
Taxonomy Database with cross-release taxonomic comparison. *GigaScience*.
2026. doi:[10.1093/gigascience/giag086](https://doi.org/10.1093/gigascience/giag086)


### BibTeX

```bibtex

@article{ebouXgtCommandlineInterface2026,
  title = {Xgt: A Command-Line Interface for the {{Genome Taxonomy Database}} with Cross-Release Taxonomic Comparison},
  shorttitle = {Xgt},
  author = {Ebou, Anicet E T and Koua, Dominique K and Z{\'e}z{\'e}, Adolphe},
  year = 2026,
  month = aug,
  journal = {GigaScience},
  pages = {giag086},
  issn = {2047-217X},
  doi = {10.1093/gigascience/giag086},
}
```

## Reporting issues

Found a bug or want to request a feature?
[Open an issue](https://github.com/Ebedthan/xgt/issues).

Please include:
- OS and architecture
- xgt version (`xgt --version`)
- The command you ran and the output or error you received

## License

Dual-licensed under the [MIT License](LICENSE-MIT) and the
[Apache 2.0 License](LICENSE-APACHE). You may use either at your option.


## Developer notes

- Minimum Rust version: **1.85.1**
- Follows [Semantic Versioning](https://semver.org/)
- Run unit tests: `cargo test`
- Run integration tests (requires network):
  `cargo test --features integration-tests --test integration`
- Contributions welcome, please open an issue before submitting large changes
