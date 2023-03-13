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
                .get("page")
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
                    .get("gtdb_species_rep_only")
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
        let sort_by = if self.sort_by == *"" {
            String::from("")
        } else {
            format!("&sortBy={}", self.sort_by)
        };

        let sort_desc = if self.sort_desc == *"" {
            String::from("")
        } else {
            format!("&sortDesc={}", self.sort_desc)
        };

        let filter_text = if self.filter_text == *"" {
            String::from("")
        } else {
            format!("&filterText={}", self.filter_text)
        };

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
    }
}

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
pub struct GenomeApi {
    accession: String,
}

impl GenomeApi {
    pub fn from(accession: String) -> Self {
        GenomeApi { accession }
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
mod tests {
    use super::*;

    use std::collections::HashMap;

    #[test]
    fn test_new() {
        let options: HashMap<String, String> = [
            ("items_per_page".to_string(), "20".to_string()),
            ("sort_by".to_string(), "name".to_string()),
            ("sort_desc".to_string(), "false".to_string()),
            ("search_field".to_string(), "all".to_string()),
            ("filter_text".to_string(), "example".to_string()),
            ("gtdb_species_rep_only".to_string(), "true".to_string()),
            ("ncbi_type_material_only".to_string(), "false".to_string()),
        ]
        .iter()
        .cloned()
        .collect();
        let search = Search::new("test".to_string(), options);
        assert_eq!(search.search, "test");
        assert_eq!(search.page, 1);
        assert_eq!(search.items_per_page, 20);
        assert_eq!(search.sort_by, "name");
        assert_eq!(search.sort_desc, "false");
        assert_eq!(search.search_field, "all");
        assert_eq!(search.filter_text, "example");
        assert_eq!(search.gtdb_species_rep_only, true);
        assert_eq!(search.ncbi_type_material_only, false);
    }

    #[test]
    fn test_request() {
        let search = Search {
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
        let expected = "https://api.gtdb.ecogenomic.org/search/gtdb?search=test&page=2&itemsPerPage=20&sortBy=name&sortDesc=false&searchField=all&filterText=example&gtdbSpeciesRepOnly=true&ncbiTypeMaterialOnly=false";
        assert_eq!(search.request(), expected);
    }

    #[test]
    fn test_genome_request() {
        let genome_api = GenomeApi::from(String::from("accession"));

        let metadata_request = genome_api.request(GenomeRequestType::Metadata);
        let expected_metadata_request =
            String::from("https://api.gtdb.ecogenomic.org/genome/accession/metadata");

        assert_eq!(metadata_request, expected_metadata_request);

        let taxon_history_request = genome_api.request(GenomeRequestType::TaxonHistory);
        let expected_taxon_history_request =
            String::from("https://api.gtdb.ecogenomic.org/genome/accession/taxon-history");

        assert_eq!(taxon_history_request, expected_taxon_history_request);

        let card_request = genome_api.request(GenomeRequestType::Card);
        let expected_card_request =
            String::from("https://api.gtdb.ecogenomic.org/genome/accession/card");

        assert_eq!(card_request, expected_card_request);
    }
}
