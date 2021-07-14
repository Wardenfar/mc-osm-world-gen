use std::cmp::{max, min};

use crate::coord::{Coord, Point};
use crate::parser::Store;

#[derive(Debug)]
pub struct Tile {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl Tile {
    pub fn top_right(&self) -> Point {
        Point {
            x: self.bottom_right.x,
            y: self.top_left.y,
        }
    }

    pub fn bottom_left(&self) -> Point {
        Point {
            x: self.top_left.x,
            y: self.bottom_right.y,
        }
    }
}

impl Tile {
    pub fn new(a: Coord, b: Coord) -> Self {
        let pa = a.to_point();
        let pb = b.to_point();
        let tl = Point {
            x: f64::min(pa.x, pb.x),
            y: f64::min(pa.y, pb.y),
        };
        let br = Point {
            x: f64::max(pa.x, pb.x),
            y: f64::max(pa.y, pb.y),
        };
        Tile {
            top_left: tl,
            bottom_right: br,
        }
    }
}

pub fn render(store: &Store, tile: &Tile) {
    let way_ids: Vec<i64> = store.ways_in_tile(tile);
}

#[cfg(test)]
mod test {
    use std::io::Error;
    use std::process::id;

    use geo::LineString;

    use crate::parser::parse_pbf;

    use super::*;
    use geo::intersects::Intersects;

    #[test]
    fn intersect() {
        let a: LineString<f64> = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]].into();
        let b: LineString<f64> = vec![[-0.5, 0.5], [0.5, -0.5]].into();
        assert!(a.intersects(&b))
    }

    #[test]
    fn find() -> Result<(), Error> {
        let store = parse_pbf("herblay.pbf")?;
        let tile = Tile::new(Coord {
            lat: 48.99090,
            lon: 2.15105,
        }, Coord {
            lat: 48.99077,
            lon: 2.15126,
        });

        let ids = store.ways_in_tile(&tile);
        assert_eq!(ids.len(), 1);

        Ok(())
    }
}