use crate::functionality::structure::{AcceptedFormat, DiskMapping};
use crate::remotes::sql;
use crate::translators::{
    behaviour::TranslatorBehaviour, json_translator::JsonTranslator,
    prisma_translator::PrismaTranslator,
};
use dialoguer::Select;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;

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
    pub fn sync(&self) {
        let descriptions = self.get_descriptions();
        for mapping in self.disk_mappings.iter() {
            self.sync_one(mapping.format, mapping.path.to_owned(), &descriptions);
        }
    }
    /// Sync one database schema
    pub fn sync_one(&self, format: AcceptedFormat, path: String, descriptions: &Vec<sql::Table>) {
        match format {
            AcceptedFormat::Json => {
                let translator = JsonTranslator { path, json: None };
                translator.write_to_disk(&descriptions)
            }
            AcceptedFormat::Prisma => {
                let translator = PrismaTranslator {
                    path,
                    disk_schema: None,
                    db_schema: None,
                };
                translator.write_to_disk(&descriptions)
            }
        }
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
    /// TODO - move the UI parts to the UI
    // fn select_format(&self) -> AcceptedFormat {
    //     let mut formats = AcceptedFormat::all_as_array();
    //     let mut options = vec![];
    //     for fmt in formats.iter_mut() {
    //         options.push(fmt.as_string());
    //     }
    //     options.push("exit");
    //     let selection = Select::new().items(&options).default(0).interact().unwrap();
    //     match options[selection] {
    //         "json" => return AcceptedFormat::Json,
    //         "prisma" => return AcceptedFormat::Prisma,
    //         "exit" => {
    //             println!("defaulting to json...");
    //             return AcceptedFormat::Json;
    //         }
    //         _ => {
    //             println!("defaulting to json...");
    //             return AcceptedFormat::Json;
    //         }
    //     }
    // }

    /// Push a new disk mapping to the database. Does not save to disk.
    pub fn create_disk_mapping(&mut self, format: AcceptedFormat, path: String) {
        let mapping = DiskMapping { format, path };
        self.disk_mappings.push(mapping)
    }

    /// Edit an existing mapping
    pub fn update_disk_mapping(&mut self, format: AcceptedFormat, path: String) {
        if self.disk_mappings.len() == 0 {
            println!("no saved disk mappings, creating a new one...");
            self.create_disk_mapping(format, path);
            return;
        }
        let mut mapping_update_index = 0;
        let mut found = false;
        for (i, disk_mapping) in self.disk_mappings.iter().enumerate() {
            println!("disk mapping: {:?}", disk_mapping);
            if disk_mapping.format == format {
                mapping_update_index = i;
                found = true;
            }
        }
        if !found {
            println!("no mapping found, creating a new one");
            self.create_disk_mapping(format, path);
            return;
        }
        println!(
            "selected {:?}, please enter the full path",
            self.disk_mappings[mapping_update_index].format
        );
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("failed to read the input.");
        let new_path_buf: PathBuf = PathBuf::from(input.trim().to_string());
        self.disk_mappings[mapping_update_index].path =
            new_path_buf.to_str().expect("path to exist").to_string();
    }

    /// Print a representation of the database
    pub fn display(&self) {
        println!("name: {}", self.name);
        println!("db_url: {}", self.db_url);
        for mapping in self.disk_mappings.iter() {
            match mapping.format {
                AcceptedFormat::Json => println!("json_path: {}", mapping.path),
                AcceptedFormat::Prisma => println!("prisma_path: {}", mapping.path),
            }
        }
    }

    /// Get a json value of the database.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "db_url": self.db_url,
            "disk_mappings": self.disk_mappings
        })
    }

    /// User interactivity for editing a database
    /// Returns true if the user wants to edit another database.
    pub fn edit(&mut self) -> bool {
        let options = vec!["name", "db_url", "disk_mappings", "done"];
        let mut selection = Select::new().items(&options).default(0).interact().unwrap();
        let mut edit_another = false;
        while options[selection] != "done" {
            match options[selection] {
                "name" => {
                    println!("pls enter the new name");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read line");
                    self.update_name(input.trim().to_string());
                }
                "db_url" => {
                    println!("pls enter the new db url");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read line");
                    self.update_db_url(input.trim().to_string());
                }
                "disk_mappings" => {
                    self.edit_disk_mappings();
                }
                "edit_another" => {
                    edit_another = true;
                    break;
                }
                "done" => {
                    edit_another = false;
                }
                _ => println!("invalid selection"),
            }
            selection = Select::new().items(&options).default(0).interact().unwrap();
        }
        edit_another
    }

    /// Todo - move the UI parts to the UI
    fn edit_disk_mappings(&mut self) -> bool {
        let string_options = AcceptedFormat::all_as_string_array();
        let mut options: Vec<&str> = vec![];
        for option in &string_options {
            options.push(option.as_str());
        }
        options.push("exit");

        let selection = Select::new().items(&options).default(0).interact().unwrap();
        let mut edit_another = false;
        match options[selection] {
            "prisma" => {
                println!("pls enter the full path to the prisma schema");
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("failed to read input");
                self.update_disk_mapping(AcceptedFormat::Prisma, input);
                edit_another = false;
            }
            "json" => {
                println!("pls enter the full path to the json schema");
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("failed to read input");
                self.update_disk_mapping(AcceptedFormat::Json, input);
                edit_another = false;
            }
            "exit" => {
                edit_another = false;
            }
            _ => println!("invalid selection or selection not yet implemented"),
        }
        edit_another
    }

    /// Get the database table descriptions from remote
    pub fn get_descriptions(&self) -> Vec<sql::Table> {
        println!("{:?}", &self.db_url);
        sql::get_table_descriptions(&self.db_url).expect("Failed to get table descriptions")
    }
}
