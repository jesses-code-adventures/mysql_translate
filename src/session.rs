use crate::database;
use crate::prisma_translator::PrismaTranslator;
use crate::structure::{AcceptedFormat, DiskMapping};
use dialoguer::Select;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Session<'a> {
    pub databases: Vec<database::Database<'a>>,
    pub _data_location: String,
}

impl<'a> Session<'a> {
    pub fn new(full_path: &String) -> Session {
        let mut s = Session {
            databases: Vec::new(),
            _data_location: full_path.to_string(),
        };
        s.load()
            .unwrap_or_else(|_| println!("failed to load session"));
        return s;
    }
    pub fn load(&mut self) -> Result<(), std::io::Error> {
        let path = Path::new(self._data_location.as_str());
        let file = File::open(&path).unwrap_or_else(|_| File::create(&path).unwrap());
        let reader = std::io::BufReader::new(file);
        let the_json: Vec<serde_json::Value> =
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new());
        for mut database in the_json {
            let name = database["name"].as_str().unwrap().to_string();
            let db_url = database["db_url"].as_str().unwrap().to_string();
            let disk_mappings = DiskMapping::from_json(database["disk_mappings"].take()).unwrap();
            let database = database::Database {
                name,
                db_url,
                disk_mappings,
            };
            self.add_database(database);
        }
        Ok(())
    }
    pub fn add_database(&mut self, database: database::Database<'a>) {
        if database.db_url.len() == 0 {
            println!("Database url is empty. Skipping.");
            return;
        }
        if self
            .find_existing_database_index(&database.db_url)
            .is_some()
        {
            return;
        }
        self.databases.push(database);
        self.save()
            .unwrap_or_else(|_| println!("database failed to save"));
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
    pub fn create_database_entry(&mut self) -> Result<(), std::io::Error> {
        println!("pls enter the database url:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        println!("what would you like to call this database?");
        let mut name = String::new();
        std::io::stdin().read_line(&mut name)?;
        self.add_database(database::Database {
            name: name.trim().to_string(),
            db_url: input.trim().to_string(),
            disk_mappings: Vec::new(),
        });
        self.save()
            .unwrap_or_else(|_| println!("new database failed to save"));
        Ok(())
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
        self.save()
            .unwrap_or_else(|_| println!("saving failed on removal"));
    }
    pub fn sort(&mut self) {
        self.databases.sort_by(|a, b| a.name.cmp(&b.name));
    }
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
    /// Returns the index of the selected database in self.databases
    pub fn select_database(&self) -> Result<usize, std::io::Error> {
        let selection = Select::new()
            .with_prompt("select a database")
            .default(0)
            .items(
                &self
                    .databases
                    .iter()
                    .map(|database| database.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact()?;
        Ok(selection)
    }
    pub fn edit_databases(&mut self) -> Result<(), std::io::Error> {
        let mut edit_another = true;
        while edit_another {
            let db_index = self.select_database()?;
            edit_another = self.databases[db_index].edit();
        }
        Ok(())
    }
    pub fn select_schema_to_write(&mut self) -> Result<(), std::io::Error> {
        let _database = self.select_database();
        let options = vec!["json", "prisma", "exit"];
        let selection = Select::new()
            .with_prompt("which schema would you like to write?")
            .default(0)
            .items(&options)
            .interact()?;
        match options[selection] {
            "json" => Ok(()),
            "prisma" => Ok(()),
            "exit" => Ok(()),
            _ => Ok(()),
        }
    }

    pub fn select_schema_to_view(&self) -> Result<(), std::io::Error> {
        let db_index = self.select_database()?;
        let options = vec!["json", "prisma", "exit"];
        let selection = Select::new()
            .with_prompt("which schema would you like to see?")
            .default(0)
            .items(&options)
            .interact()?;

        match options[selection] {
            "json" => {
                let schema = self.databases[db_index].view_schema(AcceptedFormat::Json);
                match schema {
                    Some(x) => {
                        println!("{:?}", serde_json::to_writer_pretty(std::io::stdout(), &x));
                        Ok(())
                    }
                    None => {
                        println!("no schema found");
                        Ok(())
                    }
                }
            }
            "prisma" => {
                let mut mapping_index = 0;
                let mut found = false;
                for (i, mapping) in self.databases[db_index].disk_mappings.iter().enumerate() {
                    if mapping.format == AcceptedFormat::Prisma {
                        mapping_index = i;
                        found = true;
                    }
                }
                if !found {
                    println!("no prisma schema found at the set path!");
                    return Ok(());
                }
                let translator = PrismaTranslator {
                    disk_mapping: &self.databases[db_index].disk_mappings[mapping_index],
                };
                translator.load();
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn main_menu(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let selection = Select::new()
            .with_prompt("what would you like to do?")
            .default(0)
            .items(&[
                "display databases",
                "sync databases",
                "add database",
                "edit databases",
                "browse schemas",
                "write one",
                "exit",
            ])
            .interact()
            .unwrap();
        match selection {
            0 => {
                self.display();
                self.main_menu()?;
            }
            1 => {
                self.sync();
                self.main_menu()?;
            }
            2 => {
                self.create_database_entry()?;
                self.main_menu()?;
            }
            3 => {
                self.edit_databases()?;
                self.main_menu()?;
            }
            4 => {
                self.select_schema_to_view()?;
                self.main_menu()?;
            }
            5 => {
                self.select_schema_to_write()?;
                self.main_menu()?;
            }
            6 => {}
            _ => {
                println!("Invalid selection");
                self.main_menu()?;
            }
        }
        Ok(())
    }
}
