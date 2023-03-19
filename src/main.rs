#![allow(dead_code)]
mod database;
mod json_translator;
mod session;
mod set_env_variables;
mod sql;

fn welcome() {
    println!("hello! welcome to translate.");
}

fn main() {
    // Sets the environment variables using .env in the root directory.
    welcome();
    set_env_variables::set_vars();
    let mut session = session::Session {
        _data_location: String::from(".data/session.json"),
        databases: vec![],
    };
    session.load_json();
    session.main_menu();
    println!("goodbye!");
}
