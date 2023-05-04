#[derive(Debug, Clone, PartialEq, Copy)]
pub enum GenomeRequestType {
    Metadata,
    TaxonHistory,
    Card,
}

impl GenomeRequestType {
    pub fn to_string(grt: GenomeRequestType) -> String {
        match grt {
            GenomeRequestType::Card => String::from("card"),
            GenomeRequestType::Metadata => String::from("metadata"),
            GenomeRequestType::TaxonHistory => String::from("taxon-history"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenomeAPI {
    accession: String,
}

impl GenomeAPI {
    pub fn from(accession: String) -> Self {
        GenomeAPI { accession }
    }

    pub fn request(&self, request_type: GenomeRequestType) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/genome/{}/{}",
            self.accession,
            GenomeRequestType::to_string(request_type)
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_genome_request_type_to_string() {
        assert_eq!(
            GenomeRequestType::to_string(GenomeRequestType::Card),
            String::from("card")
        );
        assert_eq!(
            GenomeRequestType::to_string(GenomeRequestType::Metadata),
            String::from("metadata")
        );
        assert_eq!(
            GenomeRequestType::to_string(GenomeRequestType::TaxonHistory),
            String::from("taxon-history")
        );
    }

    #[test]
    fn test_genome_api_request() {
        let genome_api = GenomeAPI::from(String::from("GCA_009858685.1"));
        assert_eq!(
            genome_api.request(GenomeRequestType::Card),
            String::from("https://api.gtdb.ecogenomic.org/genome/GCA_009858685.1/card")
        );
        assert_eq!(
            genome_api.request(GenomeRequestType::Metadata),
            String::from("https://api.gtdb.ecogenomic.org/genome/GCA_009858685.1/metadata")
        );
        assert_eq!(
            genome_api.request(GenomeRequestType::TaxonHistory),
            String::from("https://api.gtdb.ecogenomic.org/genome/GCA_009858685.1/taxon-history")
        );
    }
}
