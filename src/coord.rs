use std::f64::consts::PI;
use std::ops::{Add, Mul, Sub};

use geo::Coordinate;
use mercator::{lnglat_to_mercator, mercator_to_lnglat};

pub const MAX_ZOOM: u8 = 18;
pub const TILE_SIZE: u32 = 256;

#[derive(Debug)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Coord {
    pub fn new(lat: f64, lon: f64) -> Self {
        Coord {
            lat,
            lon,
        }
    }

    pub fn to_point(&self, zoom: usize) -> Point {
        let subpixel = googleprojection::from_ll_to_subpixel(&(self.lon, self.lat), zoom).unwrap();
        Point {
            x: subpixel.0,
            y: subpixel.1,
        }
    }

    // pub fn to_tile(&self, zoom: u8) -> Tile {
    //     let (x, y) = coords_to_xy(coords, zoom);
    //     let tile_index = |t| (t as u32) / TILE_SIZE;
    //     Tile {
    //         zoom,
    //         x: tile_index(x),
    //         y: tile_index(y),
    //     }
    // }
}

// fn coords_to_xy(coords: &Coord, zoom: u8) -> (f64, f64) {
//     let (lat_rad, lon_rad) = (coords.lat.to_radians(), coords.lon.to_radians());
//
//     let x = lon_rad + PI;
//     let y = PI - ((PI / 4f64) + (lat_rad / 2f64)).tan().ln();
//
//     let rescale = |x: f64| {
//         let factor = x / (2f64 * PI);
//         let dimension_in_pixels = f64::from(TILE_SIZE * (1 << zoom));
//         factor * dimension_in_pixels
//     };
//
//     (rescale(x), rescale(y))
// }

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point {
            x,
            y,
        }
    }

    pub fn to_coord(&self) -> Coord {
        let (lon, lat) = mercator_to_lnglat(self.x, self.y, 121.0, 0.9999, 250000.0);
        Coord {
            lat,
            lon,
        }
    }

    pub fn to_geo(&self) -> Coordinate<f64> {
        Coordinate {
            x: self.x,
            y: self.y,
        }
    }
}

macro_rules! point_op {
    // `()` indicates that the macro takes no argument.
    ($trait:tt, $fn:ident, $op:tt) => {
        impl std::ops::$trait for Point {
            type Output = Point;

            fn $fn(self, rhs: Self) -> Self::Output {
                Point {
                    x: self.x $op rhs.x,
                    y: self.y $op rhs.y
                }
            }
        }
    };
}


macro_rules! coord_op {
    // `()` indicates that the macro takes no argument.
    ($trait:tt, $fn:ident, $op:tt) => {
        impl std::ops::$trait for Coord {
            type Output = Coord;

            fn $fn(self, rhs: Self) -> Self::Output {
                Coord {
                    lat: self.lat $op rhs.lat,
                    lon: self.lon $op rhs.lon
                }
            }
        }
    };
}

point_op!(Sub,sub,-);
point_op!(Add,add,+);
point_op!(Mul,mul,*);
point_op!(Div,div,/);

coord_op!(Sub,sub,-);
coord_op!(Add,add,+);
coord_op!(Mul,mul,*);
coord_op!(Div,div,/);
