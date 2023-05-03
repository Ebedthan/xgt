# xgt
[![Continuous Integration](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/xgt/branch/main/graph/badge.svg?token=OFAOB6K5KB)](https://codecov.io/gh/Ebedthan/xgt)
<a href="https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat">
</a>
<a href="https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE">
    <img src="https://img.shields.io/badge/license-APACHE-blue?style=flat">
</a>

`xgt` is a [Rust](https://www.rust-lang.org/) tool that enables efficient querying and parsing of the GTDB database. `xgt` consists of a collection of commands mirroring the GTDB API and providing additional parsing capability.

# Installation

```
git clone https://github.com/Ebedthan/xgt.git
cd xgt

# If default rust install directory is ~/.cargo
cargo install --path . --root ~/.cargo
xgt -h
```

# Quick start guide

```
# Search subcommand: search GTDB ----------------------------------------------
## Search all Aminobacter (genus) genomes
xgt search Aminobacter

## Search all genomes with genus name containing aminobacter
xgt search -p aminobacter

## Search from a list
xgt search -f list.txt

# Genome subcommand: information about a genome -------------------------------
## Get GTDB genome information
xgt genome GCA_001512625.1

## Get taxon history on GTDB
xgt genome --history GCA_001512625.1

## Get genome metadata
xgt genome --metadata GCA_001512625.1

# Taxon subcommand: information about a specific taxon ------------------------
## Get direct descendant of a taxon
xgt taxon g__Aminobacter

## Search for a taxon in GTDB's current release
xgt taxon --search g__Aminobacter

## Search for a taxon in GTDB's current release with partial matching
xgt taxon --search -p g__Aminobacter
```

Full help is available from `xgt --help`.

### Minimum supported Rust version
`xgt` minimum [Rust](https://www.rust-lang.org/) version is 1.64.0.

### Semver
`xgt` is following [Semantic Versioning 2.0](https://semver.org/).

### Licence
`xgt` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT) for details.