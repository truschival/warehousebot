use super::bot::{Commands, Error};
use crate::{EAST_LIT, NORTH_LIT, SOUTH_LIT, WEST_LIT};

pub struct Cli<T> {
    executor: T,
}

#[derive(Debug)]
pub enum CliError {
    CommandFailed(String),
    CommandUnknown,
    CommandNotImplemented,
}

fn command_error_to_cli_error(err: Error) -> CliError {
    match err {
        Error::HitWall => CliError::CommandFailed("Bot hit a wall".to_string()),
        Error::StoreageFull => CliError::CommandFailed("Storage full".to_string()),
        Error::ScanFailed => CliError::CommandFailed("Scan failed".to_string()),
        Error::ClientError => {
            CliError::CommandFailed("Something is really bad - better abort".to_string())
        }
    }
}

fn process_navigation_command(res: Result<(), Error>, direction: &str) -> Result<String, CliError> {
    match res {
        Ok(_) => Ok(format!("going {direction}")),
        Err(e) => Err(command_error_to_cli_error(e)),
    }
}

impl<T: Commands> Cli<T> {
    pub fn new(ex: T) -> Self {
        Cli { executor: ex }
    }

    pub fn dispatch_command_for_string(&self, cmd: &str) -> Result<String, CliError> {
        let cmd = cmd.to_ascii_lowercase();
        let cmd = cmd.trim();

        match cmd {
            NORTH_LIT => process_navigation_command(self.executor.go_north(), cmd),
            WEST_LIT => process_navigation_command(self.executor.go_west(), cmd),
            SOUTH_LIT => process_navigation_command(self.executor.go_south(), cmd),
            EAST_LIT => process_navigation_command(self.executor.go_east(), cmd),
            "NEAR" | "FAR" => Err(CliError::CommandNotImplemented),
            _ => Err(CliError::CommandUnknown),
        }
    }
}
