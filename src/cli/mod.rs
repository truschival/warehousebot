use log::debug;

use super::botcommands::{BotCommands, CommandError};

pub struct Cli<T> {
    executor: T,
}

#[derive(Debug)]
pub enum CliError {
    CommandFailed(String),
    CommandUnknown,
    CommandNotImplemented,
}

fn command_error_to_cli_error(err: CommandError) -> CliError {
    match err {
        CommandError::HitWall => CliError::CommandFailed("hit the wall".to_string()),
        CommandError::StoreageFull => CliError::CommandFailed("storage full".to_string()),
    }
}

impl<T: BotCommands> Cli<T> {
    pub fn new(ex: T) -> Self {
        Cli { executor: ex }
    }

    pub fn dispatch_command_for_string(&self, cmd: &str) -> Result<String, CliError> {
        let cmd = cmd.to_uppercase();
        match cmd.trim() {
            "NORTH" => {
                debug!("norden");

                match self.executor.go_north() {
                    Ok(_) => {
                        debug!("Go North worked");
                        Ok("going north".to_string())
                    }
                    Err(e) => Err(command_error_to_cli_error(e)),
                }
            }
            "WEST" => {
                debug!("West");
                match self.executor.go_west() {
                    Ok(_) => {
                        debug!("Go West worked");
                        Ok("going west".to_string())
                    }
                    Err(e) => Err(command_error_to_cli_error(e)),
                }
            }
            "SOUTH" | "East" => Err(CliError::CommandNotImplemented),
            "NEAR" | "FAR" => Err(CliError::CommandNotImplemented),
            _ => Err(CliError::CommandUnknown),
        }
    }
}
