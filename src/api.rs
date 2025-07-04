use std::fmt;

#[derive(Debug, Clone)]
pub enum GtdbApiRequest {
    Taxon {
        name: String,
        kind: TaxonEndPoint,
        limit: Option<u32>,
        is_reps_only: Option<bool>,
    },
    Search {
        query: String,
        page: u16,
        items_per_page: u32,
        sort_by: String,
        sort_desc: bool,
        search_field: String,
        filter_text: String,
        gtdb_species_rep_only: bool,
        ncbi_type_material_only: bool,
        output_format: String,
    },
    Genome {
        accession: String,
        request_type: GenomeRequestType,
    },
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum GenomeRequestType {
    Metadata,
    TaxonHistory,
    Card,
}

impl fmt::Display for GenomeRequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GenomeRequestType::Card => "card",
            GenomeRequestType::Metadata => "metadata",
            GenomeRequestType::TaxonHistory => "taxon-history",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for GenomeRequestType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "metadata" => Ok(GenomeRequestType::Metadata),
            "taxon-history" => Ok(GenomeRequestType::TaxonHistory),
            "card" => Ok(GenomeRequestType::Card),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaxonEndPoint {
    Name,
    Search,
    SearchAll,
    Genomes,
}

impl GtdbApiRequest {
    pub fn to_url(&self) -> String {
        match self {
            GtdbApiRequest::Taxon {
                name,
                kind,
                limit,
                is_reps_only,
            } => match kind {
                TaxonEndPoint::Name => format!("https://api.gtdb.ecogenomic.org/taxon/{}", name),
                TaxonEndPoint::Search => format!(
                    "https://api.gtdb.ecogenomic.org/taxon/search/{}?limit={}",
                    name,
                    limit.unwrap_or(1000)
                ),
                TaxonEndPoint::SearchAll => format!(
                    "https://api.gtdb.ecogenomic.org/taxon/search/{}/all-releases?limit={}",
                    name,
                    limit.unwrap_or(1000)
                ),
                TaxonEndPoint::Genomes => format!(
                    "https://api.gtdb.ecogenomic.org/taxon/{}/genomes?sp_reps_only={}",
                    name,
                    is_reps_only.unwrap_or(false)
                ),
            },

            GtdbApiRequest::Search {
                query,
                page,
                items_per_page,
                sort_by,
                sort_desc,
                search_field,
                filter_text,
                gtdb_species_rep_only,
                ncbi_type_material_only,
                output_format,
            } => {
                let mut url = format!(
                    "https://api.gtdb.ecogenomic.org/search/gtdb{}?",
                    if output_format == "json" {
                        String::new()
                    } else {
                        format!("/{}", output_format)
                    }
                );
                let mut params = vec![
                    format!("search={}", query),
                    format!("page={}", page),
                    format!("itemsPerPage={}", items_per_page),
                    format!("searchField={}", search_field),
                ];

                if !sort_by.is_empty() {
                    params.push(format!("sortBy={}", sort_by));
                }

                if *sort_desc {
                    params.push("sortDesc=true".into());
                }

                if !filter_text.is_empty() {
                    params.push(format!("filterText={}", filter_text));
                }

                if *gtdb_species_rep_only {
                    params.push("gtdbSpeciesRepOnly=true".into());
                }

                if *ncbi_type_material_only {
                    params.push("ncbiTypeMaterialOnly=true".into());
                }

                url.push_str(&params.join("&"));
                url
            }

            GtdbApiRequest::Genome {
                accession,
                request_type,
            } => format!(
                "https://api.gtdb.ecogenomic.org/genome/{}/{}",
                accession, request_type
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display_genome_request_type() {
        assert_eq!(GenomeRequestType::Metadata.to_string(), "metadata");
        assert_eq!(GenomeRequestType::TaxonHistory.to_string(), "taxon-history");
        assert_eq!(GenomeRequestType::Card.to_string(), "card");
    }

    #[test]
    fn test_from_str_valid_inputs() {
        assert_eq!(
            GenomeRequestType::from_str("metadata").unwrap(),
            GenomeRequestType::Metadata
        );
        assert_eq!(
            GenomeRequestType::from_str("taxon-history").unwrap(),
            GenomeRequestType::TaxonHistory
        );
        assert_eq!(
            GenomeRequestType::from_str("card").unwrap(),
            GenomeRequestType::Card
        );
    }

    #[test]
    fn test_from_str_invalid_input() {
        assert!(GenomeRequestType::from_str("invalid").is_err());
        assert!(GenomeRequestType::from_str("").is_err());
        assert!(GenomeRequestType::from_str("CARD").is_err()); // case sensitive
    }

    #[test]
    fn test_taxon_name_url() {
        let req = GtdbApiRequest::Taxon {
            name: "Bacillus".into(),
            kind: TaxonEndPoint::Name,
            limit: None,
            is_reps_only: None,
        };
        assert_eq!(
            req.to_url(),
            "https://api.gtdb.ecogenomic.org/taxon/Bacillus"
        );
    }

    #[test]
    fn test_taxon_search_url() {
        let req = GtdbApiRequest::Taxon {
            name: "Bacillus".into(),
            kind: TaxonEndPoint::Search,
            limit: Some(500),
            is_reps_only: None,
        };
        assert_eq!(
            req.to_url(),
            "https://api.gtdb.ecogenomic.org/taxon/search/Bacillus?limit=500"
        );
    }

    #[test]
    fn test_taxon_genomes_url() {
        let req = GtdbApiRequest::Taxon {
            name: "Bacillus".into(),
            kind: TaxonEndPoint::Genomes,
            limit: None,
            is_reps_only: Some(true),
        };
        assert_eq!(
            req.to_url(),
            "https://api.gtdb.ecogenomic.org/taxon/Bacillus/genomes?sp_reps_only=true"
        );
    }

    #[test]
    fn test_search_api_url_csv() {
        let req = GtdbApiRequest::Search {
            query: "Bacillus".into(),
            page: 2,
            items_per_page: 500,
            sort_by: "name".into(),
            sort_desc: true,
            search_field: "gtdb_tax".into(),
            filter_text: "species".into(),
            gtdb_species_rep_only: true,
            ncbi_type_material_only: true,
            output_format: "csv".into(),
        };

        let url = req.to_url();
        assert!(url.starts_with("https://api.gtdb.ecogenomic.org/search/gtdb/csv?"));
        assert!(url.contains("search=Bacillus"));
        assert!(url.contains("sortBy=name"));
        assert!(url.contains("sortDesc=true"));
        assert!(url.contains("searchField=gtdb_tax"));
        assert!(url.contains("gtdbSpeciesRepOnly=true"));
        assert!(url.contains("ncbiTypeMaterialOnly=true"));
    }

    #[test]
    fn test_genome_card_request_url() {
        let req = GtdbApiRequest::Genome {
            accession: "GCF_000005845.2".into(),
            request_type: GenomeRequestType::Card,
        };

        assert_eq!(
            req.to_url(),
            "https://api.gtdb.ecogenomic.org/genome/GCF_000005845.2/card"
        );
    }
}
