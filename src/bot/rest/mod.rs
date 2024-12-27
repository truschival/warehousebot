use super::{Commands, Error};
use crate::{direction_to_literal, warehouse::Coords2D, Direction};
use log::{debug, error, warn};
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RestBot {
    location: Coords2D,
    pub bot: String,
    base_url: String,
}

impl RestBot {
    pub fn new(bot: String, base_url: String) -> Self {
        Self {
            bot,
            location: Coords2D { x: 0, y: 0 },
            base_url,
        }
    }

    fn navigate(&mut self, direction: Direction) -> Result<(), Error> {
        let client = reqwest::blocking::Client::new();

        let res = client
            .put(format!(
                "{}/{}/move/{}",
                self.base_url,
                self.bot,
                direction_to_literal(&direction)
            ))
            .send();
        match res {
            Ok(s) => {
                debug!("HTTP Status: {:?}", s.status());
                match s.status().as_u16() {
                    200 => {
                        self.location = self.location.go(direction);
                        Ok(())
                    }
                    404 => {
                        warn!("No such bot!");
                        Err(Error::InvalidBot)
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
    fn locate(&self) -> crate::warehouse::Coords2D {
        self.location.clone()
    }
    fn go_east(&mut self) -> Result<(), Error> {
        debug!("go_east");
        self.navigate(Direction::EAST)
    }

    fn go_north(&mut self) -> Result<(), Error> {
        debug!("go_north");
        self.navigate(Direction::NORTH)
    }

    fn go_south(&mut self) -> Result<(), Error> {
        debug!("go_south");
        self.navigate(Direction::SOUTH)
    }

    fn go_west(&mut self) -> Result<(), Error> {
        debug!("go_west");
        self.navigate(Direction::WEST)
    }

    fn scan_near(&self) -> Result<(), Error> {
        debug!("Scan Near");
        Err(Error::ScanFailed)
    }

    fn reset(&mut self) -> Result<(), Error> {
        debug!("Reset Bot");
        let client = reqwest::blocking::Client::new();

        let res = client
            .put(format!(
                "{}/{}/reset",
                "http://springschool-lb-54580289.eu-central-1.elb.amazonaws.com/api/james/",
                self.bot
            ))
            .send();
        match res {
            Ok(s) => {
                debug!("HTTP Status: {:?}", s.status());
                match s.status().as_u16() {
                    200 => {
                        self.location = Coords2D::default();
                        Ok(())
                    }
                    404 => {
                        warn!("No such bot!");
                        Err(Error::InvalidBot)
                    }
                    _ => {
                        warn!("Other weird error occured!");
                        Err(Error::ClientError)
                    }
                }
            }
            Err(e) => {
                error!("Error occured: {:?} - {}", e.status(), e.to_string());
                Err(Error::ClientError)
            }
        }
    }
}
