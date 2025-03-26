#[derive(Debug, Clone, Default)]
pub struct TaxonAPI {
    name: String,
}

impl TaxonAPI {
    /// Creates a new `TaxonAPI` instance from a given name.
    pub fn new(name: impl Into<String>) -> Self {
        TaxonAPI { name: name.into() }
    }

    /// Constructs a URL for a name request.
    pub fn get_name_request(&self) -> String {
        format!("https://api.gtdb.ecogenomic.org/taxon/{}", self.name)
    }

    /// Constructs a URL for a search request.
    pub fn get_search_request(&self, limit: u32) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/taxon/search/{}?limit={}",
            self.name, limit
        )
    }

    /// Constructs a URL for a search request across all releases.
    pub fn get_search_all_request(&self, limit: u32) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/taxon/search/{}/all-releases?limit={}",
            self.name, limit
        )
    }

    /// Constructs a URL for a genome request.
    pub fn get_genomes_request(&self, is_reps_only: bool) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/taxon/{}/genomes?sp_reps_only={}",
            self.name, is_reps_only
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let api = TaxonAPI::new("test_taxon");
        assert_eq!(api.name, "test_taxon");
    }

    #[test]
    fn test_get_name_request() {
        let api = TaxonAPI::new("test_taxon");
        let expected_url = "https://api.gtdb.ecogenomic.org/taxon/test_taxon";
        assert_eq!(api.get_name_request(), expected_url);
    }

    #[test]
    fn test_get_search_request() {
        let api = TaxonAPI::new("test_taxon");
        let expected_url = "https://api.gtdb.ecogenomic.org/taxon/search/test_taxon?limit=1000";
        assert_eq!(api.get_search_request(1000), expected_url);
    }

    #[test]
    fn test_get_search_all_request() {
        let api = TaxonAPI::new("test_taxon");
        let expected_url =
            "https://api.gtdb.ecogenomic.org/taxon/search/test_taxon/all-releases?limit=1000";
        assert_eq!(api.get_search_all_request(1000), expected_url);
    }

    #[test]
    fn test_get_genomes_request() {
        let api = TaxonAPI::new("test_taxon");
        let expected_url_reps =
            "https://api.gtdb.ecogenomic.org/taxon/test_taxon/genomes?sp_reps_only=true";
        let expected_url_non_reps =
            "https://api.gtdb.ecogenomic.org/taxon/test_taxon/genomes?sp_reps_only=false";
        assert_eq!(api.get_genomes_request(true), expected_url_reps);
        assert_eq!(api.get_genomes_request(false), expected_url_non_reps);
    }
}
