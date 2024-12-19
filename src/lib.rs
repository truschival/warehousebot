pub mod cli;

pub mod botcommands {
    use log::debug;
    #[derive(Debug)]
    pub enum CommandError {
        HitWall,
        StoreageFull,
    }
    //
    pub trait BotCommands {
        fn go_north(&self) -> Result<(), CommandError>;
        fn go_south(&self) -> Result<(), CommandError>;
        fn go_west(&self) -> Result<(), CommandError>;
        fn go_east(&self) -> Result<(), CommandError>;
    }

    pub struct MockHandler {}

    impl BotCommands for MockHandler {
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
    }
}
