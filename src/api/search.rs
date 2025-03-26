use crate::cli::search::SearchArgs;

#[derive(Debug, Clone)]
pub struct SearchAPI {
    /// The search query entered by the user
    search: String,

    /// The current page number for paginated results
    page: u16,

    /// The number of items displayed per page
    items_per_page: u32,

    /// The field by which results should be sorted
    sort_by: String,

    /// Sorting order: expected to be "true" for descending, "false" for ascending
    sort_desc: bool,

    /// The specific field to search within
    search_field: String,

    /// Additional text-based filters applied to the search
    filter_text: String,

    /// Whether to include only GTDB species representative genomes
    gtdb_species_rep_only: bool,

    /// Whether to include only NCBI type material
    ncbi_type_material_only: bool,

    /// Output format, e.g., "json", "csv" or "tsv"
    output_format: String,
}

impl Default for SearchAPI {
    fn default() -> Self {
        SearchAPI {
            search: String::new(),
            page: 1,
            items_per_page: 1_000,
            sort_by: String::new(),
            sort_desc: false,
            search_field: "all".to_string(),
            filter_text: String::new(),
            gtdb_species_rep_only: false,
            ncbi_type_material_only: false,
            output_format: "csv".to_string(),
        }
    }
}

impl SearchAPI {
    pub fn new() -> Self {
        SearchAPI::default()
    }

    fn set_search(mut self, s: String) -> Self {
        self.search = s;
        self
    }

    fn set_search_field(mut self, field: String) -> Self {
        self.search_field = field;
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
        self.output_format = outfmt.to_string();
        self
    }

    pub fn from(search: &str, args: &SearchArgs) -> Self {
        SearchAPI::new()
            .set_search(search.to_owned())
            .set_gtdb_species_rep_only(args.is_representative_species_only())
            .set_ncbi_type_material_only(args.is_type_species_only())
            .set_outfmt(&args.get_outfmt().to_string())
            .set_search_field(args.get_search_field().to_string())
    }

    pub fn request(&self) -> String {
        let url = format!(
            "https://api.gtdb.ecogenomic.org/search/gtdb{}?",
            if self.output_format == "json" {
                String::from("")
            } else {
                format!("/{}", self.output_format)
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

        if self.sort_desc {
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
        assert_eq!(api.items_per_page, 1_000);
        assert_eq!(api.sort_by, "");
        assert_eq!(api.sort_desc, false);
        assert_eq!(api.search_field, "all");
        assert_eq!(api.filter_text, "");
        assert_eq!(api.gtdb_species_rep_only, false);
        assert_eq!(api.ncbi_type_material_only, false);
        assert_eq!(api.output_format, "csv");
    }

    #[test]
    fn test_builder_pattern() {
        let api = SearchAPI::new()
            .set_search("test_search".to_string())
            .set_gtdb_species_rep_only(true)
            .set_ncbi_type_material_only(true)
            .set_outfmt("json");

        assert_eq!(api.search, "test_search");
        assert_eq!(api.page, 1);
        assert_eq!(api.items_per_page, 1_000);
        assert_eq!(api.sort_by, "");
        assert_eq!(api.sort_desc, false);
        assert_eq!(api.search_field, "all");
        assert_eq!(api.filter_text, "");
        assert_eq!(api.gtdb_species_rep_only, true);
        assert_eq!(api.ncbi_type_material_only, true);
        assert_eq!(api.output_format, "json");
    }

    #[test]
    fn test_search_api_request() {
        let api = SearchAPI::new()
            .set_search("test_search".to_string())
            .set_gtdb_species_rep_only(true)
            .set_ncbi_type_material_only(true)
            .set_outfmt("json");

        let expected_url = "https://api.gtdb.ecogenomic.org/search/gtdb?search=test_search&page=1&itemsPerPage=1000&searchField=all&gtdbSpeciesRepOnly=true&ncbiTypeMaterialOnly=true";
        assert_eq!(api.request(), expected_url);
    }

    #[test]
    fn test_search_api_request_default() {
        let api = SearchAPI::default();
        let expected_url = "https://api.gtdb.ecogenomic.org/search/gtdb/csv?page=1&itemsPerPage=1000&searchField=all";
        assert_eq!(api.request(), expected_url);
    }
}
