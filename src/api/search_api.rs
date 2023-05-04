use crate::cmd::utils::SearchArgs;

#[derive(Debug, Clone)]
pub struct SearchAPI {
    search: String,
    page: i32,
    items_per_page: i32,
    sort_by: String,
    sort_desc: String,
    search_field: String,
    filter_text: String,
    gtdb_species_rep_only: bool,
    ncbi_type_material_only: bool,
}

impl Default for SearchAPI {
    fn default() -> Self {
        SearchAPI {
            search: String::new(),
            page: 1,
            items_per_page: 1_000_000_000,
            sort_by: String::new(),
            sort_desc: String::new(),
            search_field: "all".to_string(),
            filter_text: String::new(),
            gtdb_species_rep_only: false,
            ncbi_type_material_only: false,
        }
    }
}

impl SearchAPI {
    pub fn new() -> Self {
        SearchAPI::default()
    }

    fn set_search(&mut self, s: String) {
        self.search = s;
    }

    fn set_gtdb_species_rep_only(&mut self, b: bool) {
        self.gtdb_species_rep_only = b;
    }

    fn set_ncbi_type_material_only(&mut self, b: bool) {
        self.ncbi_type_material_only = b;
    }

    pub fn from(search: String, args: &SearchArgs) -> Self {
        let mut search_api = SearchAPI::new();
        search_api.set_search(search);
        search_api.set_gtdb_species_rep_only(args.get_rep());
        search_api.set_ncbi_type_material_only(args.get_type_material());

        search_api
    }

    pub fn request(self) -> String {
        let mut url = String::from("https://api.gtdb.ecogenomic.org/search/gtdb?");

        if !self.search.is_empty() {
            url += &format!("search={}", self.search);
        }

        if self.page != 0 {
            url += &format!("&page={}", self.page);
        }

        if self.items_per_page != 0 {
            url += &format!("&itemsPerPage={}", self.items_per_page);
        }

        if !self.sort_by.is_empty() {
            url += &format!("&sortBy={}", self.sort_by);
        }

        if !self.sort_desc.is_empty() {
            url += &format!("&sortDesc={}", self.sort_desc);
        }

        if !self.search_field.is_empty() {
            url += &format!("&searchField={}", self.search_field);
        }

        if !self.filter_text.is_empty() {
            url += &format!("&filterText={}", self.filter_text);
        }

        if self.gtdb_species_rep_only {
            url += "&gtdbSpeciesRepOnly=true";
        }

        if self.ncbi_type_material_only {
            url += "&ncbiTypeMaterialOnly=true";
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let args = SearchArgs::new();
        let search = SearchAPI::from("test".to_string(), &args);
        assert_eq!(search.search, "test");
        assert_eq!(search.page, 1);
        assert_eq!(search.items_per_page, 1_000_000_000);
        assert_eq!(search.sort_by, "");
        assert_eq!(search.sort_desc, "");
        assert_eq!(search.search_field, "all");
        assert_eq!(search.filter_text, "");
        assert!(!search.gtdb_species_rep_only);
        assert!(!search.ncbi_type_material_only);
    }

    #[test]
    fn test_search_api_default() {
        let api = SearchAPI::default();
        assert_eq!(api.search, "");
        assert_eq!(api.page, 1);
        assert_eq!(api.items_per_page, 1_000_000_000);
        assert_eq!(api.sort_by, "");
        assert_eq!(api.sort_desc, "");
        assert_eq!(api.search_field, "all");
        assert_eq!(api.filter_text, "");
        assert_eq!(api.gtdb_species_rep_only, false);
        assert_eq!(api.ncbi_type_material_only, false);
    }

    #[test]
    fn test_set_search() {
        let mut api = SearchAPI::new();
        api.set_search(String::from("test"));
        assert_eq!(api.search, String::from("test"));
    }

    #[test]
    fn test_set_gtdb_species_rep_only() {
        let mut api = SearchAPI::new();
        api.set_gtdb_species_rep_only(true);
        assert_eq!(api.gtdb_species_rep_only, true);
    }

    #[test]
    fn test_set_ncbi_type_material_only() {
        let mut api = SearchAPI::new();
        api.set_ncbi_type_material_only(true);
        assert_eq!(api.ncbi_type_material_only, true);
    }

    #[test]
    fn test_from() {
        let args = SearchArgs::new();
        let api = SearchAPI::from(String::from("test"), &args);
        assert_eq!(api.search, String::from("test"));
        assert_eq!(api.gtdb_species_rep_only, false);
        assert_eq!(api.ncbi_type_material_only, false);
    }

    #[test]
    fn test_request() {
        let search = SearchAPI {
            search: "test".to_string(),
            page: 2,
            items_per_page: 20,
            sort_by: "name".to_string(),
            sort_desc: "false".to_string(),
            search_field: "all".to_string(),
            filter_text: "example".to_string(),
            gtdb_species_rep_only: true,
            ncbi_type_material_only: false,
        };
        let expected = "https://api.gtdb.ecogenomic.org/search/gtdb?search=test&page=2&itemsPerPage=20&sortBy=name&sortDesc=false&searchField=all&filterText=example&gtdbSpeciesRepOnly=true";
        assert_eq!(search.request(), expected);
    }
}
