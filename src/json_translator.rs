use crate::sql::{Description, Table};
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn format_description(description: &Description) -> String {
    let mut result = String::new();
    result.push_str(&description.type_);
    if description.key.len() > 0 {
        result.push_str(&format!(" {}", description.key));
    }
    if description.null == "NO" {
        result.push_str(" NOT NULL");
    }
    result
}

fn format_descriptions(descriptions: &Vec<Description>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for description in descriptions {
        result.insert(
            String::from(&description.field),
            format_description(description),
        );
    }
    result
}

fn get_json(descriptions: &Vec<Table>) -> serde_json::Value {
    let mut result = HashMap::new();
    for description in descriptions {
        result.insert(
            &description.name,
            json!(format_descriptions(&description.description)),
        );
    }
    let mut tables = HashMap::new();
    tables.insert("tables", result);
    json!(tables)
}

fn dump_json(the_json: &serde_json::Value, full_path: &str) {
    let path = Path::new(full_path);
    let file = File::create(&path).unwrap();
    let buf_writer = BufWriter::new(file);
    serde_json::ser::to_writer_pretty(buf_writer, &the_json).unwrap();
}

/// Converts a vector of sql::Table to a json object and dumps to disk.
pub fn write(descriptions: &Vec<Table>, full_write_path: &str) {
    println!("writing json to {}", full_write_path);
    let the_json = get_json(descriptions);
    dump_json(&the_json, full_write_path);
    println!("done");
}
