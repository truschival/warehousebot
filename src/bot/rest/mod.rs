use super::{Commands, Error, FarScanResult};
use crate::{
    warehouse::{Cell, Coords2D},
    Direction,
};
use log::{debug, error, warn};
use reqwest;
use serde::{Deserialize, Serialize};

//{
// "field_info": {
//     "id": "Field",
//     "max_capacity": 12,
//     "shelf_inventory": [
//       "beer",
//       "beer"
//     ],
//     "walls": {
//       "east": true,
//       "north": true,
//       "south": true,
//       "west": false
//     }
//   },
//   "request_status": "ok"
// }

mod rest_responses {
    use serde::Deserialize;
    #[derive(Deserialize, Debug)]
    pub struct Walls {
        pub east: bool,
        pub north: bool,
        pub south: bool,
        pub west: bool,
    }

    #[derive(Deserialize, Debug)]
    pub struct CellInfo {
        pub shelf_inventory: Vec<String>,
        pub walls: Walls,
    }

    #[derive(Deserialize, Debug)]
    pub struct ScanNearResponse {
        pub field_info: CellInfo,
        // WHY!?!?! We already get a status in the HTTP response
        // Nobody needs request_status: String,
    }
    #[derive(Deserialize, Debug)]
    pub struct FarScanInfo {
        pub north: i32,
        pub east: i32,
        pub west: i32,
        pub south: i32,
    }
    #[derive(Deserialize, Debug)]
    pub struct ScanFarResponse {
        pub scan_info: FarScanInfo,
        // WHY!?!?! We already get a status in the HTTP response
        // Nobody needs request_status: String,
    }
}

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

    fn direction_to_literal(direction: &crate::Direction) -> &str {
        match direction {
            Direction::NORTH => stringify!(north),
            Direction::WEST => stringify!(west),
            Direction::SOUTH => stringify!(south),
            Direction::EAST => stringify!(east),
        }
    }

    fn navigate(&mut self, direction: Direction) -> Result<(), Error> {
        let client = reqwest::blocking::Client::new();

        let res = client
            .put(format!(
                "{}/{}/move/{}",
                self.base_url,
                self.bot,
                Self::direction_to_literal(&direction)
            ))
            .send();
        match res {
            Ok(s) => {
                debug!("HTTP Status: {:?}", s.status());
                match s.status().as_u16() {
                    200 => {
                        self.location = self.location.neighbor_coords(&direction);
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

    pub fn scan_near(&self) -> Result<Cell, Error> {
        let res = reqwest::blocking::get(format!("{}/{}/scan/near", self.base_url, self.bot));

        match res {
            Ok(s) => {
                debug!("HTTP Status: {:?}", s.status());
                match s.status().as_u16() {
                    200 => {
                        if let Ok(snr) = s.json::<rest_responses::ScanNearResponse>() {
                            Ok(scan_near_to_cell(snr.field_info))
                        } else {
                            log::error!("Deserialization of scan_near response failed!");
                            Err(Error::ScanFailed)
                        }
                    }
                    404 => {
                        warn!("No such bot!");
                        Err(Error::InvalidBot)
                    }
                    _ => {
                        error!("Got unexpected HTTP status!");
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

    pub fn scan_far(&self) -> Result<FarScanResult, Error> {
        let res = reqwest::blocking::get(format!("{}/{}/scan/far", self.base_url, self.bot));

        match res {
            Ok(s) => {
                debug!("HTTP Status: {:?}", s.status());
                match s.status().as_u16() {
                    200 => {
                        if let Ok(sfr) = s.json::<rest_responses::ScanFarResponse>() {
                            let mut far_scan_result: std::collections::HashMap<Direction, i32> =
                                FarScanResult::with_capacity(4);
                            far_scan_result.insert(Direction::NORTH, sfr.scan_info.north);
                            far_scan_result.insert(Direction::EAST, sfr.scan_info.east);
                            far_scan_result.insert(Direction::SOUTH, sfr.scan_info.south);
                            far_scan_result.insert(Direction::WEST, sfr.scan_info.west);

                            Ok(far_scan_result)
                        } else {
                            log::error!("Deserialization of scan_near response failed!");
                            Err(Error::ScanFailed)
                        }
                    }
                    404 => {
                        warn!("No such bot!");
                        Err(Error::InvalidBot)
                    }
                    _ => {
                        error!("Got unexpected HTTP status!");
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

fn scan_near_to_cell(ci: rest_responses::CellInfo) -> Cell {
    let mut c = Cell::default();
    let walls = ci.walls;
    if walls.east {
        _ = c.add_wall(&Direction::EAST);
    }
    if walls.north {
        _ = c.add_wall(&Direction::NORTH);
    }
    if walls.west {
        _ = c.add_wall(&Direction::WEST);
    }
    if walls.south {
        _ = c.add_wall(&Direction::SOUTH);
    }

    log::debug!(" goods stored: {:?}", &ci.shelf_inventory);
    for item in ci.shelf_inventory {
        _ = c.put_good_on_shelf(item);
    }
    c
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

    fn scan_near(&self) -> Result<Cell, Error> {
        debug!("Scan Near");
        self.scan_near()
    }

    fn scan_far(&self) -> Result<super::FarScanResult, Error> {
        debug!("Scan far");
        self.scan_far()
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
