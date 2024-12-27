pub mod cli;
pub mod warehouse;
use std::fs;
#[derive(Debug)]
pub enum Direction {
    NORTH = 0,
    EAST = 1,
    SOUTH = 2,
    WEST = 3,
}

#[derive(Debug)]
pub enum Error {
    InvalidDirection,
}

pub const NORTH_LIT: &str = "north";
pub const EAST_LIT: &str = "east";
pub const SOUTH_LIT: &str = "south";
pub const WEST_LIT: &str = "west";

pub const WAREHOUSE_STATE_FILE_NAME: &str = "warehouse.json";
pub const BOT_STATE_FILE_NAME: &str = "bot.json";

pub fn literal_to_direction(lit: &str) -> Result<Direction, Error> {
    let lit = lit.to_lowercase();
    match lit.as_str() {
        NORTH_LIT => Ok(Direction::NORTH),
        WEST_LIT => Ok(Direction::WEST),
        SOUTH_LIT => Ok(Direction::SOUTH),
        EAST_LIT => Ok(Direction::EAST),
        _ => Err(Error::InvalidDirection),
    }
}

fn direction_to_literal(direction: &crate::Direction) -> String {
    match direction {
        Direction::NORTH => NORTH_LIT.to_string(),
        Direction::WEST => WEST_LIT.to_string(),
        Direction::SOUTH => SOUTH_LIT.to_string(),
        Direction::EAST => EAST_LIT.to_string(),
    }
}

fn save_state(data: &str, filename: &str) -> Result<(), std::io::Error> {
    if let Some(mut filepath) = dirs::data_dir() {
        if !filepath.exists() {
            match fs::create_dir_all(&filepath) {
                Ok(_) => log::info!("Created dir {:?}", &filepath),
                Err(e) => return Err(e),
            }
        }

        filepath.push(filename);
        log::debug!("Writing to {:?}", filepath);
        fs::write(&filepath, data)
    } else {
        Err(std::io::ErrorKind::NotADirectory.into())
    }
}

pub mod bot {
    use crate::{
        warehouse::Coords2D,
        Direction::{EAST, NORTH, SOUTH, WEST},
    };
    use log::debug;
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, fs, path::PathBuf};

    pub mod rest;

    #[derive(Debug, PartialEq)]
    pub enum Error {
        HitWall,
        StoreageFull,
        ScanFailed,
        ClientError,
        InvalidBot,
    }
    //
    pub trait Commands {
        fn locate(&self) -> Coords2D;
        fn go_north(&mut self) -> Result<(), Error>;
        fn go_south(&mut self) -> Result<(), Error>;
        fn go_west(&mut self) -> Result<(), Error>;
        fn go_east(&mut self) -> Result<(), Error>;

        fn scan_near(&self) -> Result<(), Error>;
        fn reset(&mut self) -> Result<(), Error>;
    }

    #[derive(Serialize, Deserialize)]
    pub struct MockBot {
        pub bot: String,
        pub location: Coords2D,
        call_count: HashMap<isize, i32>,
    }

    impl Default for MockBot {
        fn default() -> Self {
            Self {
                bot: "paul".to_string(),
                location: Coords2D { x: 0, y: 0 },
                call_count: HashMap::from([
                    (NORTH as isize, 0),
                    (WEST as isize, 0),
                    (SOUTH as isize, 0),
                    (EAST as isize, 0),
                ]),
            }
        }
    }

    impl MockBot {
        pub fn get_call_count(&self) -> &HashMap<isize, i32> {
            &self.call_count
        }

        pub fn from_file(path: PathBuf) -> Option<Self> {
            if let Ok(s) = fs::read_to_string(path) {
                let ds: Self = serde_json::from_str(&s).unwrap();
                return Some(ds);
            }
            None
        }
    }

    impl Commands for MockBot {
        fn locate(&self) -> Coords2D {
            self.location.clone()
        }

        fn go_east(&mut self) -> Result<(), Error> {
            debug!("go_east");
            Ok(())
        }

        fn go_north(&mut self) -> Result<(), Error> {
            debug!("go_north");
            Ok(())
        }

        fn go_south(&mut self) -> Result<(), Error> {
            debug!("go_south");
            Ok(())
        }

        fn go_west(&mut self) -> Result<(), Error> {
            debug!("go_west");
            Err(Error::HitWall)
        }

        fn scan_near(&self) -> Result<(), Error> {
            debug!("Scan Near");
            Err(Error::ScanFailed)
        }

        fn reset(&mut self) -> Result<(), Error> {
            debug!("Reset Bot");
            self.location = Coords2D::default();
            Ok(())
        }
    }
}
