use super::{CommandError, Commands};
use log::{debug, error, warn};
use reqwest;

pub struct RestBot {
    pub bot: String,
    base_url: String,
}

impl RestBot {
    pub fn new(bot: String, base_url: String) -> Self {
        Self { bot, base_url }
    }

    fn navigate(&self, direction: &str) -> Result<(), CommandError> {
        let client = reqwest::blocking::Client::new();
        let res = client
            .put(format!("{}/{}/move/{direction}", self.base_url, self.bot))
            .send();
        match res {
            Ok(s) => {
                debug!("HTTP Status: {:?}", s.status());
                match s.status().as_u16() {
                    200 => Ok(()),
                    404 => {
                        warn!("No such bot!");
                        Err(CommandError::ClientError)
                    }
                    405 => Err(CommandError::HitWall),
                    _ => Err(CommandError::ClientError),
                }
            }
            Err(e) => {
                error!("Error occured: {:?} - {}", e.status(), e.to_string());
                Err(CommandError::ClientError)
            }
        }
    }
}

impl Default for RestBot {
    fn default() -> Self {
        Self::new(
            "john".to_string(),
            "http://springschool-lb-54580289.eu-central-1.elb.amazonaws.com/api/bot".to_string(),
        )
    }
}

impl Commands for RestBot {
    fn go_east(&self) -> Result<(), CommandError> {
        debug!("go_east");
        self.navigate("east")
    }

    fn go_north(&self) -> Result<(), CommandError> {
        debug!("go_north");
        self.navigate("north")
    }

    fn go_south(&self) -> Result<(), CommandError> {
        debug!("go_south");
        self.navigate("south")
    }

    fn go_west(&self) -> Result<(), CommandError> {
        debug!("go_west");
        self.navigate("west")
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
