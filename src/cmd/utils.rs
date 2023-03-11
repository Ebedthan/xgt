use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchArgs {
    needle: String,
    level: String,
    id: bool,
    partial: bool,
    count: bool,
    raw: bool,
    rep: bool,
    type_material: bool,
    out: String,
}

impl SearchArgs {
    fn new() -> Self {
        SearchArgs {
            needle: String::new(),
            level: String::new(),
            id: false,
            partial: false,
            count: false,
            raw: false,
            rep: false,
            type_material: false,
            out: String::new(),
        }
    }

    pub fn get_needle(&self) -> String {
        self.needle.clone()
    }

    fn set_needle(&mut self, needle: &String) {
        self.needle = needle.to_string();
    }

    pub fn get_level(&self) -> String {
        self.level.clone()
    }

    fn set_level(&mut self, level: &String) {
        self.level = level.to_string();
    }

    pub fn get_gid(&self) -> bool {
        self.id
    }

    fn set_gid(&mut self, gvalue: &String) {
        if gvalue == "false" {
            self.id = false;
        } else {
            self.id = true
        }
    }

    pub fn get_partial(&self) -> bool {
        self.partial
    }

    fn set_partial(&mut self, pvalue: &String) {
        if pvalue == "false" {
            self.partial = false;
        } else {
            self.partial = true
        }
    }

    pub fn get_count(&self) -> bool {
        self.count
    }

    fn set_count(&mut self, cvalue: &String) {
        if cvalue == "false" {
            self.count = false;
        } else {
            self.count = true
        }
    }

    pub fn get_raw(&self) -> bool {
        self.raw
    }

    fn set_raw(&mut self, rvalue: &String) {
        if rvalue == "false" {
            self.raw = false;
        } else {
            self.raw = true
        }
    }

    pub fn get_type_material(&self) -> bool {
        self.type_material
    }

    fn set_type_material(&mut self, tvalue: &String) {
        if tvalue == "false" {
            self.type_material = false;
        } else {
            self.type_material = true
        }
    }

    pub fn get_rep(&self) -> bool {
        self.rep
    }

    fn set_rep(&mut self, rvalue: &String) {
        if rvalue == "false" {
            self.rep = false;
        } else {
            self.rep = true;
        }
    }

    pub fn get_out(&self) -> String {
        self.out.clone()
    }

    fn set_out(&mut self, file: &String) {
        self.out = file.to_string();
    }

    pub fn from(args: Vec<(&str, &String)>) -> Self {
        let mut new_args = SearchArgs::new();

        for arg in args {
            if arg.0 == "needle" {
                new_args.set_needle(arg.1);
            } else if arg.0 == "level" {
                new_args.set_level(arg.1);
            } else if arg.0 == "id" {
                new_args.set_gid(arg.1);
            } else if arg.0 == "partial" {
                new_args.set_partial(arg.1);
            } else if arg.0 == "count" {
                new_args.set_count(arg.1);
            } else if arg.0 == "raw" {
                new_args.set_raw(arg.1);
            } else if arg.0 == "rep" {
                new_args.set_rep(arg.1);
            } else if arg.0 == "type_material" {
                new_args.set_type_material(arg.1);
            } else {
                new_args.set_out(arg.1);
            }
        }

        new_args
    }
}

pub fn bool_as_string(b: bool) -> String {
    if b {
        String::from("true")
    } else {
        String::from("false")
    }
}

