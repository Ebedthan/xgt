# Changelog

All notable changes to xgt are documented in this file. 
Versions follow [Semantic Versioning](https://semver.org/).

## [v1.2.0] - 2026-08-17

Minor release focused on performance, correctness, and code quality.
Introduces a transparent local response cache, fixes all known URL
encoding bugs, and delivers a significant codebase refactor that reduces
the source by ~200 lines while improving maintainability.

### Added

**Transparent local response cache**
- All API responses are now cached locally in a SQLite database
  (`~/.cache/xgt/cache.db` on Linux/macOS,
  `%LOCALAPPDATA%\xgt\cache.db` on Windows).
- Cache is enabled by default. Pass `--no-cache` (global flag) to
  bypass it for a single request.
- TTLs are set per endpoint type: `diff` results (fixed release pairs
  are immutable) are cached for 1 year; `genome --history` for 90 days;
  `genome card`, `genome metadata`, and `taxon` for 30 days; `search`
  results for 7 days.
- `--cache-info` prints entry count, expired entry count, and database
  size on disk, then exits.
- `--clear-cache` deletes all cached responses, then exits.
- Cache failures (missing file, locked database, corrupt entry) fall
  through silently to a live API request — the cache never breaks the
  tool.
- Expired entries are evicted automatically on each cache write, with
  no separate maintenance step required.
- New dependency: `rusqlite` 0.31 (bundled SQLite, no system library
  required); `sha2` 0.10 for cache key hashing; `dirs` 5.0 for
  XDG-compliant cache directory resolution.

### Fixed

**`search --id` and `search --count` incorrectly wrote a CSV header**
- After the `BatchWriter` refactor, `write_global_header` was called
  unconditionally, prepending the CSV column header to `--id` and
  `--count` output. Both modes produce plain values with no header row.
- Fixed by guarding `write_global_header` with
  `!args.id && !args.count` in `search()`.

### Changed

**Codebase refactor — ~200 lines removed**
- `OutputFormat::sep()` method added to `utils.rs` — eliminates 11
  repeated `if outfmt == Tsv { "\t" } else { "," }` expressions across
  all command files.
- `From<String>` implementations for `SearchField` and `OutputFormat`
  replaced with `match s.as_str()` — cleaner and exhaustiveness-checked.
- `From<&str>` added for both types — eliminates 11 unnecessary
  `.clone()` calls at conversion sites.
- `BatchWriter` struct added to `utils.rs` — encapsulates the
  header-once / append / truncate / split write logic that was
  previously duplicated across `diff.rs` (once) and `genome.rs`
  (twice).
- `fetch_batch` generic function added to `utils.rs` — unifies
  `fetch_and_save_genome_data` (genome.rs) and `fetch_and_write_json`
  (taxon.rs). A single-item call replaces the old taxon path; a
  multi-item call replaces the old genome batch loop.
- `fetch_json` wrapper in `taxon.rs` removed (inlined at call site).
- Four dead functions removed from `search.rs`:
  `handle_id_or_count_response`, `process_response`,
  `handle_json_response`, `handle_xsv_response`.
- Six trivial getter methods removed from `search.rs`:
  `SearchResult::get_accession`, `get_ncbi_org_name`,
  `get_ncbi_taxonomy`, `get_gtdb_taxonomy`,
  `SearchResults::get_total_rows` — fields accessed directly.
- Progress bar helpers `bar_tick`, `bar_inc`, `bar_finish` added to
  `utils.rs` — eliminates repeated `if let Some(ref bar)` boilerplate
  across `diff.rs` and `genome.rs`.
- Shadowed `outfmt` recomputation inside the XSV arm of `search()`
  removed.
- Typo fixed: `reache` → `reach` in `utils.rs` error message.
- Doctest examples on private functions in `search.rs` marked `no_run`.
- Beta CI job marked `continue-on-error: true`.

### Dependencies

| Dependency | v1.1.x | v1.3.0 |
|---|---|---|
| `rusqlite` | — | 0.31 (bundled, new) |
| `sha2` | — | 0.10 (new) |
| `dirs` | — | 5.0 (new) |
| All others | unchanged | unchanged |


## [v1.1.1] - 2026-07-28

### Fixed

- **URL encoding bug: species-level names caused request failure.**
  User input containing spaces (e.g. `s__Escherichia coli`,
  `s__Homo sapiens`) was interpolated directly into URL query strings
  without percent-encoding, causing `ureq` to reject the request with
  `invalid uri character` after three retry attempts. All six
  interpolation sites in `api.rs` that accept free-text user input
  (`query`, `filter_text`, and all four `name` interpolations in the
  taxon endpoints) now pass values through `encode_query_value()`,
  which percent-encodes all characters outside the RFC 3986 unreserved
  set. Fields that cannot contain spaces (integers, enum-validated
  values, INSDC accessions, release identifiers) are unchanged.
  This bug affected any query at the species level and any organism
  name search containing a space.

  Affected commands:
  - `xgt search 's__Escherichia coli'`
  - `xgt search 'Escherichia coli' -F org`
  - `xgt taxon 's__Escherichia coli'`
  - `xgt taxon 's__Escherichia coli' --genomes`
  - `xgt taxon --search 'Escherichia coli'`
 
## [v1.1.0] - 2026-07-22
 
Focused release delivering three promised post-v1.0.0 improvements:
tolerant deserialisation, parallel pagination, and version awareness.
Also fixes two bugs reported against v1.0.0.
 
### Added
 
**Tolerant deserialisation**
- All struct fields in `genome.rs`, `taxon.rs`, and `diff.rs` now use
  custom `serde` deserialiser functions (`deser_opt_f64`, `deser_opt_i64`,
  `deser_opt_i32`, `deser_opt_string`, `deser_opt_bool`, `deser_bool`)
  that accept any JSON scalar type and coerce it to the declared Rust type.
- Missing fields now deserialise to `None` via `#[serde(default)]` rather
  than producing an error.
- Previously, a field changing from a quoted string to a bare number in the
  GTDB API caused an `invalid type` panic. This class of error is now
  handled silently for all struct fields.
**Parallel pagination**
- `xgt search` now fetches result pages concurrently using
  `std::thread` and `std::sync::mpsc`, with no new dependencies.
- Pages are dispatched in chunks so that at most `--max-concurrent N`
  connections are open simultaneously (default: 5), preventing HTTP 429
  rate-limit errors.
- Results are sorted by page number before merging to guarantee row order
  matches the API's natural order regardless of thread completion order.
- New `--max-concurrent N` flag on `xgt search` lets users tune
  concurrency for their network and API rate-limit tolerance.
- Wall-time improvement for multi-page queries: median time for
  `g__Escherichia` (53 pages) reduced from ~5 s to under 2 s on a home
  network under typical conditions.
**`--check-update` flag**
- `xgt --check-update` queries the GitHub releases API and prints whether
  a newer version is available.
- Uses the existing `ureq` agent and TLS configuration — no new
  dependencies.
- The `command` subcommand field is now `Option<Commands>` so
  `xgt --check-update` works without a subcommand argument.
**Integration test suite**
- `tests/integration.rs`: 22 end-to-end tests against the live GTDB API
  covering all four subcommands and the API availability checks.
- Gated behind `--features integration-tests` so CI unit test runs are
  unaffected.
- New `[lib]` target in `Cargo.toml` exposes internal modules for use by
  the integration test crate (`xgt::cmd::*`, `xgt::utils`).
- Run with:
  `cargo test --features integration-tests --test integration`
### Fixed
 
- **Bug 5 — `--id` returned prefixed gid instead of clean accession.**
  `xgt search --id` previously returned internal GTDB identifiers
  prefixed with `GB_` or `RS_` (e.g. `GB_GCA_000005845.2`) instead of
  the clean INSDC accession (`GCA_000005845.2`). This broke the documented
  pipeline `xgt search --id | xgt genome -f -` with HTTP 400 errors.
  The `accession` field is now used, with `gid` as a fallback only when
  `accession` is absent.
- **Bug 2 — `--verbose` disabled SSL verification.**
  `is_gtdb_db_online` and `get_api_version` were called with `insecure:
  true` hardcoded, silently disabling certificate verification on every
  verbose invocation regardless of whether `--insecure` was passed.
  Both now receive `false`.
### Changed
 
- `command` field on `Cli` changed from `Commands` to `Option<Commands>`
  to support `xgt --check-update` without a subcommand. The binary still
  prints help when called with no arguments via `arg_required_else_help`
  logic.
- Doctest examples on private functions in `search.rs` marked `no_run`
  to prevent false failures when compiled as a lib target.
- CI beta job marked `continue-on-error: true` — beta is informational
  and must not block stable releases.
### Dependencies
 
No new runtime dependencies. `serde_json::Value` (already a transitive
dependency) is now used directly in `utils.rs` for the deserialisation
helpers.
 

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
| `indicatif` | - | 0.17 (new) |
| `clap_complete` | - | 4.5 (new) |

---

## [v0.5.0] - 2024

- Initial public release with `search`, `genome`, and `taxon`
  subcommands.
- Basic GTDB API querying with CSV output.
- `--insecure` / `-k` required for all commands due to SSL
  certificate issues with the GTDB API.
