pub mod cli;
pub mod warehouse;

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

pub mod bot {
    use crate::Direction::{EAST, NORTH, SOUTH, WEST};
    use log::debug;
    use std::collections::HashMap;

    pub mod rest;

    #[derive(Debug, PartialEq)]
    pub enum Error {
        HitWall,
        StoreageFull,
        ScanFailed,
        ClientError,
    }
    //
    pub trait Commands {
        fn go_north(&self) -> Result<(), Error>;
        fn go_south(&self) -> Result<(), Error>;
        fn go_west(&self) -> Result<(), Error>;
        fn go_east(&self) -> Result<(), Error>;

        fn scan_near(&self) -> Result<(), Error>;
        fn reset(&self) -> Result<(), Error>;
    }

    pub struct MockBot {
        pub bot: String,
        call_count: HashMap<isize, i32>,
    }

    impl Default for MockBot {
        fn default() -> Self {
            Self {
                bot: "paul".to_string(),
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
    }

    impl Commands for MockBot {
        fn go_east(&self) -> Result<(), Error> {
            debug!("go_east");
            Ok(())
        }

        fn go_north(&self) -> Result<(), Error> {
            debug!("go_north");
            Ok(())
        }

        fn go_south(&self) -> Result<(), Error> {
            debug!("go_south");
            Ok(())
        }

        fn go_west(&self) -> Result<(), Error> {
            debug!("go_west");
            Err(Error::HitWall)
        }

        fn scan_near(&self) -> Result<(), Error> {
            debug!("Scan Near");
            Err(Error::ScanFailed)
        }

        fn reset(&self) -> Result<(), Error> {
            debug!("Reset Bot");
            Ok(())
        }
    }
}
