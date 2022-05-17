use tiny_skia::*;

use crate::coord::Point;
use crate::parser::Store;
use crate::{REGION_BLOCK_SIZE, REGION_BLOCK_SIZE_F64};

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

#[derive(Debug)]
pub struct Pixel(pub u8, pub u8, pub u8);

pub fn render(store: &Store, tile: &Tile, line_width: f32) -> Pixmap {
    let mut pixmap = Pixmap::new(REGION_BLOCK_SIZE, REGION_BLOCK_SIZE).unwrap();

    let mut red = Paint::default();
    red.set_color_rgba8(255, 0, 0, 255);
    let mut green = Paint::default();
    green.set_color_rgba8(0, 255, 0, 255);
    let mut white = Paint::default();
    white.set_color_rgba8(255, 255, 255, 255);

    for id in store.ways_in_tile_by_type(tile, Some(String::from("building"))) {
        let path = build_path(store, tile, id);
        if let Some(path) = path {
            pixmap.fill_path(&path, &red, FillRule::Winding, Transform::identity(), None);

            let stroke = Stroke {
                width: 0.0,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &green, &stroke, Transform::identity(), None);
        }
    }

    for id in store.ways_in_tile_by_type(tile, Some(String::from("highway"))) {
        let path = build_path(store, tile, id);
        if let Some(path) = path {
            let stroke = Stroke {
                width: line_width,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &white, &stroke, Transform::identity(), None);
        }
    }

    pixmap
}

fn build_path(store: &Store, tile: &Tile, id: i64) -> Option<Path> {
    let mut path_builder = PathBuilder::new();

    let top_left = &tile.top_left;
    let tile_size = f64::abs(tile.bottom_right.x - tile.top_left.x);

    if let Some(w) = store.ways.get(&id) {
        let points = w
            .node_ids
            .iter()
            .filter_map(|nid| store.nodes.get(nid))
            .map(|n| &n.point);

        for (idx, p) in points.enumerate() {
            let x = ((p.x - top_left.x) * REGION_BLOCK_SIZE_F64 / tile_size) as f32;
            let y = ((p.y - top_left.y) * REGION_BLOCK_SIZE_F64 / tile_size) as f32;

            if idx == 0 {
                path_builder.move_to(x, y);
            } else {
                path_builder.line_to(x, y);
            }
        }
    }

    path_builder.finish()
}
