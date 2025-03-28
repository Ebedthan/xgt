use std::fmt;

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
pub struct GenomeAPI {
    accession: String,
}

impl From<String> for GenomeAPI {
    fn from(accession: String) -> Self {
        GenomeAPI { accession }
    }
}

impl From<&str> for GenomeAPI {
    fn from(accession: &str) -> Self {
        GenomeAPI {
            accession: accession.to_string(),
        }
    }
}

impl GenomeAPI {
    pub fn request(&self, request_type: GenomeRequestType) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/genome/{}/{}",
            self.accession, request_type
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genome_request_type_to_string() {
        assert_eq!(GenomeRequestType::Card.to_string(), "card");
        assert_eq!(GenomeRequestType::Metadata.to_string(), "metadata");
        assert_eq!(GenomeRequestType::TaxonHistory.to_string(), "taxon-history");
    }

    #[test]
    fn test_genome_api_from_string() {
        let accession = String::from("GCA_000001405.28");
        let api: GenomeAPI = accession.clone().into();
        assert_eq!(api.accession, accession);
    }

    #[test]
    fn test_genome_api_request_metadata() {
        let api = GenomeAPI::from("GCA_000001405.28".to_string());
        let url = api.request(GenomeRequestType::Metadata);
        assert_eq!(
            url,
            "https://api.gtdb.ecogenomic.org/genome/GCA_000001405.28/metadata"
        );
    }

    #[test]
    fn test_genome_api_request_taxon_history() {
        let api = GenomeAPI::from("GCA_000001405.28".to_string());
        let url = api.request(GenomeRequestType::TaxonHistory);
        assert_eq!(
            url,
            "https://api.gtdb.ecogenomic.org/genome/GCA_000001405.28/taxon-history"
        );
    }

    #[test]
    fn test_genome_api_request_card() {
        let api = GenomeAPI::from("GCA_000001405.28".to_string());
        let url = api.request(GenomeRequestType::Card);
        assert_eq!(
            url,
            "https://api.gtdb.ecogenomic.org/genome/GCA_000001405.28/card"
        );
    }
}
