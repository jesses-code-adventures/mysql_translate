use mysql_translate::session::Session;
use mysql_translate::set_env_variables;
use mysql_translate::structure::UI;
use mysql_translate::ui::terminal::TerminalUI;

use std::cell::RefCell;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sets the environment variables using .env in the root directory.
    set_env_variables::set_vars();
    let mut data_location =
        env::var("STORAGE").expect("storage directory to exist as an environment variable");
    data_location.push_str("/session.json");
    let mut session_wrapped = Session::new(&data_location);
    if session_wrapped.is_none() {
        session_wrapped = Session::new_bare_session(&data_location);
    }
    let session = RefCell::new(session_wrapped.expect("session to exist"));
    let mut ui = TerminalUI::new(session);
    if ui.get_session().databases.len() == 0 {
        ui.create_database_entry()?;
    }
    ui.main_loop()?;
    Ok(())
}
