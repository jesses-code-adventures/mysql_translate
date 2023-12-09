use crate::remotes::sql::{Description, Table};
use crate::translators::behaviour::TranslatorBehaviour;
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;

/// A translator for json
pub struct JsonTranslator {
    pub path: String,
    pub json: Option<serde_json::Value>,
}

/// Public implementation for JsonTranslator
impl TranslatorBehaviour<serde_json::Value> for JsonTranslator {
    /// Get json data from the database's output
    fn get_translation(&self, database: &Vec<Table>) -> serde_json::Value {
        let mut result = HashMap::new();
        for database in database {
            let this_result = json!(self.format_database(&database.description));
            result.insert(&database.name, this_result);
        }
        let mut tables = HashMap::new();
        tables.insert("tables", result);
        json!(tables)
    }

    /// Load json from a database into the translator.
    fn load_from_database(&mut self, database: &Vec<Table>) {
        self.json = Some(self.get_translation(database))
    }

    /// Load json from a path into the translator.
    fn load_from_disk(&mut self) -> Result<()> {
        let file = File::open(&self.path)?;
        let val = serde_json::from_reader(file).unwrap_or_else(|_| {
            eprintln!("couldn't read the file! skipping loading...");
            serde_json::Value::Null
        });
        if !val.is_null() {
            self.json = Some(val);
        }
        Ok(())
    }

    /// Receive data from the datbase and write straight to disk.
    fn write_to_disk(&self, database: &Vec<Table>) -> Result<()> {
        println!("writing json to {}", &self.path);
        let the_json = self.get_translation(database);
        self.dump_json(&the_json)?;
        Ok(())
    }

    /// A pretty string representation of the translator's json.
    fn get_string(&self) -> String {
        return serde_json::to_string_pretty(self.json.as_ref().unwrap()).unwrap();
    }
}

/// Private implementation behaviours for JsonTranslator
impl JsonTranslator {
    /// Formats one field description.
    fn format_table(&self, field: &Description) -> String {
        let mut result = String::new();
        result.push_str(&field.type_);
        if field.key.len() > 0 {
            result.push_str(&format!(" {}", field.key));
        }
        if field.null == "NO" {
            result.push_str(" NOT NULL");
        }
        if field.extra.contains("auto_increment") {
            result.push_str(" AUTO_INCREMENT");
        }
        result
    }

    /// Coalesces the description of a database's tables into a hashmap.
    fn format_database(&self, database: &Vec<Description>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        // println!("{:?}", database);
        for db in database {
            result.insert(String::from(&db.field), self.format_table(db));
        }
        result
    }

    /// Write json to a path.
    fn dump_json(&self, the_json: &serde_json::Value) -> Result<()> {
        let file = File::create(&self.path)?;
        let buf_writer = BufWriter::new(file);
        serde_json::ser::to_writer_pretty(buf_writer, &the_json)?;
        Ok(())
    }
}
