#[derive(Debug, Clone)]
pub struct TaxonAPI {
    name: String,
}

impl TaxonAPI {
    pub fn from(name: String) -> Self {
        TaxonAPI { name }
    }
    pub fn get_name_request(&self) -> String {
        format!("https://api.gtdb.ecogenomic.org/taxon/{}", self.name)
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_search_request(&self) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/taxon/search/{}?limit=1000000",
            self.name
        )
    }
    pub fn get_search_all_request(&self) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/taxon/search/{}/all-releases?limit=10000000",
            self.name
        )
    }
    pub fn get_genomes_request(&self, is_reps_only: bool) -> String {
        format!(
            "https://api.gtdb.ecogenomic.org/taxon/{}/genomes?sp_reps_only={}",
            self.name, is_reps_only
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from() {
        let taxon_api = TaxonAPI::from("s__Escherichia coli".to_string());
        assert_eq!(taxon_api.name, "s__Escherichia coli");
    }

    #[test]
    fn test_get_name_request() {
        let taxon_api = TaxonAPI::from("s__Escherichia coli".to_string());
        let expected = "https://api.gtdb.ecogenomic.org/taxon/s__Escherichia coli".to_string();
        assert_eq!(taxon_api.get_name_request(), expected);
    }

    #[test]
    fn test_get_name() {
        let taxon_api = TaxonAPI::from("s__Escherichia coli".to_string());
        assert_eq!(taxon_api.get_name(), "s__Escherichia coli");
    }

    #[test]
    fn test_get_search_request() {
        let taxon_api = TaxonAPI::from("s__Escherichia coli".to_string());
        let expected =
            "https://api.gtdb.ecogenomic.org/taxon/search/s__Escherichia coli?limit=1000000"
                .to_string();
        assert_eq!(taxon_api.get_search_request(), expected);
    }

    #[test]
    fn test_get_search_all_request() {
        let taxon_api = TaxonAPI::from("s__Escherichia coli".to_string());
        let expected = "https://api.gtdb.ecogenomic.org/taxon/search/s__Escherichia coli/all-releases?limit=10000000".to_string();
        assert_eq!(taxon_api.get_search_all_request(), expected);
    }
}
