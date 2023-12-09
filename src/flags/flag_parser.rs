use clap::{command, crate_name, Parser};

/// Simple program to greet a person
#[derive(Parser, Debug, Clone)]
#[command(author, version, name=crate_name!())]
pub struct CommandLineFlags {
    /// Testing mode is activated
    #[arg(short, long)]
    pub test: bool,
}

pub fn get_command_line_flags() -> CommandLineFlags {
    let args = CommandLineFlags::parse();
    args
}
