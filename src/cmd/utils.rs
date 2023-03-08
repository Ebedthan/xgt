#[derive(Debug, Clone)]
pub struct SearchArgs {
    needle: String,
    level: String,
    gid: bool,
    partial: bool,
    count: bool,
    raw: bool,
    rep: bool,
    type_material: bool,
}

impl SearchArgs {
    fn new() -> Self {
        SearchArgs {
            needle: String::new(),
            level: String::new(),
            gid: false,
            partial: false,
            count: false,
            raw: false,
            rep: false,
            type_material: false,
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
        self.gid
    }

    fn set_gid(&mut self, gvalue: &String) {
        if gvalue == "false" {
            self.gid = false;
        } else {
            self.gid = true
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
            self.rep = true
        }
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
            } else {
                new_args.set_type_material(arg.1);
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
