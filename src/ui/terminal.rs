use anyhow::Result;
use dialoguer::Select;
use std::cell::RefCell;

use crate::{
    functionality::{database, session::Session, structure::AcceptedFormat},
    ui::behaviour::UI,
};

static MAIN_MENU_ITEMS: [&str; 8] = [
    "display databases",
    "sync databases",
    "add database",
    "edit databases",
    "view tables from disk",
    "view tables from database",
    "write one",
    "exit",
];
static WELCOME_STRING: &str = "welcome to mysqueal translate~!";
static GOODBYE_STRING: &str = "goodbye!";
static NEWLINES_STRING: &str = "\n\n";

/// A TUI implementation for mysql translate.
/// Probably not the nicest way to do this.
pub struct TerminalUI {
    pub session: RefCell<Session>,
}

impl TerminalUI {
    fn welcome(&self) {
        println!("{}", WELCOME_STRING);
    }

    fn clear_terminal(&self) {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }

    fn goodbye(&self) {
        println!("{}", GOODBYE_STRING);
    }

    fn create_newlines(&self) {
        println!("{}", NEWLINES_STRING);
    }

    fn prompt_user_until_successful(&self, prompt: String) -> String {
        let mut is_empty = false;
        let mut input_failed = false;
        let mut input = String::new();
        loop {
            self.clear_terminal();
            if is_empty {
                println!("input cannot be empty");
            }
            if input_failed {
                println!("invalid input");
            }
            println!("{}", prompt);
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {}
                Err(_) => {
                    input_failed = true;
                    is_empty = false;
                    continue;
                }
            }
            if input.trim().is_empty() {
                is_empty = true;
                input_failed = false;
                continue;
            } else {
                break;
            }
        }
        input
    }
}

impl UI for TerminalUI {
    /// Create a new TerminalUI
    fn new(session: RefCell<Session>) -> Self {
        let resp = TerminalUI { session };
        resp.clear_terminal();
        resp.welcome();
        resp
    }

    /// Get a non mutable reference to the session
    fn get_session(&self) -> std::cell::Ref<Session> {
        self.session.borrow()
    }

    /// Get a mutable reference to the session
    fn get_session_mut(&self) -> std::cell::RefMut<Session> {
        self.session.borrow_mut()
    }

    /// Returns the index of the selected database in self.databases
    fn select_database(&self) -> Result<usize> {
        let selection = Select::new()
            .with_prompt("select a database")
            .default(0)
            .items(
                &self
                    .get_session_mut()
                    .databases
                    .iter()
                    .map(|database| database.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact()?;
        Ok(selection)
    }

    fn create_database_entry(&mut self) -> Result<()> {
        let name = self.prompt_user_until_successful("enter the database name".to_string());
        let db_url =
            self.prompt_user_until_successful("enter the database connection url".to_string());
        self.get_session_mut()
            .add_database(database::Database {
                name,
                db_url,
                disk_mappings: Vec::new(),
            })
            .expect("adding database should work");
        self.get_session_mut()
            .databases
            .last_mut()
            .expect("database to exist")
            .edit();
        self.get_session_mut()
            .save()
            .expect("saving database should work");
        self.get_session_mut()
            .load()
            .expect("loading database should work");
        println!("entry created successfully");
        Ok(())
    }

    fn select_schema_to_write(&mut self) -> Result<()> {
        self.clear_terminal();
        let db_index = self.select_database()?;
        let options = AcceptedFormat::all_as_string_array();
        let selection = Select::new()
            .with_prompt("which schema would you like to view from the disk?")
            .default(0)
            .items(&options)
            .interact()?;
        match self
            .get_session()
            .write_one_schema_from_database(selection, db_index, &options)
        {
            Ok(_) => {
                println!("write successful");
                Ok(())
            }
            Err(e) => {
                println!("write failed: {}", e);
                Ok(())
            }
        }
    }

    fn view_tables_from_disk(&self) -> Result<()> {
        self.clear_terminal();
        let db_index = self.select_database()?;
        let options = AcceptedFormat::all_as_string_array();
        let selection = Select::new()
            .with_prompt("which schema would you like to view from the disk?")
            .default(0)
            .items(&options)
            .interact()?;
        println!(
            "{}",
            self.get_session()
                .view_table_from_disk(selection, db_index, &options)?
        );
        Ok(())
    }

    fn view_tables_from_database(&self) -> Result<()> {
        self.clear_terminal();
        let db_index = self.select_database()?;
        let options = AcceptedFormat::all_as_string_array();
        let selection = Select::new()
            .with_prompt("which schema would you like view from the database?")
            .default(0)
            .items(&options)
            .interact()?;
        println!(
            "{}",
            self.get_session()
                .view_table_from_database(selection, db_index, &options)?
        );
        Ok(())
    }

    fn edit_databases(&mut self) -> Result<()> {
        let mut edit_another = true;
        while edit_another {
            let db_index = self.select_database()?;
            edit_another = self.get_session_mut().databases[db_index].edit();
        }
        Ok(())
    }

    /// Main entry point
    fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.create_newlines();
            let selection = Select::new()
                .with_prompt("what would you like to do?")
                .default(0)
                .items(&MAIN_MENU_ITEMS)
                .interact()?;
            match selection {
                0 => {
                    self.clear_terminal();
                    self.get_session_mut().display();
                }
                1 => {
                    self.clear_terminal();
                    self.get_session_mut().sync()
                }
                2 => self.create_database_entry()?,
                3 => self.edit_databases()?,
                4 => self.view_tables_from_disk()?,
                5 => self.view_tables_from_database()?,
                6 => self.select_schema_to_write()?,
                7 => {
                    self.goodbye();
                    return Ok(());
                }
                _ => println!("Invalid selection"),
            }
        }
    }
}
