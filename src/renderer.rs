use std::cmp::{max, min};
use std::fs::File;

use cairo::{Context, Format, glib, ImageSurface};
use cairo::glib::Error;
use cairo::PatternType::Surface;

use crate::coord::{Coord, Point};
use crate::parser::{Node, Store};

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

pub fn render(store: &Store, tile: &Tile, size: f64, line_width: f64) {
    let way_ids: Vec<i64> = store.ways_in_tile(tile);
    let surface = ImageSurface::create(Format::Rgb30, size as i32, size as i32).expect("create surface");
    let ctx = Context::new(&surface).expect("create context");

    ctx.set_source_rgb(0f64, 0f64, 0f64);
    ctx.rectangle(0f64, 0f64, size, size);
    ctx.fill();

    ctx.set_line_width(line_width);
    ctx.set_source_rgb(1f64, 1f64, 1f64);

    // ctx.translate(size / 2f64, size / 2f64);
    // ctx.rotate(0f64.to_radians());

    let top_left = &tile.top_left;
    let tile_size = f64::abs(&tile.bottom_right.x - &tile.top_left.x);

    for id in way_ids {
        if let Some(w) = store.ways.get(&id) {
            let points: Vec<&Point> = w.node_ids.iter()
                .filter_map(|nid| store.nodes.get(nid))
                .map(|n| &n.point)
                .collect();

            let mut first = true;
            for p in points {
                let x = (p.x - top_left.x) * size / tile_size;
                let y = (p.y - top_left.y) * size / tile_size;

                if first {
                    ctx.move_to(x, y);
                    first = false;
                } else {
                    ctx.line_to(x, y);
                }
            }
        }
    }
    ctx.stroke();

    let mut file = File::create("file.png").expect("Couldn't create 'file.png'");
    match surface.write_to_png(&mut file) {
        Ok(_) => println!("file.png created"),
        Err(_) => println!("Error create file.png"),
    }
}

#[cfg(test)]
mod test {
    use std::io::Error;
    use std::process::id;

    use geo::intersects::Intersects;
    use geo::LineString;

    use crate::parser::parse_pbf;

    use super::*;

    #[test]
    fn intersect() {
        let a: LineString<f64> = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]].into();
        let b: LineString<f64> = vec![[-0.5, 0.5], [0.5, -0.5]].into();
        assert!(a.intersects(&b))
    }

    #[test]
    fn render() {
        let store = parse_pbf("herblay.pbf").expect("read pbf file");
        let tile = Tile::new(Coord {
            lat: 48.99265,
            lon: 2.14804,
        }, Coord {
            lat: 48.98922,
            lon: 2.15341,
        });

        super::render(&store, &tile, 256f64, 1f64);
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