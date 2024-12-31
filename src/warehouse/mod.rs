use crate::Direction;
use crate::Direction::{EAST, NORTH, SOUTH, WEST};
use crate::{direction_to_literal, literal_to_direction};
use log::{debug, error};
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct Wall {}

pub type CellGrid = HashMap<Coords2D, Cell>;
#[derive(Default, Deserialize, Serialize, PartialEq, Debug)]
pub struct Warehouse {
    #[serde(serialize_with = "serialize_cellgrid")]
    #[serde(deserialize_with = "deserialize_cellgrid")]
    cell_layout: CellGrid,
}

//Hashmap with Coords as key does not serialize nicely
fn serialize_cellgrid<S>(grid: &CellGrid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(grid.len()))?;
    for (k, v) in grid {
        map.serialize_entry(&k.to_string(), &v)?;
    }
    map.end()
}

fn deserialize_cellgrid<'a, D>(deserializer: D) -> Result<CellGrid, D::Error>
where
    D: Deserializer<'a>,
{
    struct MapVisitor;

    impl<'a> Visitor<'a> for MapVisitor {
        type Value = CellGrid;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("Expected \"i32,i32\" -> Cell mapping")
        }

        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'a>,
        {
            let mut values = CellGrid::new();
            while let Some((key, value)) = (access.next_entry::<String, Cell>())? {
                values.insert(Coords2D::from_string(key.as_str()).unwrap(), value);
            }

            Ok(values)
        }
    }

    let visitor = MapVisitor;
    deserializer.deserialize_map(visitor)
}

#[derive(Debug, PartialEq)]
pub enum Error {
    CellInvalid,
    WallExists,
    GoodsIncompatible,
    StorageExceeded,
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Coords2D {
    pub x: i32,
    pub y: i32,
}
impl Coords2D {
    pub fn go(&self, dir: Direction) -> Self {
        match dir {
            NORTH => Self {
                x: self.x,
                y: self.y - 1,
            },
            EAST => Self {
                x: self.x + 1,
                y: self.y,
            },
            SOUTH => Self {
                x: self.x,
                y: self.y + 1,
            },
            WEST => Self {
                x: self.x - 1,
                y: self.y,
            },
        }
    }
    pub fn from_string(stringrep: &str) -> Option<Self> {
        let tokens: Vec<&str> = stringrep.split(",").collect();
        if tokens.len() < 2 {
            return None;
        }

        Some(Self {
            x: tokens[0]
                .parse()
                .unwrap_or_else(|_| panic!("expected i32 for yx got {}", tokens[0])),
            y: tokens[1]
                .parse()
                .unwrap_or_else(|_| panic!("expected i32 for y got {}", tokens[1])),
        })
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum CellType {
    XCross,
    TCross,
    Corner,
    Hallway,
    DeadEnd,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Cell {
    pub pos: Coords2D,
    pub id: String,
    // I really wanted a hashmap for enum->Wall - but that's not working
    walls: HashMap<String, Wall>,
    shelf_inventory: Vec<String>,
    visited: bool,
    cell_type: CellType,
}

impl std::fmt::Display for Coords2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cell: {} - Storage ({}/{}) - Goods: {}",
            self.pos,
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
            id: String::new(),
            walls: HashMap::new(),
            shelf_inventory: Vec::new(),
            visited: false,
            cell_type: CellType::XCross,
        }
    }

    pub fn has_wall(&self, side: &Direction) -> bool {
        self.walls.contains_key(&direction_to_literal(side))
    }

