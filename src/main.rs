use mysql_translate::{
    flags::flag_parser::get_command_line_flags,
    functionality::{session::Session, structure::get_session_data_location},
    ui::{behaviour::UI, terminal::TerminalUI},
};
use std::cell::RefCell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Gets the STORAGE environment variable using .env in the root directory.
    let args = get_command_line_flags();
    let session_data_location = get_session_data_location();
    let session = match Session::new(&session_data_location) {
        Some(x) => RefCell::new(x),
        None => RefCell::new(
            Session::new_bare_session(&session_data_location).expect("session to exist"),
        ),
    };
    session.borrow_mut().set_command_line_flags(args);
    let mut ui = TerminalUI::new(session);
    if ui.session.borrow().databases.len() == 0 {
        ui.create_database_entry()?;
    }
    ui.main_loop()?;
    Ok(())
}
