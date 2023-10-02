use mysql_translate::{
    functionality::{session::Session, structure::get_session_data_location},
    ui::{behaviour::UI, terminal::TerminalUI},
};

use std::cell::RefCell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Gets the STORAGE environment variable using .env in the root directory.
    let session_data_location = get_session_data_location();
    let mut session_wrapped = Session::new(&session_data_location);
    if session_wrapped.is_none() {
        session_wrapped = Session::new_bare_session(&session_data_location);
    }
    let session = RefCell::new(session_wrapped.expect("session to exist"));
    let mut ui = TerminalUI::new(session);
    if ui.get_session().databases.len() == 0 {
        ui.create_database_entry()?;
    }
    ui.main_loop()?;
    Ok(())
}
