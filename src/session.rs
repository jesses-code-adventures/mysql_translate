use crate::database;
use dialoguer::Select;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Session {
    pub databases: Vec<database::Database>,
    pub _data_location: String,
}

impl Session {
    pub fn new() -> Session {
        Session {
            databases: Vec::new(),
            _data_location: String::from(".data/session.json"),
        }
    }
    pub fn add(&mut self, database: database::Database) {
        if database.db_url.len() == 0 {
            println!("Database url is empty. Skipping.");
            return;
        }
        if self
            .find_existing_database_index(&database.db_url)
            .is_some()
        {
            println!("Database already exists. Skipping.");
            return;
        }
        self.databases.push(database);
        self.save();
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
    pub fn create_database_entry(&mut self) {
        println!("pls enter the database url:");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");
        println!("what would you like to call this database?");
        let mut name = String::new();
        std::io::stdin()
            .read_line(&mut name)
            .expect("failed to read line");
        self.add(database::Database {
            name: name.trim().to_string(),
            db_url: input.trim().to_string(),
            json_path: None,
            prisma_path: None,
        });
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
    fn remove_database(&mut self, index: usize) {
        self.databases.remove(index);
        self.save();
    }
    pub fn sort(&mut self) {
        self.databases.sort_by(|a, b| a.name.cmp(&b.name));
    }
    pub fn save(&self) {
        let path = Path::new(self._data_location.as_str());
        let file = File::create(&path).unwrap();
        let buf_writer = BufWriter::new(file);
        let the_json = self
            .databases
            .iter()
            .map(|database| database.to_json())
            .collect::<Vec<_>>();
        serde_json::to_writer_pretty(buf_writer, &the_json).unwrap();
    }
    pub fn load_json(&mut self) {
        let path = Path::new(self._data_location.as_str());
        let file = File::open(&path).unwrap_or_else(|_| File::create(&path).unwrap());
        let reader = std::io::BufReader::new(file);
        let the_json: Vec<serde_json::Value> =
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new());
        for database in the_json {
            let name = database["name"].as_str().unwrap().to_string();
            let db_url = database["db_url"].as_str().unwrap().to_string();
            let json_path = Some(
                database["json_path"]
                    .as_str()
                    .unwrap_or_else(|| "")
                    .to_string(),
            );
            let prisma_path = Some(
                database["prisma_path"]
                    .as_str()
                    .unwrap_or_else(|| "")
                    .to_string(),
            );
            let database = database::Database {
                name,
                db_url,
                json_path,
                prisma_path,
            };
            self.add(database);
        }
    }
    pub fn select_database(&mut self) -> &mut database::Database {
        // User can select a database to edit.
        self.sort();
        let selection = Select::new()
            .with_prompt("Select a database to edit")
            .default(0)
            .items(
                &self
                    .databases
                    .iter()
                    .map(|database| database.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact()
            .unwrap();
        &mut self.databases[selection]
    }
    pub fn edit_databases(&mut self) {
        let mut edit_another = true;
        while edit_another {
            let database = self.select_database();
            edit_another = database.edit();
        }
        self.save();
    }
    pub fn main_menu(&mut self) {
        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&[
                "Display databases",
                "Sync databases",
                "Add database",
                "Edit databases",
                "Exit",
            ])
            .interact()
            .unwrap();
        match selection {
            0 => {
                self.display();
                self.main_menu();
            }
            1 => {
                self.sync();
                self.main_menu();
            }
            2 => {
                self.create_database_entry();
                self.main_menu();
            }
            3 => {
                self.edit_databases();
                self.main_menu();
            }
            4 => {
                println!("Goodbye!");
            }
            _ => {
                println!("Invalid selection");
                self.main_menu();
            }
        }
    }
}
