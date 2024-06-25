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
# Search subcommand: search GTDB
## Search all Escherichia (genus) genomes
xgt search g__Escherichia

## Search all genomes with genus name containing Escherichia
xgt search --partial g__Escherichia -o output.json

## Search from a list
xgt search -f list.txt

# Genome subcommand: information about a genome
## Get GTDB genome information
xgt genome GCA_001512625.1

## Get taxon history on GTDB
xgt genome --history GCA_001512625.1

## Get genome metadata
xgt genome --metadata GCA_001512625.1

# Taxon subcommand: information about a specific taxon
## Get direct descendant of a taxon
xgt taxon g__Escherichia

## Search for a taxon in GTDB's current release
xgt taxon --search g__Escherichia

## Search for a taxon in GTDB's current release with partial matching
xgt taxon --search --partial g__Escherichia
```

Full help is available from `xgt --help`.

## Certificate verification

`xgt` through `ureq` performs peer SSL certificate verification by default.
To tell `xgt` to _not_ verify the peer, use the `-k/--insecure` option.
Currently (as of Apr 28, 2024), you should add this option to your command to get the desired result as GTDB API's server has a certificate issue.

## Minimum supported Rust version
`xgt` minimum [Rust](https://www.rust-lang.org/) version is 1.70.0.

## Semver
`xgt` is following [Semantic Versioning 2.0](https://semver.org/).

## Licence
`xgt` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT) for details.

## Note
Unstable work is on dev branch.