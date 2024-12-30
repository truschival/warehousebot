pub mod cli;
pub mod cliplotter;
pub mod warehouse;
use log::{debug, info};
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::PathBuf};

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
    HitWall,
    StoreageFull,
    ScanFailed,
    ClientError,
    InvalidBot,
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

fn get_data_dir() -> PathBuf {
    if let Some(filepath) = dirs::data_dir() {
        filepath
    } else {
        log::error!("dirs::data_dir() failed - returning current_dir()!");
        std::env::current_dir().expect("current_dir() failed!")
    }
}

fn save_state<T: Serialize>(object: &T, filename: &str) -> Result<(), std::io::Error> {
    let serialized = serde_json::to_string(object).expect("cannot serialize object");
    let mut filepath = get_data_dir();

    if !filepath.exists() {
        match fs::create_dir_all(&filepath) {
            Ok(_) => info!("Created dir {:?}", &filepath),
            Err(e) => return Err(e),
        }
    }

    filepath.push(filename);
    debug!("Save state to {:?}", &filepath);
    fs::write(&filepath, serialized)
}

fn load_state<T: DeserializeOwned>(filename: &str) -> Option<T> {
    let mut filepath = get_data_dir();
    filepath.push(filename);
    log::debug!("Loading from {:?}", &filepath);
    match fs::read_to_string(filepath) {
        Ok(s) => {
            log::debug!("Read string from file");
            let wh = serde_json::from_str::<T>(s.as_str()).expect("Deserialization failed!");
            Some(wh)
        }
        Err(e) => {
            log::error!("Read failed {}", e.to_string());
            None
        }
    }
}

pub fn move_bot_in_warehouse(
    bot: &mut impl bot::Commands,
    direction: crate::Direction,
) -> Result<(), Error> {
    match direction {
        Direction::NORTH => {
            debug!("move north");
            bot.go_north()
        }
        Direction::EAST => {
            debug!("move east");
            bot.go_east()
        }
        Direction::SOUTH => {
            debug!("move south");
            bot.go_south()
        }
        Direction::WEST => {
            debug!("move west");
            bot.go_west()
        }
    }
}

pub fn explore_warehouse_manually(
    bot: &mut impl bot::Commands,
    warehouse: &mut warehouse::Warehouse,
    direction: crate::Direction,
) -> Result<(), Error> {
    if let Err(e) = move_bot_in_warehouse(bot, direction) {
        log::error!("Bot error occured: {:?}", e);
        return Err(e);
    }
    match bot.scan_near() {
        Ok(c) => {
            warehouse.insert_or_update_cell(bot.locate(), c);
            Ok(())
        }
        Err(e) => {
            log::error!("Bot error occured: {:?}", e);
            Err(e)
        }
    }
}

pub mod bot {
    use crate::{
        warehouse::{Cell, Coords2D},
        Direction::{EAST, NORTH, SOUTH, WEST},
        Error,
    };
    use log::debug;
    use serde::{Deserialize, Serialize};
    use std::{collections::HashMap, fs, path::PathBuf};

    pub mod rest;

    pub trait Commands {
        fn locate(&self) -> Coords2D;
        fn go_north(&mut self) -> Result<(), Error>;
        fn go_south(&mut self) -> Result<(), Error>;
        fn go_west(&mut self) -> Result<(), Error>;
        fn go_east(&mut self) -> Result<(), Error>;

        fn scan_near(&self) -> Result<Cell, Error>;
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
            self.location.x += 1;
            Ok(())
        }

        fn go_north(&mut self) -> Result<(), Error> {
            debug!("go_north");
            self.location.y += 1;
            Ok(())
        }

        fn go_south(&mut self) -> Result<(), Error> {
            debug!("go_south");
            self.location.y -= 1;
            Ok(())
        }

        fn go_west(&mut self) -> Result<(), Error> {
            debug!("go_west");
            self.location.x -= 1;
            Ok(())
        }

        fn scan_near(&self) -> Result<Cell, Error> {
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
