use std::i32;

use crate::{
    warehouse::{CellGrid, Coords2D},
    Direction,
};
use log::{debug, error};

pub const ROBOT_SPRITE: &str = "o"; // robot face (U+1F916)
pub const NORTH_WALL: &str = "+-"; // horizontal scan line-1 (U+23BA)
pub const NORTH_WEST_CORNER: &str = "+ ";
pub const BOTTOM_WALL: &str = "+-"; // horizontal scan line-9 (U+23BD)
pub const LEFT_WALL: &str = "|"; //vertical line extension (U+23D0)
pub const RIGHT_WALL: &str = "|"; //vertical line extension (U+23D0)

// TODO: Maybe it is not the best return type
pub fn min_max_xy<'a, T>(coord_iter: T) -> ((i32, i32), (i32, i32))
where
    T: Iterator<Item = &'a Coords2D>,
{
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for c in coord_iter {
        if c.x < min_x {
            min_x = c.x;
        }
        if c.x > max_x {
            max_x = c.x;
        }
        if c.y < min_y {
            min_y = c.y;
        }
        if c.y > max_y {
            max_y = c.y;
        }
    }
    debug!("X: [{}..{}] Y:[{}..{}]", min_x, max_x, min_y, max_y);
    ((min_x, max_x), (min_y, max_y))
}

pub fn output_gridsize(min: i32, max: i32) -> usize {
    if max < min {
        panic!("Max cannot be smaller than min!");
    }
    ((max - min) * 2 + 3) as usize // Note: we are 0 indext so there are 3 cells from -1 to 1
}

pub fn draw_warehouse(grid: &CellGrid) -> String {
    // How large is the map?
    let ((x_min, x_max), (y_min, y_max)) = min_max_xy(grid.keys());
    let size_x = output_gridsize(x_min, x_max);
    let size_y = output_gridsize(y_min, y_max);
    // Allocate memory
    let mut s = String::with_capacity(size_x * size_y);

    // Rows: Each row writes 2 at least lines:
    // top_wall (if cell exists) \n
    // leftwall (if cell exists), space (robot), right_wall (for the last entry)
    // bottom_wall for the last row: We go 1 below the last coordinate to make sure to draw the lines
    for y_index in y_min..=(y_max + 1) {
        let mut top_wall = String::with_capacity(size_x + 1); // add 1 for newline
        let mut side_walls = String::with_capacity(size_y + 1);
        // Columns
        for x_index in x_min..=(x_max + 1) {
            let current_pos = Coords2D {
                x: x_index,
                y: y_index,
            };

            match grid.get(&current_pos) {
                // If we have a cell at this place draw left and top
                Some(cell) => {
                    if cell.has_wall(&Direction::NORTH) {
                        top_wall.push_str(NORTH_WALL);
                    } else {
                        top_wall.push_str(NORTH_WEST_CORNER);
                    }

                    if cell.has_wall(&Direction::WEST) {
                        side_walls.push_str(LEFT_WALL);
                    } else {
                        side_walls.push_str(" ");
                    }
                    // ID or Robot sprite
                    side_walls.push_str(&cell.id);
                }
                // here is no cell, but was is there a cell above or to the left that needs walls?
                None => {
                    if let Some(north) = grid.get(&coords_to_north(&current_pos)) {
                        if north.has_wall(&Direction::SOUTH) {
                            top_wall.push_str(NORTH_WALL);
                        } else {
                            top_wall.push_str(NORTH_WEST_CORNER);
                        }
                    } else if grid.contains_key(&coords_to_north_west(&current_pos)) {
                        top_wall.push_str(NORTH_WEST_CORNER);
                    } else {
                        top_wall.push_str("  "); // No wall and space
                    }

                    if let Some(west) = grid.get(&coords_to_west(&current_pos)) {
                        if west.has_wall(&Direction::EAST) {
                            side_walls.push_str(LEFT_WALL);
                            side_walls.push_str(" "); // Space or Robot sprite
                        } else {
                            side_walls.push_str("  "); // No wall and space
                        }
                    } else {
                        side_walls.push_str("  "); // No wall and space
                    }
                }
            }
        }
        top_wall.push('\n');
        side_walls.push('\n');
        s.push_str(&top_wall);
        s.push_str(&side_walls);
    }
    s
}

