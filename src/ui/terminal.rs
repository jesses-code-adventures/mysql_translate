use crate::{
    functionality::{database, session::Session, structure::AcceptedFormat},
    ui::behaviour::UI,
};
use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, MoveUp},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType, SetTitle},
};
use dialoguer::Select;
use std::cell::RefCell;
use std::process::exit;

/// TODO: Remove view tables from disk vs view tables from database
/// TODO: Add view diff between database and disk
static MAIN_MENU_ITEMS: [&str; 9] = [
    "display databases",
    "sync databases",
    "sync one",
    "add database",
    "edit databases",
    "view tables from disk",
    "view tables from database",
    "clear terminal",
    "exit",
];
static TITLE: &str = "~ mysql translate ~";
static WELCOME_STRING: &str = "welcome to mysql translate 🌴";
static GOODBYE_STRING: &str = "goodbye 👋";

/// An interactive CLI implementation of the UI
pub struct TerminalUI {
    pub session: RefCell<Session>,
    keep_previous_content: bool,
}

impl TerminalUI {
    /// Prints a green message
    fn happy_message(&self, message: &str) {
        execute!(
            std::io::stdout(),
            SetForegroundColor(Color::DarkGreen),
            Print(format!("{}\n", message)),
            ResetColor
        )
        .expect("print to work");
    }

    /// Prints a red message
    fn sad_message(&self, message: &str) {
        execute!(
            std::io::stdout(),
            SetForegroundColor(Color::DarkRed),
            Print(format!("{}\n", message)),
            ResetColor
        )
        .expect("print to work");
    }

    /// Prints a nice other colour for prompts (currently dark magenta)
    fn prompt_message(&self, message: &str) {
        execute!(
            std::io::stdout(),
            SetForegroundColor(Color::DarkMagenta),
            Print(format!("{}\n", message)),
            ResetColor
        )
        .expect("print to work");
    }

    fn welcome(&self) {
        self.happy_message(WELCOME_STRING);
    }

