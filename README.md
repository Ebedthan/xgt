# xgt

*Efficient querying and parsing of the GTDB database*

[![Continuous Integration](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/xgt/branch/main/graph/badge.svg?token=OFAOB6K5KB)](https://codecov.io/gh/Ebedthan/xgt)
<a href="https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat">
</a>
<a href="https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE">
    <img src="https://img.shields.io/badge/license-APACHE-blue?style=flat">
</a>

> [!WARNING]
> You should add the `-k/--insecure` option to all your current command as the GTDB API has currently SSL issue.
> We have not yet decided to switch it as a default configuration as disabling peer SSL certificate verification can be a critical issue.

## 🗺️ Overview

`xgt` is a [Rust](https://www.rust-lang.org/) tool that enables efficient querying and parsing of the GTDB database. `xgt` consists of a collection of commands mirroring the GTDB API and providing additional parsing capability.

## 📋 Features

### `search` subcommand
It offers both exact and partial matches, along with additional parsing capabilities. Additionally, it supports searching the GTDB using multiple names listed in a plain text file.

### `genome` subcommand
It can be used to retrieve information about a genome. The --metadata option provides concise genome metadata such as accession and surveillance data, while --history retrieves the genome taxon history in the GTDB. The default option fetches nucleotide, gene, and taxonomy metadata of the genome.

### `taxon` subcommand
This tool fetches information about a specific taxon. Users can search for
the direct descendants of a taxon and retrieve taxon genomes in the GTDB using partial or exact matches.

### Certificate verification

`xgt` through `ureq` performs peer SSL certificate verification by default.
To tell `xgt` to _not_ verify the peer, use the `-k/--insecure` option.
Currently (as of Apr 28, 2024), you should add this option to your command to get the desired result as GTDB API's server has a certificate issue.

## 🔧 Installing

### From source

```
git clone https://github.com/Ebedthan/xgt.git
cd xgt

# If default rust install directory is ~/.cargo
cargo install --path . --root ~/.cargo
xgt -h
```

### Using binaries

Please find the binaries for the latest release using the [release page](https://github.com/Ebedthan/xgt/releases) or using the direct link below:
* [Apple Silicon macOS](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-aarch64-apple-darwin.tar.xz) with its [checksum](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-aarch64-apple-darwin.tar.xz.sha256)
* [Intel macOS](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-x86_64-apple-darwin.tar.xz) with its [checksum](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-x86_64-apple-darwin.tar.xz.sha256)
* [x64 Windows](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-x86_64-pc-windows-msvc.zip) with its [checksum](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-x86_64-pc-windows-msvc.zip.sha256)
* [x64 Linux](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-x86_64-unknown-linux-gnu.tar.xz) with its [checksum](https://github.com/Ebedthan/xgt/releases/download/v0.4.0/xgt-x86_64-unknown-linux-gnu.tar.xz.sha256)

## 💡 Example

```
# Search subcommand: search GTDB
## Search all Escherichia (genus) genomes
xgt search -kw g__Escherichia

## Search all genomes with genus name containing Escherichia
xgt search -k -o output.csv g__Escherichia

## Search from a list
xgt search -k -f list.txt

# Genome subcommand: information about a genome
## Get GTDB genome information
xgt genome -k GCA_001512625.1

## Get taxon history on GTDB
xgt genome -k --history GCA_001512625.1

## Get genome metadata
xgt genome -k --metadata GCA_001512625.1

# Taxon subcommand: information about a specific taxon
## Get direct descendant of a taxon
xgt taxon -k g__Escherichia

## Search for a taxon in GTDB's current release
xgt taxon -kw --search g__Escherichia

## Search for a taxon in GTDB's current release with partial matching
xgt taxon -k --search g__Escherichia
```

## ⚠️ Issue Tracker

Found a bug ? Have an enhancement request ? Head over to the [GitHub issue
tracker](https://github.com/Ebedthan/xgt/issues) if you need to report
or ask something. If you are filing in on a bug, please include as much
information as you can about the issue, and try to recreate the same bug
in a simple, easily reproducible situation.

## ⚖️ License

`xgt` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT) for details.

Full help is available from `xgt --help`.

## Minimum supported Rust version
`xgt` minimum [Rust](https://www.rust-lang.org/) version is 1.70.0.

## Semver
`xgt` is following [Semantic Versioning 2.0](https://semver.org/).

## Note
Unstable work is on dev branch.
