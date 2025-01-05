use super::bot::Commands;
use crate::{
    cliplotter, explore_warehouse_manually, load_state, save_state,
    update_warehouse_at_bot_postion, update_warehouse_from_scan_far, warehouse::Warehouse, Error,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const CLI_STATE_FILE_NAME: &str = "cli_state.json";
#[derive(Serialize, Deserialize)]
pub struct Cli<T: Serialize> {
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
        Error::InvalidDirection => CliError::CommandFailed("Invalid Direction".to_string()),
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

impl<T: Commands + Serialize + DeserializeOwned> Cli<T> {
    pub fn new(ex: T) -> Self {
        Cli {
            executor: ex,
            warehouse: Warehouse::default(),
        }
    }

    pub fn init_state(&mut self) -> Result<String, CliError> {
        // init bot in warehouse, wherever it starts we have the origin of our coordinate system
        botresult_to_cli(update_warehouse_at_bot_postion(
            &mut self.executor,
            &mut self.warehouse,
        ))
    }

    pub fn dispatch_command_for_string(&mut self, cmd: &str) -> Result<String, CliError> {
        let cmd = cmd.to_ascii_lowercase();
        let cmd = cmd.trim();

        match cmd {
            "north" => {
                botresult_to_cli(explore_warehouse_manually(
                    &mut self.executor,
                    &mut self.warehouse,
                    crate::Direction::NORTH,
                ))?;
                self.show_warehouse()
            }
            "east" => {
                botresult_to_cli(explore_warehouse_manually(
                    &mut self.executor,
                    &mut self.warehouse,
                    crate::Direction::EAST,
                ))?;
                self.show_warehouse()
            }
            "south" => {
                botresult_to_cli(explore_warehouse_manually(
                    &mut self.executor,
                    &mut self.warehouse,
                    crate::Direction::SOUTH,
                ))?;
                self.show_warehouse()
            }
            "west" => {
                botresult_to_cli(explore_warehouse_manually(
                    &mut self.executor,
                    &mut self.warehouse,
                    crate::Direction::WEST,
                ))?;
                self.show_warehouse()
            }
            "save_state" => {
                if let Err(e) = save_state(&self, CLI_STATE_FILE_NAME) {
                    Err(CliError::CommandFailed(e.to_string()))
                } else {
                    Ok("Saved State".to_string())
                }
            }
            "load_state" => match load_state::<Self>(CLI_STATE_FILE_NAME) {
                Some(cli_state) => {
                    self.warehouse = cli_state.warehouse;
                    self.executor = cli_state.executor;
                    Ok("Loaded State".to_string())
                }
                None => Err(CliError::CommandFailed("error loading state".to_string())),
            },
            "show_warehouse" => self.show_warehouse(),
            "inspect_warehouse" => Ok(self.warehouse_info()),
            "reset_warehouse" => {
                self.warehouse.reset();
                Ok("cleared warehouse".to_string())
            }
            "near" => match self.executor.scan_near() {
                Ok(cx) => Ok(cx.to_string()),
                Err(e) => Err(command_error_to_cli_error(e)),
            },
            "locate" => Ok(self.executor.locate().to_string()),
            "far" => {
                let far_scan = self.executor.scan_far().unwrap();
                let info = format!("{:?}", &far_scan);
                let botlocation = self.executor.locate();
                if let Err(e) =
                    update_warehouse_from_scan_far(&mut self.warehouse, far_scan, &botlocation)
                {
                    Err(command_error_to_cli_error(e))
                } else {
                    Ok(info)
                }
            }

            _ => Err(CliError::CommandUnknown),
        }
    }

    fn warehouse_info(&self) -> String {
        let storage = self.warehouse.storage_capacity();
        format!(
            "#cells: {} storage: {}/{}\n",
            self.warehouse.cells(),
            storage.0,
            storage.1
        )
    }

    fn show_warehouse(&mut self) -> Result<String, CliError> {
        let botlocation = Some(self.executor.locate());
        let mut s = self.warehouse_info();
        s.push_str(cliplotter::draw_warehouse(self.warehouse.get_cellgrid(), botlocation).as_str());
        Ok(s)
    }
}