    pub fn add_wall(&mut self, side: Direction) -> Result<CellType, Error> {
        // I can't use enum as key in hashmap... convert back
        let side_str = direction_to_literal(&side);

        if self.walls.contains_key(&side_str) {
            error!("You already have a wall on side: '{side_str}'!");
            return Err(Error::WallExists);
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
            _ => {
                error!("Can't add a 4th wall!");
                return Err(Error::CellInvalid);
            }
        }
        self.walls.insert(side_str, Wall::default());
        Ok(self.cell_type.clone())
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

impl Warehouse {
    pub fn add_default_cell(&mut self, pos: Coords2D) {
        if self.cell_layout.contains_key(&pos) {
            debug!("Not adding cell, pos {} already filled", pos);
            return;
        }
        debug!("Adding default cell at {}", &pos);
        let cell = Cell::new(pos.clone());
        self.cell_layout.insert(pos, cell);
    }

    pub fn insert_or_update_cell(&mut self, pos: Coords2D, cell: Cell) {
        self.cell_layout.insert(pos, cell);
    }

    pub fn storage_capacity(&self) -> usize {
        let mut storage = 0;
        for (_, c) in self.cell_layout.iter() {
            storage += c.storage_capacity();
        }
        storage
    }

    pub fn cells(&self) -> usize {
        self.cell_layout.len()
    }

    pub fn get_cellgrid(&self) -> &CellGrid {
        &self.cell_layout
    }
    pub fn reset(&mut self) {
        debug!("reset map!");
        self.cell_layout.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SOUTH_LIT;

    #[test]
    fn test_coords_from_str() {
        let strcoords = "3,4";
        let c = Coords2D::from_string(strcoords).unwrap();
        assert_eq!(c.x, 3);
        assert_eq!(c.y, 4);

        let wrong = "43";
        assert!(Coords2D::from_string(wrong).is_none());
    }

    #[test]
    #[should_panic]
    fn test_coords_from_str_panic() {
        let wrong = "4,acht";
        assert!(Coords2D::from_string(wrong).is_none());
    }

    #[test]
    fn test_new_cell() {
        let c = Cell::new(Coords2D { x: 1, y: 2 });
        assert_eq!(c.pos.x, 1);
        assert_eq!(c.pos.y, 2);
        assert_eq!(c.walls.len(), 0);
        assert!(!c.was_visited());
        assert_eq!(c.storage_capacity(), 4);
    }

    #[test]
    fn test_visit_cell() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        c.visit();
        assert!(c.was_visited());
    }

    #[test]
    fn test_cell_add_walls_ok() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        assert_eq!(c.cell_type, CellType::XCross);

        assert_eq!(c.add_wall(SOUTH).unwrap(), CellType::TCross);
        assert_eq!(c.storage_capacity(), 6);

        assert_eq!(c.add_wall(NORTH).unwrap(), CellType::Hallway);
        assert_eq!(c.storage_capacity(), 8);

        assert!(c.add_wall(EAST).is_ok());
        assert_eq!(c.cell_type, CellType::DeadEnd);
        assert_eq!(c.storage_capacity(), 12);

        assert_eq!(c.walls.len(), 3);
        assert!(c.walls.contains_key(SOUTH_LIT))
    }

    #[test]
    fn test_cell_build_corner() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        assert_eq!(c.add_wall(SOUTH).unwrap(), CellType::TCross);
        assert_eq!(c.storage_capacity(), 6);
        assert_eq!(c.add_wall(EAST).unwrap(), CellType::Corner);
        assert_eq!(c.storage_capacity(), 9);

        let mut c2 = Cell::new(Coords2D { x: 2, y: 3 });
        assert_eq!(c2.add_wall(EAST).unwrap(), CellType::TCross);
        assert_eq!(c2.storage_capacity(), 6);

        assert_eq!(c2.add_wall(SOUTH).unwrap(), CellType::Corner);
        assert_eq!(c.storage_capacity(), 9);

        println!("{}", c);
    }

    #[test]
    fn test_build_box_panic() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        assert_eq!(c.add_wall(WEST).unwrap(), CellType::TCross);
        assert_eq!(c.storage_capacity(), 6);
        assert_eq!(c.add_wall(NORTH).unwrap(), CellType::Corner);
        assert_eq!(c.storage_capacity(), 9);
        assert_eq!(c.add_wall(EAST).unwrap(), CellType::DeadEnd);
        assert_eq!(c.storage_capacity(), 12);

        assert!(c.add_wall(SOUTH).is_err());
    }

    #[test]
    fn test_cell_add_wall_twice() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        assert!(c.add_wall(SOUTH).is_ok());
        assert!(c.add_wall(SOUTH).is_err());
    }

    #[test]
    fn test_add_items_ok() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        assert_eq!(c.stored_good_type(), "-");
        assert_eq!(c.occupied_storage(), 0);
        assert_eq!(c.storage_capacity(), 4);
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert_eq!(c.occupied_storage(), 1);
        assert_eq!(c.stored_good_type(), "Kürbis");
    }

    #[test]
    fn test_add_items_fail_full() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_err());
        assert_eq!(c.occupied_storage(), c.storage_capacity());
    }

    #[test]
    fn test_add_items_fail_mixed_goods() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        assert!(c.put_good_on_shelf("Kürbis".to_string()).is_ok());
        let e = c.put_good_on_shelf("Milch".to_string());
        assert!(e.is_err());
        assert_eq!(e.unwrap_err(), Error::GoodsIncompatible);
        assert_eq!(c.occupied_storage(), 1);
    }

    #[test]
    fn test_serde_default_cell() {
        let c = Cell::new(Coords2D { x: 1, y: 2 });
        let ser = serde_json::to_string(&c).unwrap();
        assert_eq!(&ser, "{\"pos\":{\"x\":1,\"y\":2},\"id\":\"\",\"walls\":{},\"shelf_inventory\":[],\"visited\":false,\"cell_type\":\"XCross\"}");

        let c2: Cell = serde_json::from_str(ser.as_str()).unwrap();
        assert_eq!(c, c2);
    }

    #[test]
    fn test_serde_cell() {
        let mut c = Cell::new(Coords2D { x: 1, y: 2 });
        c.add_wall(NORTH).unwrap();
        c.put_good_on_shelf("Hydrazine".to_string()).unwrap();
        let ser = serde_json::to_string(&c).unwrap();
        assert_eq!(ser, "{\"pos\":{\"x\":1,\"y\":2},\"id\":\"\",\"walls\":{\"north\":{}},\"shelf_inventory\":[\"Hydrazine\"],\"visited\":false,\"cell_type\":\"TCross\"}");

        let c2: Cell = serde_json::from_str(ser.as_str()).unwrap();
        assert_eq!(c, c2);
    }

    #[test]
    fn test_serde_warehouse() {
        let mut wh = Warehouse::default();
        wh.add_default_cell(Coords2D { x: 4, y: 7 });
        let ser = serde_json::to_string(&wh).unwrap();
        assert_eq!(ser, "{\"cell_layout\":{\"4,7\":{\"pos\":{\"x\":4,\"y\":7},\"id\":\"\",\"walls\":{},\"shelf_inventory\":[],\"visited\":false,\"cell_type\":\"XCross\"}}}");

        let wh2: Warehouse = serde_json::from_str(ser.as_str()).unwrap();
        assert_eq!(wh, wh2);
    }
}
