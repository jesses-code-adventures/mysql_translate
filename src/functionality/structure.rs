use crate::functionality::session::Session;
use crate::remotes::sql::Table;
use anyhow::Result;
use core::fmt::{self, Display};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json;
use std::cell::RefCell;
use std::env;

pub fn set_vars() {
    dotenv().expect(".env not found");
}

pub fn get_session_data_location() -> String {
    set_vars();
    let mut data_location =
        env::var("STORAGE").expect("storage directory to exist as an environment variable");
    data_location.push_str("/session.json");
    data_location
}

/// A trait for translating between a database of Vec<Table> and the
/// format implemented by the child struct.
pub trait TranslatorBehaviour<T> {
    /// Writes the database output straight to the disk in the desired format.
    fn write_to_disk(&self, descriptions: &Vec<Table>);
    /// Gets type T from a Vec of Table.
    fn get_translation(&self, descriptions: &Vec<Table>) -> T;
    /// Reads the database descriptions from the database in the desired format.
    fn load_from_database(&mut self, descriptions: &Vec<Table>);
    /// Reads the database description from disk in the desired format.
    fn load_from_disk(&mut self) -> Result<()>;
    /// Pretty print the translator's output.
    fn get_string(&self) -> String;
}

pub trait UI {
    fn new(session: RefCell<Session>) -> Self;

    fn get_session(&self) -> std::cell::Ref<Session>;

    fn get_session_mut(&self) -> std::cell::RefMut<Session>;

    /// Returns the index of the selected database in self.databases
    fn select_database(&self) -> Result<usize>;

    /// Should allow the user to create a new database and reload the state so that it's
    /// immediately available
    fn create_database_entry(&mut self) -> Result<()>;

    /// Should allow the user to select a schema to write to disk from the selected remote database
    fn select_schema_to_write(&mut self) -> Result<()>;

    /// Allow the user to select a database to pull from existing local schemas and parse/output
    /// the schemas without writing to disk
    fn view_tables_from_disk(&self) -> Result<()>;

    /// Allow the user to select a database to pull from remote and parse/output
    /// the schemas without writing to disk
    fn view_tables_from_database(&self) -> Result<()>;

    /// Choose a database to edit the settings (name, url) for
    fn edit_databases(&mut self) -> Result<()>;

    /// Main entry point which should allow the user to access:
    /// self.select_database, self.create_database_entry, self.view_tables_from_disk,
    /// self.view_tables_from_database, self.edit_databases
    fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct DiskMapping {
    pub format: AcceptedFormat,
    // #[serde(borrow)]
    pub path: String,
}

impl DiskMapping {
    pub fn from_json(json: serde_json::Value) -> Result<Vec<DiskMapping>, serde_json::Error> {
        // Temporary deserialization struct
        #[derive(Deserialize)]
        struct TempMapping {
            format: AcceptedFormat,
            path: String,
        }

        let temp_mappings: Vec<TempMapping> = serde_json::from_value(json)?;
        let disk_mappings: Vec<DiskMapping> = temp_mappings
            .into_iter()
            .map(|temp_mapping| DiskMapping {
                format: temp_mapping.format,
                path: temp_mapping.path,
            })
            .collect();

        Ok(disk_mappings)
    }
}

#[derive(Serialize, PartialEq, Deserialize, Copy, Clone, Debug)]
pub enum AcceptedFormat {
    Json,
    Prisma,
}

impl AcceptedFormat {
    pub fn from_string(format: &str) -> AcceptedFormat {
        match format {
            "json" => AcceptedFormat::Json,
            "prisma" => AcceptedFormat::Prisma,
            _ => panic!("Invalid format"),
        }
    }
    pub fn as_string(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Prisma => "prisma",
        }
    }
    pub fn all_as_array() -> Vec<AcceptedFormat> {
        vec![AcceptedFormat::Json, AcceptedFormat::Prisma]
    }
    pub fn all_as_string_array() -> Vec<String> {
        vec![
            AcceptedFormat::Json.as_string().to_string(),
            AcceptedFormat::Prisma.as_string().to_string(),
        ]
    }
}

impl Display for AcceptedFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.as_string())
    }
}
