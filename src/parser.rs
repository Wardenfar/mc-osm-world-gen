use std::collections::HashMap;
use std::path::Path;

use geo::contains::Contains;
use geo::intersects::Intersects;
use geo::{Coordinate, LineString, Rect};
use osmpbf::{Element, ElementReader, Error};
use tracing::span;
use tracing::Level;

use crate::coord::{Coord, Point};
use crate::renderer::Tile;

#[derive(Debug)]
pub struct Store {
    pub nodes: HashMap<i64, Node>,
    pub ways: HashMap<i64, Way>,
    pub multi_polygons: HashMap<i64, MultiPolygon>,
    pub ways_by_type: HashMap<String, Vec<i64>>,
    pub min_point: Point,
    pub max_point: Point,
}

impl Store {
    pub fn ways_in_tile_by_type(&self, tile: &Tile, way_type: Option<String>) -> Vec<i64> {
        let mut ids = Vec::new();

        let tile_geo = Rect::<f64>::new(tile.top_left.to_geo(), tile.bottom_right.to_geo());
        let tile_border = LineString(vec![
            tile.top_left.to_geo(),
            tile.top_right().to_geo(),
            tile.bottom_right.to_geo(),
            tile.bottom_left().to_geo(),
            tile.top_left.to_geo(),
        ]);

        let restricted_ids = if let Some(t) = &way_type {
            self.ways_by_type.get(t)
        } else {
            None
        };

        for (id, way) in &self.ways {
            if let Some(restricted_ids) = restricted_ids {
                if !restricted_ids.contains(id) {
                    continue;
                }
            }

            let points: Vec<Coordinate<f64>> = way
                .node_ids
                .iter()
                .filter_map(|node_id| self.nodes.get(node_id))
                .map(|node| node.point.to_geo())
                .collect();

            let mut found = false;

            for p in &points {
                if tile_geo.contains(p) {
                    ids.push(*id);
                    found = true;
                    break;
                }
            }

            if !found {
                let way_geo = LineString(points);
                if tile_border.intersects(&way_geo) {
                    ids.push(*id);
                }
            }
        }
        ids
    }
}

impl Default for Store {
    fn default() -> Self {
        Store {
            nodes: Default::default(),
            ways: Default::default(),
            multi_polygons: Default::default(),
            ways_by_type: Default::default(),
            min_point: Point { x: 0.0, y: 0.0 },
            max_point: Point { x: 0.0, y: 0.0 },
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub id: i64,
    pub coord: Coord,
    pub point: Point,
}

#[derive(Debug)]
pub struct Way {
    pub id: i64,
    pub node_ids: Vec<i64>,
}

#[derive(Debug)]
pub struct MultiPolygon {
    pub id: i64,
    pub outer_ways: Vec<i64>,
    pub inner_ways: Vec<i64>,
}

pub fn parse_pbf(filename: impl AsRef<Path>, zoom: usize) -> Result<Store, Error> {
    let span = span!(Level::TRACE, "parse_pbf", zoom = zoom);
    let _guard = span.enter();
    let reader = ElementReader::from_path(filename)?;
    let mut store = Store::default();

    let mut min_x: f64 = 0.;
    let mut max_x: f64 = 0.;
    let mut min_y: f64 = 0.;
    let mut max_y: f64 = 0.;

    reader.for_each(|element| match element {
        Element::Node(n) => {
            let coord = Coord::new(n.lat(), n.lon());
            let point = coord.to_point(zoom);

            if min_x == 0. || point.x < min_x {
                min_x = point.x
            }
            if max_x == 0. || point.x > max_x {
                max_x = point.x
            }
            if min_y == 0. || point.y < min_y {
                min_y = point.y;
            }
            if max_y == 0. || n.lon() > max_y {
                max_y = point.y;
            }

            let node = Node {
                id: n.id(),
                coord,
                point,
            };
            store.nodes.insert(node.id, node);
        }
        Element::DenseNode(n) => {
            let coord = Coord::new(n.lat(), n.lon());
            let point = coord.to_point(zoom);

            if min_x == 0. || point.x < min_x {
                min_x = point.x
            }
            if max_x == 0. || point.x > max_x {
                max_x = point.x
            }
            if min_y == 0. || point.y < min_y {
                min_y = point.y;
            }
            if max_y == 0. || n.lon() > max_y {
                max_y = point.y;
            }

            let node = Node {
                id: n.id(),
                coord,
                point,
            };
            store.nodes.insert(node.id, node);
        }
        Element::Way(w) => {
            let id = w.id();
            let node_ids: Vec<i64> = w.refs().collect();
            let way = Way { id, node_ids };
            store.ways.insert(id, way);

            for (key, _) in w.tags() {
                match key {
                    "highway" => store
                        .ways_by_type
                        .entry("highway".to_string())
                        .or_default()
                        .push(w.id()),
                    "water" => store
                        .ways_by_type
                        .entry("water".to_string())
                        .or_default()
                        .push(w.id()),
                    "building" => store
                        .ways_by_type
                        .entry("building".to_string())
                        .or_default()
                        .push(w.id()),
                    _ => {}
                }
            }
        }
        Element::Relation(_) => {}
    })?;

    store.min_point = Point { x: min_x, y: min_y };

    store.max_point = Point { x: max_x, y: max_y };

    Ok(store)
}
