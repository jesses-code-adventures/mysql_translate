use crate::json_translator;
use crate::sql;
use dialoguer::Select;
use serde_json::json;
/// One database url can be linked up to multiple schema locations.
/// The "name" does not need to match the db name.
pub struct Database {
    pub name: String,
    pub db_url: String,
    pub json_path: Option<String>,
    pub prisma_path: Option<String>,
}

impl Database {
    /// Pull the database info from the db and propagate it.
    pub fn sync(&self) {
        let descriptions = self.get_descriptions();
        match &self.json_path {
            Some(path) => json_translator::write(&descriptions, &path),
            None => println!("No json path found."),
        }
        match &self.prisma_path {
            Some(path) => println!("prisma path: {}", path),
            None => println!("No prisma json path found."),
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
    /// Update the path to push json files to.
    pub fn update_json_path(&mut self, new_json_path: String) {
        self.json_path = Some(new_json_path);
    }
    /// Update the path to push prisma schemas to.
    pub fn update_prisma_path(&mut self, new_prisma_path: String) {
        self.prisma_path = Some(new_prisma_path);
    }
    /// Print a representation of the database
    pub fn display(&self) {
        println!("name: {}", self.name);
        println!("db_url: {}", self.db_url);
        match &self.json_path {
            Some(path) => println!("json_path: {}", path),
            None => println!("No json path found."),
        }
        match &self.prisma_path {
            Some(path) => println!("prisma_path: {}", path),
            None => println!("No prisma path found."),
        }
    }
    /// Get a json value of the database.
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "db_url": self.db_url,
            "json_path": self.json_path,
            "prisma_path": self.prisma_path,
        })
    }
    pub fn edit(&mut self) -> bool {
        let options = vec!["name", "db_url", "json_path", "prisma_path", "done"];
        let mut selection = Select::new().items(&options).default(0).interact().unwrap();
        let mut edit_another = false;
        while options[selection] != "done" {
            match options[selection] {
                "name" => {
                    println!("pls enter the new name:");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read line");
                    self.update_name(input.trim().to_string());
                }
                "db_url" => {
                    println!("pls enter the new db url:");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read line");
                    self.update_db_url(input.trim().to_string());
                }
                "json_path" => {
                    println!("pls enter the new json path:");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read line");
                    self.update_json_path(input.trim().to_string());
                }
                "prisma_path" => {
                    println!("pls enter the new prisma path:");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read line");
                    self.update_prisma_path(input.trim().to_string());
                }
                "edit_another" => {
                    edit_another = true;
                    break;
                }
                "done" => {}
                _ => println!("invalid selection"),
            }
            selection = Select::new().items(&options).default(0).interact().unwrap();
        }
        edit_another
    }

    fn get_descriptions(&self) -> Vec<sql::Table> {
        sql::get_table_descriptions(&self.db_url).expect("Failed to get table descriptions")
    }
}
