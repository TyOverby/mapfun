use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use std::fs::File;
use std::io::BufReader;

pub type Coordinate = [f64; 2];
pub type LineCoordinates = Vec<Coordinate>;
pub type PolygonCoordinates = Vec<Vec<Coordinate>>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geometry {
    Point {
        coordinates: Coordinate,
    },
    LineString {
        coordinates: LineCoordinates,
    },
    Polygon {
        coordinates: PolygonCoordinates,
    },
    MultiPoint {
        coordinates: Vec<Coordinate>,
    },
    MultiLineString {
        coordinates: Vec<LineCoordinates>,
    },
    MultiPolygon {
        coordinates: Vec<PolygonCoordinates>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct Feature {
    pub geometry: Geometry,
    pub properties: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct Geojson {
    pub features: Vec<Feature>,
}

pub fn from_file(path: &str) -> Result<Geojson> {
    println!("Attempting to load geojson from: {}", path);
    let f = File::open(path).unwrap();
    let buf_reader = BufReader::new(f);
    let v: Geojson = serde_json::from_reader(buf_reader).unwrap();

    println!("Loaded geojson");
    println!("Num features {}", v.features.len());

    Ok(v)
}