pub fn parse_gtdb<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or("null".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_needle() {
        let args = SearchArgs::new();
        assert_eq!(args.get_needle(), "");
    }

    #[test]
    fn test_set_needle() {
        let mut args = SearchArgs::new();
        args.set_needle(&String::from("test"));
        assert_eq!(args.get_needle(), "test");
    }

    #[test]
    fn test_get_level() {
        let args = SearchArgs::new();
        assert_eq!(args.get_level(), "");
    }

    #[test]
    fn test_set_level() {
        let mut args = SearchArgs::new();
        args.set_level(&String::from("level1"));
        assert_eq!(args.get_level(), "level1");
    }

    #[test]
    fn test_get_gid() {
        let args = SearchArgs::new();
        assert_eq!(args.get_gid(), false);
    }

    #[test]
    fn test_set_gid() {
        let mut args = SearchArgs::new();
        args.set_gid(&String::from("true"));
        assert_eq!(args.get_gid(), true);
    }

    #[test]
    fn test_get_partial() {
        let args = SearchArgs::new();
        assert_eq!(args.get_partial(), false);
    }

    #[test]
    fn test_set_partial() {
        let mut args = SearchArgs::new();
        args.set_partial(&String::from("true"));
        assert_eq!(args.get_partial(), true);
    }

    #[test]
    fn test_get_count() {
        let args = SearchArgs::new();
        assert_eq!(args.get_count(), false);
    }

    #[test]
    fn test_set_count() {
        let mut args = SearchArgs::new();
        args.set_count(&String::from("true"));
        assert_eq!(args.get_count(), true);
    }

    #[test]
    fn test_get_raw() {
        let args = SearchArgs::new();
        assert_eq!(args.get_raw(), false);
    }

    #[test]
    fn test_set_raw() {
        let mut args = SearchArgs::new();
        args.set_raw(&String::from("true"));
        assert_eq!(args.get_raw(), true);
    }

    #[test]
    fn test_get_type_material() {
        let args = SearchArgs::new();
        assert_eq!(args.get_type_material(), false);
    }

    #[test]
    fn test_set_type_material() {
        let mut args = SearchArgs::new();
        args.set_type_material(&String::from("true"));
        assert_eq!(args.get_type_material(), true);
    }

    #[test]
    fn test_get_rep() {
        let args = SearchArgs::new();
        assert_eq!(args.get_rep(), false);
    }

    #[test]
    fn test_set_rep() {
        let mut args = SearchArgs::new();
        args.set_rep(&String::from("true"));
        assert_eq!(args.get_rep(), true);
    }

    #[test]
    fn test_from() {
        let t = String::from("test");
        let l = String::from("level1");
        let tt = String::from("true");
        let ff = String::from("false");
        let aa = String::from("file");
        let args = vec![
            ("needle", &t),
            ("level", &l),
            ("id", &tt),
            ("partial", &ff),
            ("count", &tt),
            ("raw", &ff),
            ("rep", &tt),
            ("type_material", &ff),
            ("out", &aa),
        ];
        let search_args = SearchArgs::from(args);
        assert_eq!(
            search_args,
            SearchArgs {
                needle: t,
                level: l,
                id: true,
                partial: false,
                count: true,
                raw: false,
                rep: true,
                type_material: false,
                out: aa
            }
        )
    }

    #[test]
    fn test_new_search_args() {
        let args = SearchArgs::new();
        assert_eq!(args.get_needle(), "");
        assert_eq!(args.get_level(), "");
        assert_eq!(args.get_gid(), false);
        assert_eq!(args.get_partial(), false);
        assert_eq!(args.get_count(), false);
        assert_eq!(args.get_raw(), false);
        assert_eq!(args.get_rep(), false);
        assert_eq!(args.get_type_material(), false);
    }

    #[test]
    fn test_set_search_args() {
        let mut args = SearchArgs::new();
        args.set_needle(&String::from("test"));
        args.set_level(&String::from("1"));
        args.set_gid(&String::from("false"));
        args.set_partial(&String::from("true"));
        args.set_count(&String::from("true"));
        args.set_raw(&String::from("false"));
        args.set_rep(&String::from("true"));
        args.set_type_material(&String::from("false"));

        assert_eq!(args.get_needle(), "test");
        assert_eq!(args.get_level(), "1");
        assert_eq!(args.get_gid(), false);
        assert_eq!(args.get_partial(), true);
        assert_eq!(args.get_count(), true);
        assert_eq!(args.get_raw(), false);
        assert_eq!(args.get_rep(), true);
        assert_eq!(args.get_type_material(), false);
    }

    #[test]
    fn test_bool_as_string() {
        assert_eq!(bool_as_string(true), "true");
        assert_eq!(bool_as_string(false), "false");
    }
}
