use std::i32;

use crate::warehouse::{Cell, CellGrid, Coords2D};
use log::{debug, error};

pub const ROBOT_SPRITE: &str = "o"; // robot face (U+1F916)
pub const TOP_WALL: &str = "+-+"; // horizontal scan line-1 (U+23BA)
pub const BOTTOM_WALL: &str = "+-+"; // horizontal scan line-9 (U+23BD)
pub const LEFT_WALL: &str = "|"; //vertical line extension (U+23D0)
pub const RIGHT_WALL: &str = "|"; //vertical line extension (U+23D0)

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

    ((min_x, max_x), (min_y, max_y))
}

pub fn output_gridsize(min: i32, max: i32) -> i32 {
    if max < min {
        panic!("Max cannot be smaller than min!");
    }
    (max - min) * 2 + 3 // Note: we are 0 indext so there are 3 cells from -1 to 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gridsize() {
        assert_eq!(output_gridsize(0, 3), 9);
        assert_eq!(output_gridsize(-2, 2), 11);
        assert_eq!(output_gridsize(1, 3), 7);
        assert_eq!(output_gridsize(1, 1), 3);
    }
    #[test]
    #[should_panic]
    fn test_gridsize_panic() {
        output_gridsize(2, 0);
    }

    #[test]
    fn test_min_max_1() {
        let positions = vec![
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
        let bounds = min_max_xy(positions.iter());
        assert_eq!(bounds.0 .0, 0); // smallest x
        assert_eq!(bounds.0 .1, 4); // largest x

        assert_eq!(bounds.1 .0, 0); // smallest y
        assert_eq!(bounds.1 .1, 2); // largest y
    }

    #[test]
    fn test_min_max_2() {
        let positions = vec![
            // Row 1
            Coords2D { x: 1, y: 0 },
            Coords2D { x: 2, y: 0 },
            // Row 2
            Coords2D { x: 0, y: 1 },
            Coords2D { x: 1, y: 1 },
            Coords2D { x: 2, y: 1 },
            // Row 3
            Coords2D { x: -2, y: 2 },
            Coords2D { x: -1, y: 2 },
            Coords2D { x: 0, y: 2 },
            Coords2D { x: 1, y: 2 },
            Coords2D { x: 2, y: 2 },
            // Row 4
            Coords2D { x: -1, y: 3 },
            Coords2D { x: 0, y: 3 },
            Coords2D { x: 1, y: 3 },
            Coords2D { x: 2, y: 3 },
        ];
        let bounds = min_max_xy(positions.iter());
        assert_eq!(bounds.0 .0, -2); // smallest x
        assert_eq!(bounds.0 .1, 2); // largest x

        assert_eq!(bounds.1 .0, 0); // smallest y
        assert_eq!(bounds.1 .1, 3); // largest y
    }
}
