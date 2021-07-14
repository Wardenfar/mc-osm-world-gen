use std::collections::{HashMap, HashSet};

use geo::{Coordinate, Line, LineString, Rect};
use geo::contains::Contains;
use geo::coords_iter::GeometryCoordsIter::Polygon;
use geo::intersects::Intersects;
use osmpbf::{Element, ElementReader, Error};

use renderer::coords::Coords;

use crate::coord::Coord;
use crate::renderer::Tile;

pub struct Store {
    nodes: HashMap<i64, Node>,
    ways: HashMap<i64, Way>,
    ways_from_node: HashMap<i64, HashSet<i64>>,
}

impl Store {
    pub fn ways_in_tile(&self, tile: &Tile) -> Vec<i64> {
        let mut ids = Vec::new();

        let tile_geo = Rect::<f64>::new(
            tile.top_left.to_geo(),
            tile.bottom_right.to_geo(),
        );
        let tile_border = LineString(vec![
            tile.top_left.to_geo(),
            tile.top_right().to_geo(),
            tile.bottom_right.to_geo(),
            tile.bottom_left().to_geo(),
            tile.top_left.to_geo()
        ]);

        for (id, way) in &self.ways {
            let points: Vec<Coordinate<f64>> = way.node_ids.iter()
                .filter_map(|node_id| self.nodes.get(node_id))
                .map(|node| node.coord.to_point().to_geo())
                .collect();

            let mut found = false;

            for p in &points {
                if tile_geo.contains(p) {
                    ids.push(id.clone());
                    found = true;
                    break;
                }
            }

            if !found {
                let way_geo = LineString(points);
                if tile_border.intersects(&way_geo) {
                    ids.push(id.clone());
                    found = true;
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
            ways_from_node: Default::default(),
        }
    }
}

pub struct Node {
    id: i64,
    coord: Coord,
}

pub struct Way {
    id: i64,
    node_ids: Vec<i64>,
}

pub fn parse_pbf(filename: &str) -> Result<Store, Error> {
    let reader = ElementReader::from_path(filename)?;
    let mut store = Store::default();

    reader.for_each(|element| {
        match element {
            Element::Node(n) => {
                let node = Node {
                    id: n.id(),
                    coord: Coord::new(n.lat(), n.lon()),
                };
                store.nodes.insert(node.id, node);
            }
            Element::DenseNode(n) => {
                let node = Node {
                    id: n.id(),
                    coord: Coord::new(n.lat(), n.lon()),
                };
                store.nodes.insert(node.id, node);
            }
            Element::Way(w) => {
                let id = w.id();
                let node_ids: Vec<i64> = w.refs().collect();
                    // .map(|n| {
                    //     store.ways_from_node.entry(n.clone())
                    //         .or_default()
                    //         .insert(id);
                    //     n
                    // })

                let way = Way {
                    id,
                    node_ids,
                };
                store.ways.insert(id, way);
            }
            Element::Relation(_) => {}
        }
    })?;

    Ok(store)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() -> Result<(), Error> {
        let store = parse_pbf("herblay.pbf")?;
        println!("nodes:{}  ways:{}", store.nodes.len(), store.ways.len());
        assert!(store.nodes.len() > 0);
        assert!(store.ways.len() > 0);
        Ok(())
    }
}