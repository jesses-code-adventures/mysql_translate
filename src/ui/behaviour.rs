use crate::functionality::session::Session;
use anyhow::Result;
use core::cell::RefCell;

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
