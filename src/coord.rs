use geo::Coordinate;

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
        Coord { lat, lon }
    }

    pub fn to_point(&self, zoom: usize) -> Point {
        let subpixel = googleprojection::from_ll_to_subpixel(&(self.lon, self.lat), zoom).unwrap();
        Point {
            x: subpixel.0,
            y: subpixel.1,
        }
    }
}

impl Point {
    pub fn to_geo(&self) -> Coordinate<f64> {
        Coordinate {
            x: self.x,
            y: self.y,
        }
    }
}

macro_rules! point_op {
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
