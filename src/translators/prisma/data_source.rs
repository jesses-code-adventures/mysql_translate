use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Datasource {
    name: String,
    provider: String,
}

impl Datasource {
    pub fn new() -> Datasource {
        Datasource {
            name: String::from("db"),
            provider: String::from("mysql"),
        }
    }
    pub fn as_text(&self) -> String {
        format!(
            "datasource {} {{\n  provider = \"{}\"\n  url      = env(\"DATABASE_URL\")\n}}",
            self.name, self.provider
        )
    }
    pub fn parse_from_disk(datasource_string: &String) -> Datasource {
        let lines: Vec<&str> = datasource_string.split("\n").collect();
        let mut name = String::new();
        let mut provider = String::new();
        for line in lines {
            if line.contains("datasource") {
                name.push_str(
                    line.strip_prefix("datasource ")
                        .unwrap()
                        .strip_suffix(" {")
                        .unwrap(),
                )
            }
            if line.contains("provider") {
                let pieces: Vec<&str> = line.split("=").collect();
                provider = pieces[1].replace('"', "").replace(' ', "");
            }
        }
        Datasource { name, provider }
    }
}
