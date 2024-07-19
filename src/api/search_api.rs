use crate::cmd::utils::SearchArgs;

#[derive(Debug, Clone)]
pub struct SearchAPI {
    search: String,
    page: u16,
    items_per_page: u16,
    sort_by: String,
    sort_desc: String,
    search_field: String,
    filter_text: String,
    gtdb_species_rep_only: bool,
    ncbi_type_material_only: bool,
    outfmt: String,
}

impl Default for SearchAPI {
    fn default() -> Self {
        SearchAPI {
            search: String::new(),
            page: 1,
            items_per_page: 100,
            sort_by: String::new(),
            sort_desc: String::new(),
            search_field: "gtdb_tax".to_string(),
            filter_text: String::new(),
            gtdb_species_rep_only: false,
            ncbi_type_material_only: false,
            outfmt: "csv".to_string(),
        }
    }
}

impl SearchAPI {
    pub fn new() -> Self {
        SearchAPI::default()
    }

    fn set_search(mut self, s: &str) -> Self {
        self.search = s.to_string();
        self
    }

    pub fn set_gtdb_species_rep_only(mut self, b: bool) -> Self {
        self.gtdb_species_rep_only = b;
        self
    }

    pub fn set_ncbi_type_material_only(mut self, b: bool) -> Self {
        self.ncbi_type_material_only = b;
        self
    }

    pub fn set_outfmt(mut self, outfmt: &str) -> Self {
        self.outfmt = outfmt.to_string();
        self
    }

    pub fn from(search: &str, args: &SearchArgs) -> Self {
        SearchAPI::new()
            .set_search(search)
            .set_gtdb_species_rep_only(args.is_representative_species_only())
            .set_ncbi_type_material_only(args.is_type_species_only())
            .set_outfmt(&args.get_outfmt().to_string())
    }

    pub fn request(&self) -> String {
        let url = format!(
            "https://api.gtdb.ecogenomic.org/search/gtdb{}?",
            if self.outfmt == "json" {
                String::from("")
            } else {
                format!("/{}", self.outfmt)
            }
        );

        let mut params = vec![];

        if !self.search.is_empty() {
            params.push(format!("search={}", self.search));
        }

        if self.page != 0 {
            params.push(format!("page={}", self.page));
        }

        if self.items_per_page != 0 {
            params.push(format!("itemsPerPage={}", self.items_per_page));
        }

        if !self.sort_by.is_empty() {
            params.push(format!("sortBy={}", self.sort_by));
        }

        if !self.sort_desc.is_empty() {
            params.push(format!("sortDesc={}", self.sort_desc));
        }

        if !self.search_field.is_empty() {
            params.push(format!("searchField={}", self.search_field));
        }

        if !self.filter_text.is_empty() {
            params.push(format!("filterText={}", self.filter_text));
        }

        if self.gtdb_species_rep_only {
            params.push("gtdbSpeciesRepOnly=true".to_string());
        }

        if self.ncbi_type_material_only {
            params.push("ncbiTypeMaterialOnly=true".to_string());
        }

        url + &params.join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_search_api() {
        let api = SearchAPI::default();
        assert_eq!(api.search, "");
        assert_eq!(api.page, 1);
        assert_eq!(api.items_per_page, 100);
        assert_eq!(api.sort_by, "");
        assert_eq!(api.sort_desc, "");
        assert_eq!(api.search_field, "gtdb_tax");
        assert_eq!(api.filter_text, "");
        assert_eq!(api.gtdb_species_rep_only, false);
        assert_eq!(api.ncbi_type_material_only, false);
        assert_eq!(api.outfmt, "csv");
    }

    #[test]
    fn test_builder_pattern() {
        let api = SearchAPI::new()
            .set_search("test_search")
            .set_gtdb_species_rep_only(true)
            .set_ncbi_type_material_only(true)
            .set_outfmt("json");

        assert_eq!(api.search, "test_search");
        assert_eq!(api.page, 1);
        assert_eq!(api.items_per_page, 100);
        assert_eq!(api.sort_by, "");
        assert_eq!(api.sort_desc, "");
        assert_eq!(api.search_field, "gtdb_tax");
        assert_eq!(api.filter_text, "");
        assert_eq!(api.gtdb_species_rep_only, true);
        assert_eq!(api.ncbi_type_material_only, true);
        assert_eq!(api.outfmt, "json");
    }

    #[test]
    fn test_search_api_request() {
        let api = SearchAPI::new()
            .set_search("test_search")
            .set_gtdb_species_rep_only(true)
            .set_ncbi_type_material_only(true)
            .set_outfmt("json");

        let expected_url = "https://api.gtdb.ecogenomic.org/search/gtdb?search=test_search&page=1&itemsPerPage=100&searchField=gtdb_tax&gtdbSpeciesRepOnly=true&ncbiTypeMaterialOnly=true";
        assert_eq!(api.request(), expected_url);
    }

    #[test]
    fn test_search_api_request_default() {
        let api = SearchAPI::default();
        let expected_url = "https://api.gtdb.ecogenomic.org/search/gtdb/csv?page=1&itemsPerPage=100&searchField=gtdb_tax";
        assert_eq!(api.request(), expected_url);
    }
}
