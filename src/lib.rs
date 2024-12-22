pub mod cli;

pub mod warehouse;

pub mod bot {
    use log::debug;
    use std::collections::HashMap;

    pub mod rest;

    #[derive(Debug)]
    pub enum CommandError {
        HitWall,
        StoreageFull,
        ScanFailed,
        ClientError,
    }
    //
    pub trait Commands {
        fn go_north(&self) -> Result<(), CommandError>;
        fn go_south(&self) -> Result<(), CommandError>;
        fn go_west(&self) -> Result<(), CommandError>;
        fn go_east(&self) -> Result<(), CommandError>;

        fn scan_near(&self) -> Result<(), CommandError>;
        fn reset(&self) -> Result<(), CommandError>;
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
                    ("north".to_string(), 0),
                    ("west".to_string(), 0),
                    ("south".to_string(), 0),
                    ("east".to_string(), 0),
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
        fn go_east(&self) -> Result<(), CommandError> {
            debug!("go_east");
            Ok(())
        }

        fn go_north(&self) -> Result<(), CommandError> {
            debug!("go_north");
            Ok(())
        }

        fn go_south(&self) -> Result<(), CommandError> {
            debug!("go_south");
            Ok(())
        }

        fn go_west(&self) -> Result<(), CommandError> {
            debug!("go_west");
            Err(CommandError::HitWall)
        }

        fn scan_near(&self) -> Result<(), CommandError> {
            debug!("Scan Near");
            Err(CommandError::ScanFailed)
        }

        fn reset(&self) -> Result<(), CommandError> {
            debug!("Reset Bot");
            Ok(())
        }
    }
}
