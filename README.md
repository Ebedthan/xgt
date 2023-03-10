# xgt

`xgt` is a Rust based tool that enables efficient querying and parsing of the GTDB database. `xgt` consists of a collection of commands mirroring the GTDB API and provide additionnal fast and efficient parsing capability of the result.

# Installation

```
cargo install --release xgt
```

Alternative:

Download a release binary and you are done!

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
Dual-licensed under MIT and Apache.