pub mod cli;
pub mod cliplotter;
pub mod warehouse;
use bot::FarScanResult;
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use warehouse::{Cell, Coords2D, Warehouse};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    NORTH = 0,
    EAST = 1,
    SOUTH = 2,
    WEST = 3,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::NORTH => Direction::SOUTH,
            Direction::EAST => Direction::WEST,
            Direction::SOUTH => Direction::NORTH,
            Direction::WEST => Direction::EAST,
        }
    }
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

fn get_data_dir() -> std::path::PathBuf {
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
        log::error!("Bot error occured moving: {:?}", e);
        return Err(e);
    }

    match bot.scan_far() {
        Err(e) => {
            log::error!("Bot error occured scanning far: {:?}", e);
            Err(e)
        }
        Ok(farscan) => match update_warehouse_from_scan_far(warehouse, farscan, &bot.locate()) {
            Ok(_) => update_warehouse_at_bot_postion(bot, warehouse),
            Err(e) => Err(e),
        },
    }
}

pub fn update_warehouse_at_bot_postion(
    bot: &mut impl bot::Commands,
    warehouse: &mut Warehouse,
) -> Result<(), Error> {
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

pub fn get_coords_in_direction(
    start: &Coords2D,
    direction: &crate::Direction,
    steps: Option<&i32>,
) -> Result<Vec<Coords2D>, Error> {
    let mut ret = Vec::<Coords2D>::new();

    if let Some(steps) = steps {
        log::debug!("{} steps to {:?}", steps, &direction);
        if *steps < 0 {
            log::error!("Steps < 0 don't make sense!");
            return Err(Error::ClientError);
        }
        if *steps > 0 {
            for i in 1..=(*steps) {
                ret.push(start.neighbor_coords_distance(direction, i));
            }
        }
    }
    Ok(ret)
}

pub fn update_warehouse_from_scan_far(
    warehouse: &mut Warehouse,
    scaninfo: FarScanResult,
    botcoords: &Coords2D,
) -> Result<(), Error> {
    for dir in [
        Direction::NORTH,
        Direction::EAST,
        Direction::SOUTH,
        Direction::WEST,
    ] {
        let new_coords = get_coords_in_direction(botcoords, &dir, scaninfo.get(&dir))?;
        for i in &new_coords {
            let mut c = Cell::default();
            if i == new_coords.last().unwrap() {
                _ = c.add_wall(&dir);
            }
            warehouse.add_far_scan_cell(i.clone(), c);
        }
    }
    Ok(())
}

pub mod bot {
    use crate::{
        warehouse::{Cell, Coords2D},
        Direction, Error,
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
        fn scan_far(&self) -> Result<FarScanResult, Error>;
        fn reset(&mut self) -> Result<(), Error>;
    }

    pub type FarScanResult = HashMap<Direction, i32>;

    #[derive(Serialize, Deserialize)]
    pub struct MockBot {
        pub bot: String,
        pub location: Coords2D,
        call_count: HashMap<Direction, i32>,
    }

    impl Default for MockBot {
        fn default() -> Self {
            Self {
                bot: "paul".to_string(),
                location: Coords2D { x: 0, y: 0 },
                call_count: HashMap::from([
                    (Direction::NORTH, 0),
                    (Direction::WEST, 0),
                    (Direction::SOUTH, 0),
                    (Direction::EAST, 0),
                ]),
            }
        }
    }

    impl MockBot {
        pub fn get_call_count(&self) -> &HashMap<Direction, i32> {
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

        fn scan_far(&self) -> Result<FarScanResult, Error> {
            debug!("scan far");
            Err(Error::ScanFailed)
        }

        fn reset(&mut self) -> Result<(), Error> {
            debug!("Reset Bot");
            self.location = Coords2D::default();
            Ok(())
        }
    }
}
