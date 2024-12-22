use log::debug;
use std::collections::HashMap;

#[derive(Default)]
pub struct Wall {}

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
    pub goods_stored: String,
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
            self.space_allocated(),
            self.capacity(),
            self.goods_stored
        )
    }
}

impl Cell {
    pub fn new(pos: Coords2D) -> Self {
        debug!("New Cell at {}", &pos);
        Self {
            pos,
            walls: HashMap::new(),
            goods_stored: String::new(),
            visited: false,
            cell_type: CellType::XCross,
        }
    }

    pub fn add_wall(&mut self, side: &str) {
        if self.walls.contains_key(side) {
            panic!("You already have a wall on side: '{side}'!");
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
                    .expect("There should be a wall but is none")
                    .as_str();
                debug!("Found one wall on the {} side, adding {}", firstwall, side);
                // are we building a corner or hallway?
                match firstwall {
                    "south" | "north" => match side {
                        "west" | "east" => {
                            debug!("Its a Corner!");
                            self.cell_type = CellType::Corner;
                        }
                        "south" | "north" => {
                            debug!("Its a Hallway!");
                            self.cell_type = CellType::Hallway;
                        }
                        _ => panic!("illegal side for wall: {side}"),
                    },
                    "west" | "east" => match side {
                        "south" | "north" => {
                            debug!("Its a Corner!");
                            self.cell_type = CellType::Corner;
                        }
                        "west" | "east" => {
                            debug!("Its a Hallway!");
                            self.cell_type = CellType::Hallway;
                        }
                        _ => panic!("illegal side for wall: {side}"),
                    },
                    _ => panic!("unknown wall in walls {firstwall}!"),
                }
            }
            2 => {
                debug!("found 2 walls, adding 3rd it becomes a DeadEnd!");
                self.cell_type = CellType::DeadEnd
            }
            _ => panic!("Our cells can only have 3 walls!"),
        }
        self.walls.insert(side.to_string(), Wall::default());
    }

    pub fn was_visited(&self) -> bool {
        self.visited
    }

    pub fn visit(&mut self) {
        self.visited = true;
    }

    pub fn space_allocated(&self) -> i32 {
        3
    }

    pub fn capacity(&self) -> i32 {
        match self.cell_type {
            CellType::XCross => 4,
            CellType::TCross => 6,
            CellType::Hallway => 8,
            CellType::Corner => 9,
            CellType::DeadEnd => 12,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cell() {
        let c = Cell::new(Coords2D(1, 2));
        assert_eq!(c.pos.0, 1);
        assert_eq!(c.pos.1, 2);
        assert_eq!(c.walls.len(), 0);
        assert!(!c.was_visited());
        assert_eq!(c.capacity(), 4);
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

        c.add_wall("south");
        assert_eq!(c.cell_type, CellType::TCross);
        assert_eq!(c.capacity(), 6);

        c.add_wall("north");
        assert_eq!(c.cell_type, CellType::Hallway);
        assert_eq!(c.capacity(), 8);

        c.add_wall("east");
        assert_eq!(c.cell_type, CellType::DeadEnd);
        assert_eq!(c.capacity(), 12);

        assert_eq!(c.walls.len(), 3);
        if let None = c.walls.get("south") {
            assert!(false);
        }
    }

    #[test]
    fn test_cell_build_corner() {
        let mut c = Cell::new(Coords2D(1, 2));

        c.add_wall("south");
        assert_eq!(c.cell_type, CellType::TCross);
        assert_eq!(c.capacity(), 6);

        c.add_wall("east");
        assert_eq!(c.cell_type, CellType::Corner);
        assert_eq!(c.capacity(), 9);

        let mut c2 = Cell::new(Coords2D(2, 3));
        c2.add_wall("east");
        assert_eq!(c2.cell_type, CellType::TCross);
        assert_eq!(c2.capacity(), 6);

        c2.add_wall("south");
        assert_eq!(c.cell_type, CellType::Corner);
        assert_eq!(c.capacity(), 9);
    }

    #[test]
    #[should_panic]
    fn test_build_box_panic() {
        let mut c = Cell::new(Coords2D(1, 2));

        c.add_wall("west");
        assert_eq!(c.cell_type, CellType::TCross);
        assert_eq!(c.capacity(), 6);

        c.add_wall("north");
        assert_eq!(c.cell_type, CellType::Corner);
        assert_eq!(c.capacity(), 9);

        c.add_wall("east");
        assert_eq!(c.cell_type, CellType::DeadEnd);
        assert_eq!(c.capacity(), 12);

        c.add_wall("south");
    }

    #[test]
    #[should_panic]
    fn test_cell_add_wall_twice() {
        let mut c = Cell::new(Coords2D(1, 2));
        c.add_wall("south");
        c.add_wall("south");
    }
}
