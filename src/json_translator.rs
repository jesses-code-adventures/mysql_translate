use crate::sql::{Description, Table};
use crate::structure::TranslatorBehaviour;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// A translator for json
pub struct JsonTranslator<'a> {
    // pub disk_mapping: &'a DiskMapping<'a>,
    pub path: &'a Path,
}

/// Public implementation for JsonTranslator
impl<'a> TranslatorBehaviour<serde_json::Value> for JsonTranslator<'a> {
    /// Converts a vector of sql::Table to a json object
    fn from_database(&self, database: &Vec<Table>) -> serde_json::Value {
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

    /// Load json from a path.
    fn from_disk(&self) -> Result<serde_json::Value, std::io::Error> {
        let file = File::open(&self.path)?;
        let val = serde_json::from_reader(file).unwrap_or_else(|_| {
            println!("couldn't read the file! loading an empty one...");
            json!({})
        });
        Ok(val)
    }

    /// Converts a vector of sql::Table to a json object and dumps to disk.
    fn to_disk(&self, database: &Vec<Table>) {
        println!("writing json to {:?}", &self.path.to_str());
        let the_json = self.from_database(database);
        match self.dump_json(&the_json) {
            Ok(_) => (),
            Err(e) => println!("error writing json: {}", e),
        }
    }

    fn display(&self, value: serde_json::Value) {
        todo!();
    }
}

/// Private implementation behaviours for JsonTranslator
impl<'a> JsonTranslator<'a> {
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
        let file = File::create(self.path)?;
        let buf_writer = BufWriter::new(file);
        match serde_json::ser::to_writer_pretty(buf_writer, &the_json) {
            Ok(_) => Ok(()),
            Err(e) => Err(Into::into(e)),
        }
    }
}
