use super::{Commands, Error};
use crate::directions::{EAST, NORTH, SOUTH, WEST};
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

    fn navigate(&self, direction: &str) -> Result<(), Error> {
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
                        Err(Error::ClientError)
                    }
                    405 => Err(Error::HitWall),
                    _ => Err(Error::ClientError),
                }
            }
            Err(e) => {
                error!("Error occured: {:?} - {}", e.status(), e.to_string());
                Err(Error::ClientError)
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
    fn go_east(&self) -> Result<(), Error> {
        debug!("go_east");
        self.navigate(EAST)
    }

    fn go_north(&self) -> Result<(), Error> {
        debug!("go_north");
        self.navigate(NORTH)
    }

    fn go_south(&self) -> Result<(), Error> {
        debug!("go_south");
        self.navigate(SOUTH)
    }

    fn go_west(&self) -> Result<(), Error> {
        debug!("go_west");
        self.navigate(WEST)
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