pub fn coords_to_west(current: &Coords2D) -> Coords2D {
    Coords2D {
        x: current.x - 1,
        y: current.y,
    }
}

pub fn coords_to_north(current: &Coords2D) -> Coords2D {
    Coords2D {
        x: current.x,
        y: current.y - 1,
    }
}

pub fn coords_to_north_west(current: &Coords2D) -> Coords2D {
    Coords2D {
        x: current.x - 1,
        y: current.y - 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::warehouse::Cell;

    use super::*;

    fn warehouse_1() -> CellGrid {
        let mut cg = CellGrid::new();
        let coords = vec![
            Coords2D { x: 1, y: 0 },
            Coords2D { x: 2, y: 0 },
            Coords2D { x: 3, y: 0 },
            Coords2D { x: 0, y: 1 },
            Coords2D { x: 1, y: 1 },
            Coords2D { x: 2, y: 1 },
            Coords2D { x: 0, y: 2 },
            Coords2D { x: 1, y: 2 },
            Coords2D { x: 2, y: 2 },
            Coords2D { x: 3, y: 2 },
            Coords2D { x: 4, y: 2 },
        ];
        for pos in coords {
            cg.insert(pos.clone(), Cell::new(pos));
        }
        cg
    }

    fn warehouse_2() -> CellGrid {
        let mut cg = CellGrid::new();
        // Cell1
        let pos = Coords2D { x: -2, y: 2 };
        let mut c = Cell::new(pos.clone());
        c.id = "A".to_string();
        _ = c.add_wall(Direction::WEST);
        _ = c.add_wall(Direction::NORTH);
        _ = c.add_wall(Direction::SOUTH);
        cg.insert(pos, c);
        // Cell 2
        let pos = Coords2D { x: -1, y: 2 };
        let mut c = Cell::new(pos.clone());
        c.id = "B".to_string();
        _ = c.add_wall(Direction::NORTH);
        _ = c.add_wall(Direction::SOUTH);
        cg.insert(pos, c);
        // Cell 3
        let pos = Coords2D { x: 0, y: 1 };
        let mut c = Cell::new(pos.clone());
        c.id = "C".to_string();
        _ = c.add_wall(Direction::NORTH);
        _ = c.add_wall(Direction::WEST);
        cg.insert(pos, c);
        // Cell 4
        let pos = Coords2D { x: 0, y: 2 };
        let mut c = Cell::new(pos.clone());
        c.id = "D".to_string();
        cg.insert(pos, c);

        // Cell 5
        let pos = Coords2D { x: 0, y: 3 };
        let mut c = Cell::new(pos.clone());
        c.id = "E".to_string();
        _ = c.add_wall(Direction::WEST);
        _ = c.add_wall(Direction::SOUTH);
        cg.insert(pos, c);
        // Cell 6
        let pos = Coords2D { x: 1, y: 0 };
        let mut c = Cell::new(pos.clone());
        c.id = "F".to_string();
        _ = c.add_wall(Direction::NORTH);
        _ = c.add_wall(Direction::WEST);
        cg.insert(pos, c);
        // Cell 7
        let pos = Coords2D { x: 1, y: 1 };
        let mut c = Cell::new(pos.clone());
        c.id = "G".to_string();
        _ = c.add_wall(Direction::SOUTH);
        _ = c.add_wall(Direction::WEST);
        cg.insert(pos, c);
        // Cell 8
        let pos = Coords2D { x: 1, y: 2 };
        let mut c = Cell::new(pos.clone());
        c.id = "H".to_string();
        _ = c.add_wall(Direction::NORTH);
        cg.insert(pos, c);
        // Cell 9
        let pos = Coords2D { x: 1, y: 3 };
        let mut c = Cell::new(pos.clone());
        c.id = "I".to_string();
        cg.insert(pos, c);
        // Cell 10
        let pos = Coords2D { x: 1, y: 4 };
        let mut c = Cell::new(pos.clone());
        c.id = "J".to_string();
        _ = c.add_wall(Direction::SOUTH);
        _ = c.add_wall(Direction::WEST);
        _ = c.add_wall(Direction::EAST);
        cg.insert(pos, c);
        // Cell 11
        let pos = Coords2D { x: 2, y: 0 };
        let mut c = Cell::new(pos.clone());
        c.id = "K".to_string();
        _ = c.add_wall(Direction::NORTH);
        _ = c.add_wall(Direction::EAST);
        cg.insert(pos, c);
        // Cell 12
        let pos = Coords2D { x: 2, y: 1 };
        let mut c = Cell::new(pos.clone());
        c.id = "L".to_string();
        _ = c.add_wall(Direction::EAST);
        cg.insert(pos, c);
        // Cell 13
        let pos = Coords2D { x: 2, y: 2 };
        let mut c = Cell::new(pos.clone());
        c.id = "M".to_string();
        _ = c.add_wall(Direction::SOUTH);
        cg.insert(pos, c);
        // Cell 14
        let pos = Coords2D { x: 2, y: 3 };
        let mut c = Cell::new(pos.clone());
        c.id = "N".to_string();
        _ = c.add_wall(Direction::NORTH);
        cg.insert(pos, c);
        // Cell 15
        let pos = Coords2D { x: 2, y: 4 };
        let mut c = Cell::new(pos.clone());
        c.id = "O".to_string();
        _ = c.add_wall(Direction::SOUTH);
        _ = c.add_wall(Direction::WEST);
        _ = c.add_wall(Direction::EAST);
        cg.insert(pos, c);
        // Cell 16
        let pos = Coords2D { x: 3, y: 2 };
        let mut c = Cell::new(pos.clone());
        c.id = "P".to_string();
        _ = c.add_wall(Direction::SOUTH);
        _ = c.add_wall(Direction::NORTH);
        cg.insert(pos, c);
        // Cell 17
        let pos = Coords2D { x: 3, y: 3 };
        let mut c = Cell::new(pos.clone());
        c.id = "Q".to_string();
        _ = c.add_wall(Direction::NORTH);
        _ = c.add_wall(Direction::EAST);
        _ = c.add_wall(Direction::SOUTH);
        cg.insert(pos, c);
        // Cell 18
        let pos = Coords2D { x: 4, y: 2 };
        let mut c = Cell::new(pos.clone());
        c.id = "R".to_string();
        _ = c.add_wall(Direction::SOUTH);
        _ = c.add_wall(Direction::NORTH);
        cg.insert(pos, c);
        cg
    }

    #[test]
    fn test_gridsize() {
        assert_eq!(output_gridsize(0, 3), 9);
        assert_eq!(output_gridsize(-2, 2), 11);
        assert_eq!(output_gridsize(1, 3), 7);
        assert_eq!(output_gridsize(1, 1), 3);
    }
    // #[test]
    // #[should_panic]
    // fn test_gridsize_panic() {
    //     output_gridsize(2, 0);
    // }
    #[test]
    fn test_min_max_1() {
        let cg = warehouse_1();

        let ((x_min, x_max), (y_min, y_max)) = min_max_xy(cg.keys());
        assert_eq!(x_min, 0);
        assert_eq!(x_max, 4);
        assert_eq!(y_min, 0);
        assert_eq!(y_max, 2);
    }

    #[test]
    fn test_min_max_2() {
        let cg = warehouse_2();
        let ((x_min, x_max), (y_min, y_max)) = min_max_xy(cg.keys());
        assert_eq!(x_min, -2);
        assert_eq!(x_max, 4);

        assert_eq!(y_min, 0);
        assert_eq!(y_max, 4);
    }
    #[test]
    fn test_print() {
        let cg = warehouse_2();
        let s = draw_warehouse(&cg);
        println!("\n\n{}\n", s);
    }
}