    fn clear_terminal_line(&self) {
        if self.keep_previous_content {
            return;
        }
        execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).expect("clear to work");
    }

    fn clear_line_above(&self) {
        if self.keep_previous_content {
            return;
        }
        execute!(std::io::stdout(), MoveUp(1)).expect("move to work");
        execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).expect("clear to work");
    }

    fn clear_whole_terminal(&self) {
        if self.keep_previous_content {
            return;
        }
        execute!(std::io::stdout(), Clear(ClearType::FromCursorUp)).expect("clear to work");
        execute!(std::io::stdout(), MoveTo(0, 0)).expect("move to work");
    }

    fn goodbye(&self) {
        self.happy_message(GOODBYE_STRING);
    }

    /// General validator to ensure input isn't empty and hasn't failed
    fn prompt_user_until_successful(&mut self, prompt: &str) -> String {
        let mut input = String::new();
        loop {
            self.keep_previous_content = false;
            self.clear_terminal_line();
            self.prompt_message(prompt); // testing1
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {}
                Err(_) => {
                    self.sad_message("invalid input, please re-enter");
                    input = String::new();
                    continue;
                }
            }
            if input.trim().is_empty() {
                self.keep_previous_content = false;
                self.clear_line_above();
                self.keep_previous_content = true;
                self.sad_message("input cannot be empty");
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
        let mut resp = TerminalUI {
            session,
            keep_previous_content: false,
        };
        resp.clear_whole_terminal();
        resp.welcome();
        execute!(std::io::stdout(), SetTitle(TITLE)).unwrap();
        resp.keep_previous_content = true;
        resp
    }

    /// Returns the index of the selected database in self.databases
    fn select_database(&self) -> Result<usize> {
        self.clear_whole_terminal();
        self.prompt_message("select a database");
        let selection = Select::new()
            .default(0)
            .items(
                &self
                    .session
                    .borrow_mut()
                    .databases
                    .iter()
                    .map(|database| database.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact_opt()?
            .unwrap_or_else(|| {
                self.goodbye();
                exit(0)
            });
        Ok(selection)
    }

    fn create_database_entry(&mut self) -> Result<()> {
        self.clear_whole_terminal();
        let name = self.prompt_user_until_successful("enter the database name");
        let db_url = self.prompt_user_until_successful("enter the database connection url");
        let disk_mappings = Vec::new();
        let database = database::Database {
            name,
            db_url,
            disk_mappings,
        };
        self.session.borrow_mut().add_database(database)?;
        let database_index = self.session.borrow().databases.len() - 1;
        self.edit_database(database_index)?;
        self.session.borrow_mut().save_database_info()?;
        self.session.borrow_mut().load()?;
        self.happy_message("entry created successfully");
        Ok(())
    }

    fn select_schema_to_write(&mut self) -> Result<()> {
        self.clear_whole_terminal();
        let db_index = self.select_database()?;
        let options = AcceptedFormat::all_as_string_array();
        self.prompt_message("which schema would you like to write to the disk?");
        let selection = Select::new()
            .default(0)
            .items(&options)
            .interact_opt()?
            .unwrap_or_else(|| {
                self.goodbye();
                exit(0)
            });
        match self
            .session
            .borrow()
            .write_one_schema_from_database(selection, db_index, &options)
        {
            Ok(_) => {
                self.happy_message("write successful");
            }
            Err(e) => {
                self.sad_message(format!("error: {}", e.to_string()).as_str());
            }
        }
        Ok(())
    }

    fn view_tables_from_disk(&self) -> Result<()> {
        self.clear_terminal_line();
        let db_index = self.select_database()?;
        let options = AcceptedFormat::all_as_string_array();
        self.prompt_message("which schema would you like to view from the disk?");
        let selection = Select::new()
            .default(0)
            .items(&options)
            .interact_opt()?
            .unwrap_or_else(|| {
                self.goodbye();
                exit(0)
            });
        let schema = self
            .session
            .borrow()
            .get_current_local_database(selection, db_index, &options);
        match schema {
            Ok(x) => {
                println!("{}", x);
            }
            _ => {
                self.sad_message(
                    "Schema display failed - you might need to sync your databases locally.\n",
                );
            }
        }
        Ok(())
    }

    fn view_tables_from_database(&self) -> Result<()> {
        let db_index = self.select_database()?;
        let options = AcceptedFormat::all_as_string_array();
        self.prompt_message("which schema would you like view from the database?");
        let selection = Select::new()
            .default(0)
            .items(&options)
            .interact_opt()?
            .unwrap_or_else(|| {
                self.goodbye();
                exit(0)
            });
        println!(
            "{}",
            self.session
                .borrow()
                .view_table_from_database(selection, db_index, &options)?
        );
        Ok(())
    }

    fn edit_databases(&mut self) -> Result<()> {
        self.keep_previous_content = false;
        self.clear_terminal_line();
        let db_index = self.select_database()?;
        self.edit_database(db_index)?;
        Ok(())
    }

    fn edit_disk_mappings(&mut self, database_index: usize) -> Result<()> {
        self.clear_whole_terminal();
        let mut options = AcceptedFormat::all_as_str_array();
        options.push("exit");
        let selection = Select::new().items(&options).default(0).interact_opt()?;
        match AcceptedFormat::from_string(
            options[selection.unwrap_or_else(|| {
                self.goodbye();
                exit(0)
            })],
        ) {
            Some(x) => {
                let input = self.prompt_user_until_successful(
                    format!("enter the full path to the {} schema", x.as_string()).as_str(),
                );
                self.session.borrow_mut().databases[database_index].update_disk_mapping(x, input);
            }
            None => {
                return Ok(());
            }
        }
        Ok(())
    }

    fn edit_database(&mut self, database_index: usize) -> Result<()> {
        let options = vec!["name", "db_url", "disk_mappings", "done"];
        let mut selection = Select::new().items(&options).default(0).interact().unwrap();
        while options[selection] != "done" {
            match options[selection] {
                "name" => {
                    let input = self.prompt_user_until_successful("enter the new name");
                    self.session.borrow_mut().databases[database_index]
                        .update_name(input.trim().to_string());
                }
                "db_url" => {
                    let input = self.prompt_user_until_successful("enter the new db_url");
                    self.session.borrow_mut().databases[database_index]
                        .update_db_url(input.trim().to_string());
                }
                "disk_mappings" => {
                    self.edit_disk_mappings(database_index)?;
                }
                "edit_another" => {
                    break;
                }
                "done" => {}
                _ => println!("invalid selection"),
            }
            selection = Select::new().items(&options).default(0).interact()?;
        }
        Ok(())
    }

    fn display_session(&self) {
        for (i, _) in self.session.borrow().databases.iter().enumerate() {
            self.display_database(i);
        }
    }

    /// Print a representation of the database
    fn display_database(&self, database_index: usize) {
        let database = &self.session.borrow().databases[database_index];
        println!("name: {}", database.name);
        for mapping in database.disk_mappings.iter() {
            match mapping.format {
                AcceptedFormat::Json => println!("json_path: {}", mapping.path),
                AcceptedFormat::Prisma => println!("prisma_path: {}", mapping.path),
            }
        }
    }

    /// Main entry point
    fn main_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.prompt_message("what would you like to do?");
            let selection = Select::new()
                .default(0)
                .items(&MAIN_MENU_ITEMS)
                .interact_opt()?
                .unwrap_or_else(|| {
                    self.goodbye();
                    exit(0)
                });
            self.keep_previous_content = false;
            self.clear_line_above();
            self.keep_previous_content = true;
            match selection {
                0 => {
                    self.display_session();
                }
                1 => self.session.borrow_mut().sync()?,
                2 => self.select_schema_to_write()?,
                3 => self.create_database_entry()?,
                4 => self.edit_databases()?,
                5 => self.view_tables_from_disk()?,
                6 => self.view_tables_from_database()?,
                7 => {
                    self.keep_previous_content = false;
                    self.clear_whole_terminal();
                    self.keep_previous_content = true;
                    self.welcome();
                }
                8 => {
                    self.goodbye();
                    return Ok(());
                }
                _ => self.sad_message("Invalid selection"),
            }
        }
    }
}
