---
title: 'xgt enables efficient querying and parsing of the Genome Taxonomy Database'
tags:
  - Rust
  - bioinformatics
  - database
authors:
  - name: Anicet E. T. Ebou
    corresponding: true
    orcid: 0000-0003-4005-177X
    equal-contrib: false
    affiliation: 1
  - name: Dominique K. Koua
    orcid: 0000-0002-9078-8844
    equal-contrib: false
    affiliation: 1
affiliations:
 - name: Laboratoire de Microbiologie, Biotechnologies et Bioinformatique, Institut National Polytechnique Félix Houphouët-Boigny, BP 1093 Yamoussoukro, Côte d'Ivoire
   index: 1
date: 06 May 2024
bibliography: paper.bib
---

# Summary

Microbial genomic analyses heavily rely on curated genomes for various types of analyses, such as genomic, pangenomic, and structural variation assessments. This valuable data is accessible through the Genome Taxonomy Database (GTDB), which offers meticulously curated genomes and a standardized microbial taxonomy based on genome phylogeny [@parks:2018]. However, accessing this database programmatically can be challenging due to the limited options provided by its application programming interface (API). To address this issue, we introduce `xgt`, a free and open-source command-line tool and Rust package. 
`xgt` facilitates efficient querying and parsing of the GTDB by offering a user-friendly command-line interface. It comprises multiple modules that mirror the functionality of the database's API for search and querying purposes, while also providing enhanced parsing capabilities.

`xgt` offers a suite of tools, including:

- **xgt search**: This tool allows users to search the GTDB for a taxon by name. It offers both exact and partial matches, along with additional parsing capabilities. Additionally, it supports searching the GTDB using multiple names listed in a plain text file.  

- **xgt genome**: Users can use this tool to retrieve information about a genome. The `--metadata` option provides concise genome metadata such as accession and surveillance data, while `--history` retrieves the genome taxon history in the GTDB. The default option fetches nucleotide, gene, and taxonomy metadata of the genome.  

- **xgt taxon**: This tool fetches information about a specific taxon. Users can search for the direct descendants of a taxon and retrieve taxon genomes in the GTDB using partial or exact matches.

The `xgt` tools fetch real-time data from GTDB, ensuring that each query returns the latest information. Extensive unit tests have been incorporated to guarantee the reliability, correctness, and maintainability of `xgt`. Rust was selected for its emphasis on safety, performance, and concurrency, while package dependencies were meticulously chosen and kept to a minimum. `xgt` has been thoroughly tested on Linux/Unix, Mac OS (Darwin), and Windows platforms. As an open-source Rust package and command-line tool, `xgt` offers efficient and straightforward programmatic access to GTDB data, reducing the likelihood of error-prone manual web access during genomic data analysis.

`xgt` can be installed from the command line using the Rust package manager cargo after downloading the code source from GitHub. Each `xgt` tool feature an extensive manual available on the standard output using the help flag [-h] in the command line. The complete manual with examples can be viewed on the `xgt` website, available at [https://github.com/Ebedthan/xgt](https://github.com/Ebedthan/xgt).


# Statement of need

`xgt` contributes to addressing the persistent challenge of evaluating results within the framework of established reference databases. The Genome Taxonomy Database (GTDB) stands out as a widely utilized and high-quality resource in genomics, boasting over 500 thousand meticulously curated genomes [@parks:2022]. However, accessing GTDB solely via its API posed limitations due to its lack of built-in parsing capabilities. `xgt` resolves this issue by introducing parsing and querying functionalities, thereby enhancing accessibility and usability. 


# References