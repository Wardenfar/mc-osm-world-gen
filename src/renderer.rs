use cairo::{Antialias, Context, Format, ImageSurface};

use crate::coord::Point;
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

pub struct Pixel(pub u8, pub u8, pub u8);

pub fn render(store: &Store, tile: &Tile, size: f64, line_width: f64) -> Result<Vec<Pixel>, cairo::Error> {
    let mut surface = ImageSurface::create(Format::Rgb24, size as i32, size as i32).expect("create surface");
    {
        let ctx = Context::new(&surface).expect("create context");

        ctx.set_antialias(Antialias::None);
        ctx.set_source_rgb(0f64, 0f64, 0f64);
        ctx.rectangle(0f64, 0f64, size, size);
        ctx.fill()?;

        for id in store.ways_in_tile_by_type(tile, Some(String::from("building"))) {
            build_path(store, size, &ctx, tile, &id);
            ctx.set_source_rgb(1f64, 0f64, 0f64);
            ctx.fill_preserve()?;

            ctx.set_source_rgb(0f64, 1f64, 0f64);
            ctx.set_line_width(1f64);
            ctx.stroke()?;
        }

        for id in store.ways_in_tile_by_type(tile, Some(String::from("highway"))) {
            build_path(store, size, &ctx, tile, &id);
            ctx.set_source_rgb(1f64, 1f64, 1f64);
            ctx.set_line_width(line_width);
            ctx.stroke()?;
        }
    }

    let data = surface.data().unwrap().to_vec();

    let pixels: Vec<Pixel> = data.chunks(4)
        .map(|c| Pixel(c[2], c[1], c[0]))
        .collect();

    Ok(pixels)
}

fn build_path(store: &Store, size: f64, ctx: &Context, tile: &Tile, id: &i64) {
    let top_left = &tile.top_left;
    let tile_size = f64::abs(tile.bottom_right.x - tile.top_left.x);

    ctx.new_path();

    if let Some(w) = store.ways.get(id) {
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