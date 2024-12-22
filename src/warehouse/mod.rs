use crate::Direction::{self, EAST, NORTH, SOUTH, WEST};
use crate::{direction_to_literal, literal_to_direction};
use log::debug;
use std::collections::HashMap;

#[derive(Default)]
pub struct Wall {}

#[derive(Debug, PartialEq)]
pub enum Error {
    CellInvalid,
    GoodsIncompatible,
    StorageExceeded,
}

#[derive(PartialEq, Debug)]
pub struct Coords2D(i32, i32);

#[derive(PartialEq, Debug)]
pub enum CellType {
    XCross,
    TCross,
    Corner,
    Hallway,
    DeadEnd,
}

pub struct Cell {
    pub pos: Coords2D,
    pub walls: HashMap<String, Wall>,
    shelf_inventory: Vec<String>,
    visited: bool,
    cell_type: CellType,
}

impl std::fmt::Display for Coords2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cell: ({}, {}) - Storage ({}/{}) - Goods: {}",
            self.pos.0,
            self.pos.1,
            self.occupied_storage(),
            self.storage_capacity(),
            self.stored_good_type()
        )
    }
}

impl Cell {
    pub fn new(pos: Coords2D) -> Self {
        debug!("New Cell at {}", &pos);
        Self {
            pos,
            walls: HashMap::new(),
            shelf_inventory: Vec::new(),
            visited: false,
            cell_type: CellType::XCross,
        }
    }

    pub fn add_wall(&mut self, side: Direction) {
        // I can't use enum as key in hashmap... convert back
        let side_str = direction_to_literal(&side);

        if self.walls.contains_key(&side_str) {
            panic!("You already have a wall on side: '{side_str}'!");
        }

        match self.walls.len() {
            0 => {
                debug!("No wall yet: XCross->TCross");
                self.cell_type = CellType::TCross;
            }

            1 => {
                let firstwall = self
                    .walls
                    .keys()
                    .next()
                    .expect("There should be a wall but is none");

                let firstwall = literal_to_direction(firstwall)
                    .expect("Hashmap key did not match valid direction!");

                // are we building a corner or hallway?
                match firstwall {
                    SOUTH | NORTH => match side {
                        WEST | EAST => {
                            debug!("Its a Corner!");
                            self.cell_type = CellType::Corner;
                        }
                        SOUTH | NORTH => {
                            debug!("Its a Hallway!");
                            self.cell_type = CellType::Hallway;
                        }
                    },
                    WEST | EAST => match side {
                        SOUTH | NORTH => {
                            debug!("Its a Corner!");
                            self.cell_type = CellType::Corner;
                        }
                        WEST | EAST => {
                            debug!("Its a Hallway!");
                            self.cell_type = CellType::Hallway;
                        }
                    },
                }
            }
            2 => {
                debug!("found 2 walls, adding 3rd it becomes a DeadEnd!");
                self.cell_type = CellType::DeadEnd
            }
            _ => panic!("Our cells can only have 3 walls!"),
        }
        self.walls.insert(side_str, Wall::default());
    }

    pub fn was_visited(&self) -> bool {
        self.visited
    }

    pub fn visit(&mut self) {
        self.visited = true;
    }

    pub fn occupied_storage(&self) -> usize {
        self.shelf_inventory.len()
    }

    pub fn storage_capacity(&self) -> usize {
        match self.cell_type {
            CellType::XCross => 4,
            CellType::TCross => 6,
            CellType::Hallway => 8,
            CellType::Corner => 9,
            CellType::DeadEnd => 12,
        }
    }

    pub fn put_good_on_shelf(&mut self, good: String) -> Result<usize, Error> {
        if !self.shelf_inventory.is_empty() && self.shelf_inventory[0] != good {
            return Err(Error::GoodsIncompatible);
        }
        if self.shelf_inventory.len() == self.storage_capacity() {
            return Err(Error::StorageExceeded);
        }
        self.shelf_inventory.push(good);
        Ok(self.occupied_storage())
    }

