use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Search {
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

impl Search {
    pub fn new(search: String, options: HashMap<String, String>) -> Self {
        Search {
            search,
            page: options
                .get("items_per_page")
                .unwrap_or(&String::from("1"))
                .parse::<i32>()
                .unwrap(),
            items_per_page: options
                .get("items_per_page")
                .unwrap_or(&String::from("100"))
                .parse::<i32>()
                .unwrap(),
            sort_by: String::from(options.get("sort_by").unwrap_or(&String::from(""))),
            sort_desc: String::from(options.get("sort_desc").unwrap_or(&String::from(""))),
            search_field: String::from(options.get("search_field").unwrap_or(&String::from("all"))),
            filter_text: String::from(options.get("filter_text").unwrap_or(&String::from(""))),
            gtdb_species_rep_only: matches!(
                options
                    .get("filter_text")
                    .unwrap_or(&String::from("false"))
                    .as_str(),
                "true"
            ),
            ncbi_type_material_only: matches!(
                options
                    .get("ncbi_type_material_only")
                    .unwrap_or(&String::from("false"))
                    .as_str(),
                "true"
            ),
        }
    }

    pub fn request(self) -> String {
        let sort_by = if self.sort_by == "".to_string() {
            String::from("")
        } else {
            format!("&sortBy={}", self.sort_by)
        };

        let sort_desc = if self.sort_desc == "".to_string() {
            String::from("")
        } else {
            format!("&sortDesc={}", self.sort_desc)
        };

        let filter_text = if self.filter_text == "".to_string() {
            String::from("")
        } else {
            format!("&filterText={}", self.filter_text)
        };

        String::from(
            format!("https://api.gtdb.ecogenomic.org/search/gtdb?search={}&page={}&itemsPerPage={}{}{}&searchField={}{}&gtdbSpeciesRepOnly={}&ncbiTypeMaterialOnly={}", 
            self.search,
            self.page,
            self.items_per_page,
            sort_by,
            sort_desc,
            self.search_field,
            filter_text,
            self.gtdb_species_rep_only,
            self.ncbi_type_material_only)
        )
    }
}
