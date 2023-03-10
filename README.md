# xgt
[![Continuous Integration](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml/badge.svg)](https://github.com/Ebedthan/xgt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Ebedthan/xgt/branch/main/graph/badge.svg?token=OFAOB6K5KB)](https://codecov.io/gh/Ebedthan/xgt)
<a href="https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat">
</a>
<a href="https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE">
    <img src="https://img.shields.io/badge/license-APACHE-blue?style=flat">
</a>

`xgt` is a Rust based tool that enables efficient querying and parsing of the GTDB database. `xgt` consists of a collection of commands mirroring the GTDB API and provide additionnal fast and efficient parsing capability of the result.

# Installation

```
cargo install --release xgt
```

Alternative from source:

```
git clone https://github.com/Ebedthan/xgt.git
cd xgt

cargo build --release
cargo test
cargo install --path .
```

# Quick start guide

```
# Search all Aminobacter genomes
xgt search 'Aminobacter'

# Search all genomes with genus name containing aminobacter
xgt search -p 'aminobacter'

# Get GTDB genome information
xgt genome GCA_001512625.1

# Get taxon history on GTDB
xgt genome --history GCA_001512625.1

# Get genome metadata
xgt genome --metadata GCA_001512625.1
```

### Licence
xgt is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/Ebedthan/xgt/blob/main/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/Ebedthan/xgt/blob/main/LICENSE-MIT) for details.