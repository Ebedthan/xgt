# 🚀 xgt

**Fast and Flexible GTDB Query Tool, Built in Rust**

[![CI](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/xgt/graph/badge.svg?token=OFAOB6K5KB)](https://codecov.io/gh/Ebedthan/xgt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat)](https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat)](https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE)

## 🧭 What is `xgt`?

`xgt` is a blazing-fast command-line utility written in [Rust](https://www.rust-lang.org/) for querying and parsing data from the [GTDB](https://gtdb.ecogenomic.org/) (Genome Taxonomy Database). It mirrors core GTDB API functions and adds flexible parsing, metadata retrieval, and taxonomy exploration tools-making GTDB data more accessible for researchers and developers.


## ✨ Key Features

* 🔍 **`search`**: Search the GTDB using exact or partial name matches, or from a file of names.
* 🧬 **`genome`**: Retrieve genome metadata, taxonomic history, and more.
* 🌳 **`taxon`**: Explore taxonomic lineages, descendants, and associated genomes.


## 📦 Installation

### 🛠️ From Source (via Cargo)

```bash
git clone https://github.com/Ebedthan/xgt.git
cd xgt
cargo install --path . --root ~/.cargo
xgt --help
```

### 📁 Prebuilt Binaries

Download binaries for your platform from the [releases page](https://github.com/Ebedthan/xgt/releases):

* macOS (Apple Silicon): [Download](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-aarch64-apple-darwin.tar.xz) • [Checksum](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-aarch64-apple-darwin.tar.xz.sh256)
* macOS (Intel): [Download](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-x86_64-apple-darwin.tar.xz) • [Checksum](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-x86_64-apple-darwin.tar.xz.sha256)
* Linux (x86\_64): [Download](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-x86_64-unknown-linux-gnu.tar.xz) • [Checksum](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-x86_64-unknown-linux-gnu.tar.xz.sha256)
* Windows (x86\_64): [Download](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-x86_64-pc-windows-msvc.zip) • [Checksum](https://github.com/Ebedthan/xgt/releases/download/v0.5.0/xgt-v0.5.0-x86_64-pc-windows-msvc.zip.sha256)


## 💡 Usage Examples

```bash
# 🔍 Search for Escherichia genomes (exact match)
xgt search -kw g__Escherichia

# 🔎 Search by name with output to CSV
xgt search -k -o results.csv g__Escherichia

# 📁 Search from a list of names
xgt search -k -f list.txt

# 🧬 Genome metadata and taxonomy
xgt genome -k GCA_001512625.1

# 📜 Taxon lineage exploration
xgt taxon -k --search g__Escherichia
xgt taxon -k g__Escherichia
```

## 🧰 Subcommand Highlights

### `search`

* Exact or partial match search
* Input from CLI or file
* Output formatting (CSV)

### `genome`

* Retrieve genome metadata (`--metadata`)
* Access taxonomic history (`--history`)
* Full metadata by default

### `taxon`

* Fetch direct descendants
* Search taxon names with partial matches
* Explore genomes within a taxon


## 🐞 Reporting Issues

Found a bug or want to request a feature? [Open an issue](https://github.com/Ebedthan/xgt/issues). When submitting bugs, please include:

* OS and architecture
* Version of `xgt`
* Reproduction steps or input


## 📜 License

This project is licensed under both the [MIT License](LICENSE-MIT) and the [Apache 2.0 License](LICENSE-APACHE). You may choose the license that best suits your needs.


## 🦀 Developer Notes

* **Minimum Rust version**: `1.74.1`
* **Follows**: [Semantic Versioning](https://semver.org/)
