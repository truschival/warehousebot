use log::{error, info, warn};

pub mod botcommands {

    #[derive(Debug)]
    pub enum CommandError {
        HitWall,
        StoreageFull,
    }

    //
    pub trait BotCommands {
        fn go_north() -> Result<(), CommandError>;
        fn go_south() -> Result<(), CommandError>;
    }

    #[derive(Debug)]
    pub enum CliError {
        CommandUnknown,
        CommandNotImplemented,
    }

    pub fn command_for_string(cmd: &str) -> Result<impl Command, CliError> {
        let cmd = cmd.to_uppercase();
        match cmd.trim() {
            "NORTH" => {
                log::debug!("norden");
                Ok(GoSouth::new())
            }
            "SOUTH" => {
                log::debug!("Süden");
                Ok(GoSouth::new())
            }
            "WEST" | "East" => Err(CliError::CommandNotImplemented),
            "NEAR" | "FAR" => Err(CliError::CommandNotImplemented),
            _ => Err(CliError::CommandUnknown),
        }
    }

    pub trait Command {
        fn execute(&self);
        fn info(&self);
    }

    pub struct GoNorth {
        info: String,
    }

    impl Default for GoNorth {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GoNorth {
        pub fn new() -> Self {
            Self {
                info: "Going North!".to_string(),
            }
        }
    }

    impl Command for GoNorth {
        fn execute(&self) {
            self.info();
        }
        fn info(&self) {
            println!("{}", self.info);
        }
    }

    // Going South
    #[derive(Debug)]
    pub struct GoSouth {
        pub info: String,
    }

    impl Command for GoSouth {
        fn execute(&self) {
            self.info();
        }
        fn info(&self) {
            println!("{}", self.info);
        }
    }

    impl Default for GoSouth {
        fn default() -> Self {
            Self::new()
        }
    }

    impl GoSouth {
        pub fn new() -> Self {
            Self {
                info: "Going South".to_string(),
            }
        }
    }
}
