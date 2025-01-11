pub mod cli;
pub mod cliplotter;
pub mod warehouse;
use bot::FarScanResult;
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use warehouse::{Cell, Warehouse};

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

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Coords2D {
    pub x: i32,
    pub y: i32,
}
impl Coords2D {
    pub fn neighbor_coords(&self, dir: &Direction) -> Self {
        self.neighbor_coords_distance(dir, 1)
    }

    pub fn neighbor_coords_distance(&self, dir: &Direction, steps: i32) -> Self {
        if steps < 0 {
            warn!("consider positive steps for neighbor_coords and use opposite direction");
        }
        match dir {
            Direction::NORTH => Self {
                x: self.x,
                y: self.y - steps,
            },
            Direction::EAST => Self {
                x: self.x + steps,
                y: self.y,
            },
            Direction::SOUTH => Self {
                x: self.x,
                y: self.y + steps,
            },
            Direction::WEST => Self {
                x: self.x - steps,
                y: self.y,
            },
        }
    }

    fn get_north_west(pos: &Self) -> Self {
        Coords2D {
            x: pos.x - 1,
            y: pos.y - 1,
        }
    }

    pub fn from_string(stringrep: &str) -> Option<Self> {
        let tokens: Vec<&str> = stringrep.split(",").collect();
        if tokens.len() < 2 {
            return None;
        }

        Some(Self {
            x: tokens[0]
                .parse()
                .unwrap_or_else(|_| panic!("expected i32 for yx got {}", tokens[0])),
            y: tokens[1]
                .parse()
                .unwrap_or_else(|_| panic!("expected i32 for y got {}", tokens[1])),
        })
    }
}

