use crate::functionality::structure::{AcceptedFormat, DiskMapping};
use crate::remotes::sql;
use crate::translators::{
    behaviour::TranslatorBehaviour, json_translator::JsonTranslator,
    prisma_translator::PrismaTranslator,
};
use anyhow::Result;
use serde::Serialize;
use serde_json::json;

/// One database url can be linked up to multiple schema locations.
/// The "name" does not need to match the db name.
#[derive(Serialize, Clone)]
pub struct Database {
    pub name: String,
    pub db_url: String,
    pub disk_mappings: Vec<DiskMapping>,
}

impl Database {
    /// Pull the database info from the db and propagate it.
    pub fn sync(&self) -> Result<()> {
        let descriptions = self.get_descriptions();
        for mapping in self.disk_mappings.iter() {
            self.sync_one(mapping.format, mapping.path.to_owned(), &descriptions)?;
        }
        Ok(())
    }
    /// Sync one database schema
    pub fn sync_one(
        &self,
        format: AcceptedFormat,
        path: String,
        descriptions: &Vec<sql::Table>,
    ) -> Result<()> {
        match format {
            AcceptedFormat::Json => {
                let translator = JsonTranslator { path, json: None };
                translator.write_to_disk(&descriptions)?;
            }
            AcceptedFormat::Prisma => {
                let translator = PrismaTranslator {
                    path,
                    disk_schema: None,
                    db_schema: None,
                };
                translator.write_to_disk(&descriptions)?;
            }
        }
        Ok(())
    }

    /// Update the name associated with your database.
    /// This is not the name of the database itself.
    pub fn update_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    /// Update the database url.
    pub fn update_db_url(&mut self, new_db_url: String) {
        self.db_url = new_db_url;
    }

    /// Push a new disk mapping to the database. Does not save to disk.
    pub fn create_disk_mapping(&mut self, format: AcceptedFormat, path: String) {
        let mapping = DiskMapping { format, path };
        self.disk_mappings.push(mapping)
    }

    /// Map a local path to an accepted format, or create a new mapping if none exists.
    pub fn update_disk_mapping(&mut self, format: AcceptedFormat, path: String) {
        if self.disk_mappings.len() == 0 {
            self.create_disk_mapping(format, path);
            return;
        }
        let mut mapping_update_index = 0;
        let mut found = false;
        for (i, disk_mapping) in self.disk_mappings.iter().enumerate() {
            if disk_mapping.format == format {
                mapping_update_index = i;
                found = true;
            }
        }
        if !found {
            self.create_disk_mapping(format, path);
            return;
        }
        self.disk_mappings[mapping_update_index].path = path;
    }

    /// Get a json value of the database.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "db_url": self.db_url,
            "disk_mappings": self.disk_mappings
        })
    }

    /// Get the database table descriptions from remote
    pub fn get_descriptions(&self) -> Vec<sql::Table> {
        sql::get_table_descriptions(&self.db_url).expect("Failed to get table descriptions")
    }
}
