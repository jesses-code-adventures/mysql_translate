#![allow(dead_code)]
mod database;
mod json_translator;
mod prisma_translator;
mod session;
mod set_env_variables;
mod sql;
mod structure;

fn welcome() {
    println!("hello! welcome to translate.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sets the environment variables using .env in the root directory.
    welcome();
    set_env_variables::set_vars();
    let mut session = session::Session {
        _data_location: String::from(".data/session.json"),
        databases: vec![],
    };
    session.load()?;
    if session.databases.len() == 0 {
        session.create_database_entry()?;
        session.databases[0].edit();
        session.save()?;
    }
    session.load()?;
    session.main_menu()?;
    println!("\ngoodbye!");
    Ok(())
}
