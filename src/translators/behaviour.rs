use crate::remotes::sql::Table;
use anyhow::Result;

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
