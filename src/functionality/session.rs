use crate::functionality::{
    database,
    structure::{AcceptedFormat, DiskMapping},
};
use crate::translators::{
    behaviour::TranslatorBehaviour, json_translator::JsonTranslator,
    prisma_translator::PrismaTranslator,
};
use anyhow::Result;
use serde_json;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(Clone)]
pub struct Session {
    pub databases: Vec<database::Database>,
    pub _data_location: String,
}

impl Session {
    /// If None is returned, use new_bare_session and then create a database entry from the UI
    /// consuming the session.
    pub fn new(full_path: &String) -> Option<Session> {
        let mut s = Session {
            databases: Vec::new(),
            _data_location: full_path.to_string(),
        };
        s.load().ok()?;
        if s.databases.len() == 0 {
            return None;
        }
        return Some(s);
    }

    /// Create a new session with no databases.
    pub fn new_bare_session(full_path: &String) -> Option<Session> {
        Some(Session {
            databases: Vec::new(),
            _data_location: full_path.to_string(),
        })
    }

    /// Load an existing session from the disk.
    pub fn load(&mut self) -> Result<()> {
        let path = Path::new(self._data_location.as_str());
        let file = File::open(&path).unwrap_or_else(|_| File::create(&path).unwrap());
        let reader = std::io::BufReader::new(file);
        let the_json: Vec<serde_json::Value> =
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new());
        for mut database in the_json {
            let name = database["name"].as_str().unwrap().to_string();
            let db_url = database["db_url"].as_str().unwrap().to_string();
            let disk_mappings = DiskMapping::from_json(database["disk_mappings"].take())
                .expect("disk mappings to parse successfully from json");
            let database = database::Database {
                name,
                db_url,
                disk_mappings,
            };
            self.add_database(database)
                .expect("adding database should work");
        }
        Ok(())
    }

    /// Save the session to the disk.
    pub fn add_database(&mut self, database: database::Database) -> Result<()> {
        if self
            .find_existing_database_index(&database.db_url)
            .is_some()
        {
            return Ok(());
        }
        self.databases.push(database);
        self.save()?;
        Ok(())
    }

    pub fn display(&mut self) {
        self.sort();
        for database in &self.databases {
            database.display();
        }
    }

    pub fn sync(&self) {
        for database in &self.databases {
            database.sync();
        }
    }

    fn find_existing_database_index(&self, db_url: &str) -> Option<usize> {
        let mut index = 0;
        for database in &self.databases {
            if database.db_url == db_url {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    // fn remove_database(&mut self, index: usize) -> Result<()> {
    //     self.databases.remove(index);
    //     self.save()?;
    //     Ok(())
    // }

    fn sort(&mut self) {
        self.databases.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Write the session's databases to the disk.
    /// This does not write the schemas themselves, instead it saves the database names and urls.
    pub fn save(&self) -> Result<(), serde_json::Error> {
        let path = Path::new(self._data_location.as_str());
        let file = File::create(&path).unwrap();
        let buf_writer = BufWriter::new(file);
        let the_json = self
            .databases
            .iter()
            .map(|database| database.to_json())
            .collect::<Vec<_>>();
        serde_json::to_writer_pretty(buf_writer, &the_json)?;
        Ok(())
    }

    pub fn view_table_from_disk(
        &self,
        selection: usize,
        db_index: usize,
        options: &Vec<String>,
    ) -> Result<String> {
        match options[selection].as_str() {
            "json" => {
                let schema_path_str = self.databases[db_index].disk_mappings[selection]
                    .path
                    .clone();
                let mut translator = JsonTranslator {
                    path: schema_path_str,
                    json: None,
                };
                translator.load_from_disk()?;
                let resp_str = translator.get_string();
                Ok(resp_str)
            }
            "prisma" => {
                let mut translator = PrismaTranslator {
                    path: self.databases[db_index].disk_mappings[selection]
                        .path
                        .clone(),
                    disk_schema: None,
                    db_schema: None,
                };
                translator.load_from_disk()?;
                Ok(translator.get_string())
            }
            _ => anyhow::bail!("invalid selection"),
        }
    }

    pub fn view_table_from_database(
        &self,
        selection: usize,
        db_index: usize,
        options: &Vec<String>,
    ) -> Result<String> {
        match options[selection].as_str() {
            "json" => {
                let mut translator = JsonTranslator {
                    path: self.databases[db_index].disk_mappings[selection]
                        .path
                        .clone(),
                    json: None,
                };
                translator.load_from_database(&self.databases[db_index].get_descriptions());
                Ok(translator.get_string())
            }
            "prisma" => {
                let mut translator = PrismaTranslator {
                    path: self.databases[db_index].disk_mappings[selection]
                        .path
                        .clone(),
                    disk_schema: None,
                    db_schema: None,
                };
                translator.load_from_database(&self.databases[db_index].get_descriptions());
                Ok(translator.get_string())
            }
            _ => {
                anyhow::bail!("invalid selection")
            }
        }
    }

    pub fn write_one_schema_from_database(
        &self,
        selection: usize,
        db_index: usize,
        options: &Vec<String>,
    ) -> Result<()> {
        let descriptions = self.databases[db_index].get_descriptions();
        match options[selection].as_str() {
            "json" => {
                self.databases[db_index].sync_one(
                    AcceptedFormat::Json,
                    self.databases[db_index].disk_mappings[selection]
                        .path
                        .clone(),
                    &descriptions,
                );
                Ok(())
            }
            "prisma" => {
                self.databases[db_index].sync_one(
                    AcceptedFormat::Prisma,
                    self.databases[db_index].disk_mappings[selection]
                        .path
                        .clone(),
                    &descriptions,
                );
                Ok(())
            }
            _ => {
                anyhow::bail!("invalid selection")
            }
        }
    }
}
