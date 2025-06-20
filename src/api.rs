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