    pub fn stored_good_type(&self) -> &str {
        if let Some(good) = self.shelf_inventory.first() {
            good
        } else {
            "-"
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SOUTH_LIT;

    use super::*;

    #[test]
    fn test_new_cell() {
        let c = Cell::new(Coords2D(1, 2));
        assert_eq!(c.pos.0, 1);
        assert_eq!(c.pos.1, 2);
        assert_eq!(c.walls.len(), 0);
        assert!(!c.was_visited());
        assert_eq!(c.storage_capacity(), 4);
    }

    #[test]
    fn test_visit_cell() {
        let mut c = Cell::new(Coords2D(1, 2));
        c.visit();
        assert!(c.was_visited());
    }

    #[test]
    fn test_cell_add_walls_ok() {
        let mut c = Cell::new(Coords2D(1, 2));
        assert_eq!(c.cell_type, CellType::XCross);

        c.add_wall(SOUTH);
        assert_eq!(c.cell_type, CellType::TCross);
        assert_eq!(c.storage_capacity(), 6);

        c.add_wall(NORTH);
        assert_eq!(c.cell_type, CellType::Hallway);
        assert_eq!(c.storage_capacity(), 8);

        c.add_wall(EAST);
        assert_eq!(c.cell_type, CellType::DeadEnd);
        assert_eq!(c.storage_capacity(), 12);

        assert_eq!(c.walls.len(), 3);
        assert!(c.walls.contains_key(SOUTH_LIT))
    }

    #[test]
    fn test_cell_build_corner() {
        let mut c = Cell::new(Coords2D(1, 2));

        c.add_wall(SOUTH);
        assert_eq!(c.cell_type, CellType::TCross);
        assert_eq!(c.storage_capacity(), 6);

        c.add_wall(EAST);
        assert_eq!(c.cell_type, CellType::Corner);
        assert_eq!(c.storage_capacity(), 9);

        let mut c2 = Cell::new(Coords2D(2, 3));
        c2.add_wall(EAST);
        assert_eq!(c2.cell_type, CellType::TCross);
        assert_eq!(c2.storage_capacity(), 6);

        c2.add_wall(SOUTH);
        assert_eq!(c.cell_type, CellType::Corner);
        assert_eq!(c.storage_capacity(), 9);

        println!("{}", c);
    }

    #[test]
    #[should_panic]
    fn test_build_box_panic() {
        let mut c = Cell::new(Coords2D(1, 2));

        c.add_wall(WEST);
        assert_eq!(c.cell_type, CellType::TCross);
        assert_eq!(c.storage_capacity(), 6);

        c.add_wall(NORTH);
        assert_eq!(c.cell_type, CellType::Corner);
        assert_eq!(c.storage_capacity(), 9);

        c.add_wall(EAST);
        assert_eq!(c.cell_type, CellType::DeadEnd);
        assert_eq!(c.storage_capacity(), 12);

        c.add_wall(SOUTH);
    }

    #[test]
    #[should_panic]
    fn test_cell_add_wall_twice() {
        let mut c = Cell::new(Coords2D(1, 2));
        c.add_wall(SOUTH);
        c.add_wall(SOUTH);
    }

    #[test]
    fn test_add_items_ok() {
        let mut c = Cell::new(Coords2D(1, 2));
        assert_eq!(c.stored_good_type(), "-");
        assert_eq!(c.occupied_storage(), 0);
        assert_eq!(c.storage_capacity(), 4);
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert_eq!(c.occupied_storage(), 1);
        assert_eq!(c.stored_good_type(), "Kürbis");
    }

    #[test]
    fn test_add_items_fail_full() {
        let mut c = Cell::new(Coords2D(1, 2));
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_err());
        assert_eq!(c.occupied_storage(), c.storage_capacity());
    }

    #[test]
    fn test_add_items_fail_mixed_goods() {
        let mut c = Cell::new(Coords2D(1, 2));
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        let e = c.put_good_on_shelf("Milch".to_string());
        assert!(e.is_err());
        assert_eq!(e.unwrap_err(), Error::GoodsIncompatible);
        assert_eq!(c.occupied_storage(), 1);
    }
}