impl std::fmt::Display for Coords2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
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
    InvalidArgument,
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
    warehouse: &Warehouse,
    dir: crate::Direction,
) -> Result<(), Error> {
    let pos = bot.locate();
    if let Some(c) = warehouse.get_cell(&pos) {
        if c.has_wall(&dir) {
            log::info!("Bot will not move '{:?}' - there is a wall.", &dir);
            return Err(Error::HitWall);
        }
    } else {
        panic!("Bot is not in warehouse!");
    }

    match dir {
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
    if let Err(e) = move_bot_in_warehouse(bot, warehouse, direction) {
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
            debug!("update warehouse with results form scan_near(): {}", &c);
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
            return Err(Error::InvalidArgument);
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
    log::debug!("update_warehouse_form_scan_far {:?}", &scaninfo);

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
    use crate::{warehouse::Cell, Coords2D, Direction, Error};
    use std::collections::HashMap;

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
}
#[cfg(test)]
mod tests {
    use super::bot::*;
    use super::*;
    use std::collections::HashMap;
    use Direction::{EAST, NORTH, SOUTH, WEST};

    #[derive(Serialize, Deserialize)]
    pub struct MockBot {
        pub bot: String,
        pub location: Coords2D,
        call_count: HashMap<Direction, i32>,
        pub enable_error: bool,
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
                enable_error: false,
            }
        }
    }

    impl MockBot {
        pub fn get_call_count(&self) -> &HashMap<Direction, i32> {
            &self.call_count
        }
    }

    impl Commands for MockBot {
        fn locate(&self) -> Coords2D {
            self.location.clone()
        }

        fn go_east(&mut self) -> Result<(), Error> {
            debug!("go_east");
            self.location.x += 1;
            *self.call_count.get_mut(&EAST).unwrap() += 1;
            Ok(())
        }

        fn go_north(&mut self) -> Result<(), Error> {
            debug!("go_north");
            self.location.y -= 1;
            *self.call_count.get_mut(&NORTH).unwrap() += 1;
            Ok(())
        }

        fn go_south(&mut self) -> Result<(), Error> {
            debug!("go_south");
            self.location.y += 1;
            *self.call_count.get_mut(&SOUTH).unwrap() += 1;
            Ok(())
        }

        fn go_west(&mut self) -> Result<(), Error> {
            debug!("go_west");
            self.location.x -= 1;
            *self.call_count.get_mut(&WEST).unwrap() += 1;
            Ok(())
        }

        fn scan_near(&self) -> Result<Cell, Error> {
            debug!("Scan Near");
            Err(Error::ScanFailed)
        }

        fn scan_far(&self) -> Result<FarScanResult, Error> {
            debug!("scan far");
            if !self.enable_error {
                let mut scan = FarScanResult::new();
                scan.insert(NORTH, 2);
                scan.insert(EAST, 1);
                scan.insert(SOUTH, 1);
                scan.insert(WEST, 0);
                Ok(scan)
            } else {
                Err(Error::ScanFailed)
            }
        }

        fn reset(&mut self) -> Result<(), Error> {
            debug!("Reset Bot");
            self.location = Coords2D::default();
            Ok(())
        }
    }

    fn some_warehouse() -> Warehouse {
        //     0 1 2
        //    +-+-+-+
        // 0  |     |
        //    +     +
        // 1  |     |   // no cell in the middle
        //    +     +
        // 2  |     |
        //    +-+-+-+
        //
        let mut wh = Warehouse::default();
        // Row 1
        let mut cell = Cell::default();
        cell.add_wall(&NORTH).expect("Should be able to add wall");
        cell.add_wall(&WEST).expect("Should be able to add wall");
        wh.insert_or_update_cell(Coords2D { x: 0, y: 0 }, cell);
        let mut cell = Cell::default();
        cell.add_wall(&NORTH).expect("Should be able to add wall");
        wh.insert_or_update_cell(Coords2D { x: 1, y: 0 }, cell);
        let mut cell = Cell::default();
        cell.add_wall(&EAST).expect("Should be able to add wall");
        cell.add_wall(&NORTH).expect("Should be able to add wall");
        wh.insert_or_update_cell(Coords2D { x: 2, y: 0 }, cell);
        // Row2
        let mut cell = Cell::default();
        _ = cell.add_wall(&WEST);
        wh.insert_or_update_cell(Coords2D { x: 0, y: 1 }, cell);
        // let cell = Cell::default();
        // wh.insert_or_update_cell(Coords2D { x: 1, y: 1 }, cell);
        let mut cell = Cell::default();
        _ = cell.add_wall(&EAST);
        wh.insert_or_update_cell(Coords2D { x: 2, y: 1 }, cell);
        // Row 3
        let mut cell = Cell::default();
        _ = cell.add_wall(&SOUTH);
        _ = cell.add_wall(&WEST);
        wh.insert_or_update_cell(Coords2D { x: 0, y: 2 }, cell);
        let mut cell = Cell::default();
        _ = cell.add_wall(&SOUTH);
        wh.insert_or_update_cell(Coords2D { x: 1, y: 2 }, cell);
        let mut cell = Cell::default();
        _ = cell.add_wall(&SOUTH);
        _ = cell.add_wall(&NORTH);
        wh.insert_or_update_cell(Coords2D { x: 2, y: 2 }, cell);
        wh
    }

    #[test]
    fn test_move_bot() {
        let wh = some_warehouse();
        let mut bot = MockBot::default();
        // Should not move west
        assert!(move_bot_in_warehouse(&mut bot, &wh, WEST).is_err());
        assert_eq!(bot.get_call_count().get(&WEST).unwrap(), &0);
        // East should work
        assert!(move_bot_in_warehouse(&mut bot, &wh, EAST).is_ok());
        assert_eq!(bot.get_call_count().get(&EAST).unwrap(), &1);
        assert_eq!(bot.locate(), Coords2D { x: 1, y: 0 });
        // East should work
        assert!(move_bot_in_warehouse(&mut bot, &wh, EAST).is_ok());
        assert_eq!(bot.get_call_count().get(&EAST).unwrap(), &2);
        assert_eq!(bot.locate(), Coords2D { x: 2, y: 0 });
        // South should work
        assert!(move_bot_in_warehouse(&mut bot, &wh, SOUTH).is_ok());
        assert_eq!(bot.get_call_count().get(&SOUTH).unwrap(), &1);
        assert_eq!(bot.locate(), Coords2D { x: 2, y: 1 });
        // BAck North should work
        assert!(move_bot_in_warehouse(&mut bot, &wh, NORTH).is_ok());
        assert_eq!(bot.get_call_count().get(&NORTH).unwrap(), &1);
        assert_eq!(bot.locate(), Coords2D { x: 2, y: 0 });
        // West should work
        assert!(move_bot_in_warehouse(&mut bot, &wh, WEST).is_ok());
        assert_eq!(bot.get_call_count().get(&WEST).unwrap(), &1);
        assert_eq!(bot.locate(), Coords2D { x: 1, y: 0 });
    }

    #[test]
    fn test_update_warehouse() {
        let mut wh = some_warehouse();
        assert!(!wh
            .get_cell(&Coords2D { x: 1, y: 0 })
            .unwrap()
            .has_wall(&SOUTH));
        assert!(!wh
            .get_cell(&Coords2D { x: 0, y: 1 })
            .unwrap()
            .has_wall(&EAST));
        assert!(!wh
            .get_cell(&Coords2D { x: 2, y: 1 })
            .unwrap()
            .has_wall(&WEST));
        let mut cell = Cell::default();
        _ = cell.add_wall(&NORTH);
        _ = cell.add_wall(&EAST);
        _ = cell.add_wall(&WEST);
        wh.insert_or_update_cell(Coords2D { x: 1, y: 1 }, cell);
        // Neighbor cells should have been updated
        assert!(wh
            .get_cell(&Coords2D { x: 1, y: 0 })
            .unwrap()
            .has_wall(&SOUTH));
        assert!(wh
            .get_cell(&Coords2D { x: 0, y: 1 })
            .unwrap()
            .has_wall(&EAST));
        assert!(wh
            .get_cell(&Coords2D { x: 2, y: 1 })
            .unwrap()
            .has_wall(&WEST));
        // Cell below should not have a north wall
        assert!(!wh
            .get_cell(&Coords2D { x: 2, y: 2 })
            .unwrap()
            .has_wall(&WEST));
    }

    #[test]
    fn test_get_neighbor_coords_single_step() {
        let pos = Coords2D { x: 1, y: -1 };
        let south = get_coords_in_direction(&pos, &SOUTH, Some(&1));
        let south = south.unwrap();
        assert_eq!(south.len(), 1);
        assert_eq!(south.get(0).unwrap(), &Coords2D { x: 1, y: 0 });

        let north = get_coords_in_direction(&pos, &NORTH, Some(&1));
        let north = north.unwrap();
        assert_eq!(north.len(), 1);
        assert_eq!(north.get(0).unwrap(), &Coords2D { x: 1, y: -2 });

        let east = get_coords_in_direction(&pos, &EAST, Some(&1));
        let east = east.unwrap();
        assert_eq!(east.len(), 1);
        assert_eq!(east.get(0).unwrap(), &Coords2D { x: 2, y: -1 });

        let west = get_coords_in_direction(&pos, &WEST, Some(&1));
        let west = west.unwrap();
        assert_eq!(west.len(), 1);
        assert_eq!(west.get(0).unwrap(), &Coords2D { x: 0, y: -1 });
    }

    #[test]
    fn test_update_from_far_scan(){
        let mut wh = Warehouse::default();
        let mut cell = Cell::default();
        _ = cell.add_wall(&WEST);
        let pos = Coords2D{x:0,y:0};
        wh.insert_or_update_cell(pos.clone(), cell);
        let mut scan = FarScanResult::new();
        scan.insert(NORTH, 2);
        scan.insert(EAST, 1);
        scan.insert(SOUTH, 1);
        scan.insert(WEST, 0);
        
        update_warehouse_from_scan_far(&mut wh, scan, &pos).unwrap();
        assert!(wh.get_cell(&Coords2D { x: 0, y: -2 }).is_some());
        assert!(wh.get_cell(&Coords2D { x: 0, y: -1 }).is_some());
        assert!(!wh.get_cell(&Coords2D { x: 0, y: -1 }).unwrap().has_wall(&NORTH));
        assert!(wh.get_cell(&Coords2D { x: 0, y: -2 }).unwrap().has_wall(&NORTH));

        assert!(wh.get_cell(&Coords2D { x: 1, y: 0 }).is_some());
        assert!(wh.get_cell(&Coords2D { x: 1, y: 0 }).unwrap().has_wall(&EAST));

        assert!(wh.get_cell(&Coords2D { x: 0, y: 1 }).is_some());
        assert!(wh.get_cell(&Coords2D { x: 0, y: 1 }).unwrap().has_wall(&SOUTH));
    }

    #[test]
    fn test_get_neighbor_coords_multi_step() {
        let pos = Coords2D { x: 1, y: -1 };
        let south = get_coords_in_direction(&pos, &SOUTH, Some(&2));
        let south = south.unwrap();
        assert_eq!(south.len(), 2);
        assert_eq!(south.get(0).unwrap(), &Coords2D { x: 1, y: 0 });
        assert_eq!(south.get(1).unwrap(), &Coords2D { x: 1, y: 1 });

        let north = get_coords_in_direction(&pos, &NORTH, Some(&1));
        let north = north.unwrap();
        assert_eq!(north.len(), 1);
        assert_eq!(north.get(0).unwrap(), &Coords2D { x: 1, y: -2 });
    }

    #[test]
    fn test_get_neighbor_coords_negative_step() {
        let pos = Coords2D { x: 1, y: -1 };
        let south = get_coords_in_direction(&pos, &SOUTH, Some(&-3));
        assert!(south.is_err());
    }

    #[test]
    fn test_get_north_west() {
        let pos = Coords2D { x: 1, y: 1 };
        assert_eq!(Coords2D::get_north_west(&pos), Coords2D::default());
    }

    #[test]
    fn test_get_opposite_direction() {
        assert_eq!(SOUTH, Direction::opposite(&NORTH));
        assert_eq!(NORTH, Direction::opposite(&SOUTH));
        assert_eq!(EAST, Direction::opposite(&WEST));
        assert_eq!(WEST, Direction::opposite(&EAST));
    }

    #[test]
    fn test_fill_neighbor_walls_from_new_cell() {
        let mut warehouse = Warehouse::default();
        warehouse.insert_or_update_cell(Coords2D { x: -2, y: 0 }, Cell::default()); // north west
        warehouse.insert_or_update_cell(Coords2D { x: -2, y: 1 }, Cell::default()); // west

        warehouse.insert_or_update_cell(Coords2D { x: -1, y: 0 }, Cell::default()); //north
        warehouse.insert_or_update_cell(Coords2D { x: -1, y: 2 }, Cell::default()); //south

        warehouse.insert_or_update_cell(Coords2D { x: 0, y: 1 }, Cell::default()); //east

        let pos = Coords2D { x: -1, y: 1 };
        let mut cell = Cell::default();
        _ = cell.add_wall(&EAST);
        _ = cell.add_wall(&WEST);
        warehouse.insert_or_update_cell(pos, cell);

        // corner has no walls
        let corner = warehouse.get_cell(&Coords2D { x: -2, y: 0 }).unwrap();
        assert!(corner.get_walls().is_empty());
        // West neighbor should have east wall
        let west_neighbor = warehouse.get_cell(&Coords2D { x: -2, y: 1 }).unwrap();
        assert_eq!(west_neighbor.get_walls().len(), 1);
        assert!(west_neighbor.has_wall(&EAST));

        // north should also have no walls
        let north_neighbor = warehouse.get_cell(&Coords2D { x: -1, y: 0 }).unwrap();
        assert!(north_neighbor.get_walls().is_empty());

        // north should also have no walls
        let south_neighbor = warehouse.get_cell(&Coords2D { x: -1, y: 2 }).unwrap();
        assert!(south_neighbor.get_walls().is_empty());

        let east_neighbor = warehouse.get_cell(&Coords2D { x: 0, y: 1 }).unwrap();
        assert_eq!(east_neighbor.get_walls().len(), 1);
        assert!(east_neighbor.has_wall(&WEST));
    }
}
