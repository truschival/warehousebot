pub mod cli;

pub mod warehouse;

pub mod directions {
    pub const NORTH: &str = "north";
    pub const WEST: &str = "west";
    pub const EAST: &str = "east";
    pub const SOUTH: &str = "south";
}

pub mod bot {
    use crate::directions::{EAST, NORTH, SOUTH, WEST};
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
        call_count: HashMap<String, i32>,
    }

    impl Default for MockBot {
        fn default() -> Self {
            Self {
                bot: "paul".to_string(),
                call_count: HashMap::from([
                    (NORTH.to_string(), 0),
                    (WEST.to_string(), 0),
                    (SOUTH.to_string(), 0),
                    (EAST.to_string(), 0),
                ]),
            }
        }
    }

    impl MockBot {
        pub fn get_call_count(&self) -> &HashMap<String, i32> {
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
