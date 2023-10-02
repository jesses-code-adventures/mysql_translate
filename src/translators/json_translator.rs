use crate::functionality::structure::TranslatorBehaviour;
use crate::remotes::sql::{Description, Table};
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
            result.insert(
                &database.name,
                json!(self.format_database(&database.description)),
            );
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
            println!("couldn't read the file! skipping loading...");
            serde_json::Value::Null
        });
        if !val.is_null() {
            self.json = Some(val);
        }
        Ok(())
    }

    /// Receive data from the datbase and write straight to disk.
    fn write_to_disk(&self, database: &Vec<Table>) {
        println!("writing json to {}", &self.path);
        let the_json = self.get_translation(database);
        match self.dump_json(&the_json) {
            Ok(_) => (),
            Err(e) => println!("error writing json: {}", e),
        }
    }

    /// A pretty string representation of the translator's json.
    fn get_string(&self) -> String {
        return serde_json::to_string_pretty(self.json.as_ref().unwrap()).unwrap();
    }
}

/// Private implementation behaviours for JsonTranslator
impl JsonTranslator {
    /// Formats one database table description.
    fn format_table(&self, database: &Description) -> String {
        let mut result = String::new();
        result.push_str(&database.type_);
        if database.key.len() > 0 {
            result.push_str(&format!(" {}", database.key));
        }
        if database.null == "NO" {
            result.push_str(" NOT NULL");
        }
        result
    }

    /// Coalesces the description of a database's tables into a hashmap.
    fn format_database(&self, database: &Vec<Description>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for database in database {
            result.insert(String::from(&database.field), self.format_table(database));
        }
        result
    }

    /// Write json to a path.
    fn dump_json(&self, the_json: &serde_json::Value) -> Result<(), std::io::Error> {
        let file = File::create(&self.path)?;
        let buf_writer = BufWriter::new(file);
        match serde_json::ser::to_writer_pretty(buf_writer, &the_json) {
            Ok(_) => Ok(()),
            Err(e) => Err(Into::into(e)),
        }
    }
}
