use super::bot::{Commands, Error};
use crate::{
    move_bot_in_warehouse, save_state, warehouse::Warehouse, BOT_STATE_FILE_NAME, EAST_LIT,
    NORTH_LIT, SOUTH_LIT, WAREHOUSE_STATE_FILE_NAME, WEST_LIT,
};
use serde::Serialize;

pub struct Cli<T> {
    executor: T,
    warehouse: Warehouse,
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
        Error::InvalidBot => CliError::CommandFailed("No such bot!".to_string()),
        Error::ClientError => {
            CliError::CommandFailed("Something is really bad - better abort".to_string())
        }
    }
}

fn botresult_to_cli(res: Result<(), Error>) -> Result<String, CliError> {
    match res {
        Ok(_) => Ok("ok".to_string()),
        Err(e) => Err(command_error_to_cli_error(e)),
    }
}

impl<T: Commands + Serialize> Cli<T> {
    pub fn new(ex: T) -> Self {
        Cli {
            executor: ex,
            warehouse: Warehouse::default(),
        }
    }

    pub fn dispatch_command_for_string(&mut self, cmd: &str) -> Result<String, CliError> {
        let cmd = cmd.to_ascii_lowercase();
        let cmd = cmd.trim();

        match cmd {
            NORTH_LIT => botresult_to_cli(move_bot_in_warehouse(
                &mut self.executor,
                &mut self.warehouse,
                crate::Direction::NORTH,
            )),
            EAST_LIT => botresult_to_cli(move_bot_in_warehouse(
                &mut self.executor,
                &mut self.warehouse,
                crate::Direction::EAST,
            )),
            SOUTH_LIT => botresult_to_cli(move_bot_in_warehouse(
                &mut self.executor,
                &mut self.warehouse,
                crate::Direction::SOUTH,
            )),
            WEST_LIT => botresult_to_cli(move_bot_in_warehouse(
                &mut self.executor,
                &mut self.warehouse,
                crate::Direction::WEST,
            )),
            "save_bot" => {
                let serialized = serde_json::to_string(&self.executor).unwrap();
                if let Err(e) = save_state(&serialized, BOT_STATE_FILE_NAME) {
                    Err(CliError::CommandFailed(e.to_string()))
                } else {
                    Ok("Saved bot state".to_string())
                }
            }
            "save_warehouse" => {
                let serialized = serde_json::to_string(&self.warehouse);
                match serialized {
                    Ok(serialized) => {
                        if let Err(e) = save_state(&serialized, WAREHOUSE_STATE_FILE_NAME) {
                            Err(CliError::CommandFailed(e.to_string()))
                        } else {
                            Ok("Saved warehouse state".to_string())
                        }
                    }
                    Err(e) => {
                        log::error!("Error serializing warehouse {}", e.to_string());
                        Err(CliError::CommandFailed(e.to_string()))
                    }
                }
            }
            "locate" => Ok(format!("Location (rel): {}", self.executor.locate())),
            "reset" => botresult_to_cli(self.executor.reset()),
            "NEAR" | "FAR" => Err(CliError::CommandNotImplemented),
            _ => Err(CliError::CommandUnknown),
        }
    }
}
