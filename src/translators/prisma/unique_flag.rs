use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
pub struct UniqueFlag {
    pub map: Option<String>,
}

impl UniqueFlag {
    pub fn as_text(self) -> String {
        let mut resp = String::new();
        resp.push_str("@unique");
        if self.map.is_some() {
            resp.push_str("(map: \"");
            resp.push_str(self.map.clone().unwrap().as_str());
            resp.push_str("\")");
        }
        resp
    }
}

impl PartialEq for UniqueFlag {
    fn eq(&self, other: &Self) -> bool {
        let resp = self.map == other.map;
        if !resp {
            println!("in unique flag");
            println!("---> {:?} != {:?}", self.map, other.map);
        }
        self.map == other.map
    }
}

impl From<&str> for UniqueFlag {
    fn from(string: &str) -> Self {
        let mut new_string = string.clone();

        if new_string.starts_with("@") {
            new_string = new_string
                .strip_prefix("@")
                .expect("input to be a unique string");
        }

        new_string = new_string
            .strip_prefix("unique")
            .expect("input to be a unique string");

        if new_string.len() == 0 {
            return UniqueFlag { map: None };
        }

        new_string = new_string
            .trim()
            .strip_prefix("(")
            .expect("unique string to start with an opening paren")
            .strip_suffix(")")
            .expect("unique string to end in a closing paren");

        new_string = new_string
            .strip_prefix("map: \"")
            .expect("map to be formatted correctly")
            .strip_suffix("\"")
            .expect("closing double quotes to exist");

        UniqueFlag {
            map: Some(new_string.to_string()),
        }
    }
}
