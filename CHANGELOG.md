# Changelog

All notable changes to xgt are documented in this file. 
Versions follow [Semantic Versioning](https://semver.org/).

## [v1.0.0] - 2026-06-08

First stable release. Three years of public use and iterative
development culminating in a complete rewrite of the HTTP layer,
a significantly expanded feature set, and a commitment to semantic
versioning going forward.

### Added

**New subcommands**
- `diff`: compute per-rank taxonomic changes for a genome between
  any two GTDB releases. Requires `--from RELEASE`; `--to` defaults
  to the most recent available release for the queried accession.
  Outputs a `changed` flag, per-rank change records, and full
  taxonomy snapshots at both releases. Supports JSON, CSV, and TSV
  output and all batch input modes.
- `completions`: generate shell completion scripts for Bash, Zsh,
  Fish, and PowerShell.

**Batch processing**
- `--file FILE` / `-f FILE`: read queries from a file, one per line,
  for all subcommands (`search`, `genome`, `taxon`, `diff`).
- `-f -`: read queries from standard input, enabling direct
  composition with Unix pipelines.
- Progress bar displayed on stderr for batch inputs of more than one
  item, preserving stdout for piped downstream tools (`indicatif` 0.17).

**Output**
- `--outfmt csv|tsv|json` / `-O`: unified output format flag across
  all subcommands. Default is `csv` for `search` and `json` for
  `genome`, `taxon`, and `diff`.
- `--out FILE` / `-o FILE`: write output to a file instead of stdout.
- `--split` / `-s`: write one file per query or accession instead of
  merging output. Shell-safe filenames are derived from the query key.
- `--split-dir DIR` / `-d DIR`: specify the output directory for `--split`.

**Release targeting**
- `--release RELEASE` / `-r RELEASE`: target a specific GTDB release (e.g. `R214`,
  `R220`) for `genome card` and `genome metadata` endpoints. Accepts
  identifiers of the form `R<integer>`, validated at parse time.

**Reliability**
- Automatic retry with exponential backoff: transient errors (HTTP
  5xx, 429, and IO-level connection failures) are retried up to three
  times with delays of 1, 2, and 4 seconds, capped at 10 seconds.
- Automatic pagination: search results are fetched across all pages
  transparently, eliminating the 1 000-result truncation present in
  earlier versions. Queries on large genera such as *Escherichia*
  (52 587 accessions, 53 pages) complete without truncation.

**`taxon` subcommand**
- `--search`: search for a taxon by name in the current GTDB release,
  returning partial matches.
- `--all`: search for a taxon across all GTDB releases and NCBI
  (requires `--search`).
- `--genomes`: list genome accessions assigned to a taxon.
- `--reps`: restrict `--genomes` output to species representatives.
- `sandpiper_url` field added to taxon output, matching the updated
  GTDB API schema.

**`genome` subcommand**
- `--history` / `-H`: retrieve the complete taxonomic history of a
  genome across all GTDB releases.
- `--metadata` / `-m`: lightweight metadata-only response.
- CheckM2 fields (`checkm2_completeness`, `checkm2_contamination`,
  `checkm2_model`) added to the genome card struct.
- `marker_summary` and `ncbi_genome_representation` fields added,
  matching the updated GTDB API schema.

### Changed

**Breaking changes**
- The GTDB API base URL has changed from `api.gtdb.ecogenomic.org`
  to `gtdb-api.ecogenomic.org`. All endpoints updated accordingly.
- Output format defaults changed: `genome` and `taxon` now default to
  `json`; `search` retains `csv` as default.
- `--insecure` / `-k` is retained for backward compatibility but is
  no longer needed for normal GTDB API access and is no longer
  documented as a required flag.

**HTTP and TLS**
- Migrated from `ureq` 2.6 to `ureq` 3.3 with the `native-tls`
  backend, delegating certificate verification to the OS TLS stack.
  Resolves SSL certificate errors (`UnknownIssuer`) that affected all
  earlier versions against the GTDB API.
- All HTTP calls now route through a single `fetch_data` function with
  unified retry logic; previously each command had independent error
  handling with no retry.
- Error messages now include the offending accession or query to aid
  diagnosis during batch runs.

**Schema and deserialisation**
- `MetadataGene` fields (`checkm_completeness`, `checkm_contamination`,
  `checkm_strain_heterogeneity`, `coding_density`) corrected from
  `Option<String>` to `Option<f64>`, resolving `invalid type: floating
  point` parse errors on real API responses.
- `MetadataNCBI` numeric fields (`ncbi_taxid`, `ncbi_molecule_count`,
  `ncbi_cds_count`, `ncbi_translation_table`, etc.) corrected from
  `Option<String>` to `Option<i64>`.
- `Taxon` struct (`taxon` command): `n_desc_children` corrected to
  `Option<i64>`; `total` corrected to `Option<f64>`; `ncbi_tax_id`
  corrected to `Option<i64>`.
- GTDB API taxonomy key aliases (`gtdbDomain`, `gtdbPhylum`, etc.)
  added via `#[serde(alias)]` to handle the API's mixed camelCase /
  snake_case key format.

**`search` subcommand**
- `--field` flag values (`acc`, `org`, `gtdb`, `ncbi`) now correctly
  translate to the API-expected values (`ncbi_id`, `ncbi_org`,
  `gtdb_tax`, `ncbi_tax`) before URL construction. Previously the
  raw CLI value was sent directly, causing the API to ignore the
  field filter silently.
- Pagination is now transparent; previously results were silently
  truncated at 1 000 entries.

**`taxon --search` / `taxon --all`**
- `--all` now correctly requires `--search` at the CLI level,
  preventing silent misbehaviour when `--all` was passed alone.
- API `limit` parameter default corrected from 1 000 to 100 to match
  the GTDB API maximum, resolving HTTP 422 errors on taxon search
  queries.

**Minimum Rust version**
- Raised from 1.70.0 to 1.85 (required by `ureq` 3.3).

### Fixed

- `--insecure` no longer hardcoded to `true` during `--verbose`
  startup checks; SSL verification is now correctly enabled for
  status and version checks regardless of the `--insecure` flag.
- `genome --history` with `--out` no longer appends to an existing
  output file from a previous run; the first write now truncates.
- `genome --history` with multiple accessions and `--out` in CSV/TSV
  mode now writes a single header followed by all data rows, rather
  than repeating the header for each accession.
- `search --id` now returns clean accessions (e.g. `GCA_000005845.2`)
  instead of internal GTDB identifiers prefixed with `GB_` or `RS_`,
  which broke downstream use with `xgt genome -f -`.
- Debug `println!` removed from `filter_xsv` that was corrupting
  stdout output during normal usage.

### Removed

- `process_xsv_response` and `filter_xsv` functions removed; replaced
  by pagination-aware JSON accumulation with local CSV/TSV serialisation.
- `INTO_STRING_LIMIT` (20 MB cap) removed; no longer needed with
  transparent pagination.
- `native-tls = "0.2"` direct dependency removed; TLS is now managed
  entirely through `ureq`'s feature flag.

### Dependencies

| Dependency | v0.5.0 | v1.0.0 |
|---|---|---|
| `ureq` | 2.6.2 (native-tls feature) | 3.3.0 (native-tls feature) |
| `clap` | 4.5.35 | 4.5.35 |
| `serde` | 1.0.153 | 1.0 |
| `serde_json` | 1.0.94 | 1.0 |
| `anyhow` | 1.0.69 | 1.0 |
| `regex` | 1.11 | removed |
| `native-tls` | 0.2 (direct) | removed (via ureq feature) |
| `indicatif` | — | 0.17 (new) |
| `clap_complete` | — | 4.5 (new) |

---

## [v0.5.0] - 2024

- Initial public release with `search`, `genome`, and `taxon`
  subcommands.
- Basic GTDB API querying with CSV output.
- `--insecure` / `-k` required for all commands due to SSL
  certificate issues with the GTDB API.
