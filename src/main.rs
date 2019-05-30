use std::env;

extern crate flame;
extern crate osm_xml;
extern crate proj5;

#[macro_use]
extern crate flamer;
mod linemath;

mod osm_load;
mod svg_exporter;
mod theme;

use osm_load::*;
use svg_exporter::*;

#[derive(Clone)]
enum Kind {
    Building(RangeIdx),
    Road(RangeIdx),
    Coastline(RangeIdx),
    Park(RangeIdx),
    ProcessedCoastline(Vec<(f64, f64)>),
    ProcessedPark(Vec<(f64, f64)>),
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Layer {
    Building,
    Road,
    Coastline,
    Park,
    ParkPath,
    ParkBuilding,
}

impl Kind {
    fn to_layer(&self) -> Layer {
        match self {
            Kind::Building(_) => Layer::Building,
            Kind::Road(_) => Layer::Road,
            Kind::Coastline(_) => Layer::Coastline,
            Kind::ProcessedCoastline(_) => Layer::Coastline,
            Kind::Park(_) => Layer::Park,
            Kind::ProcessedPark(_) => Layer::Park,
        }
    }
    fn resolve_coords<'a>(&'a self, geom: &'a Geometry) -> &'a [(f64, f64)] {
        match self {
            Kind::Building(r) | Kind::Road(r) | Kind::Coastline(r) | Kind::Park(r) => {
                geom.resolve_coords(*r)
            }
            Kind::ProcessedCoastline(v) => &v[..],
            Kind::ProcessedPark(v) => &v[..],
        }
    }
}

fn filter(relationship_tags: &[Tag], way_tags: &[Tag], range: RangeIdx) -> Option<Kind> {
    type T = fn(RangeIdx) -> Kind;
    osm_load::simple_filterer(|tag| match tag {
        ("highway", _) => Some(Kind::Road as T),
        ("building", _) => Some(Kind::Building as T),
        (_, "coastline") => Some(Kind::Coastline as T),
        (_, "park") => Some(Kind::Park as T),
        (_, "garden") => Some(Kind::Park as T),
        (_, "grass") => Some(Kind::Park as T),
        (_, "memorial") => Some(Kind::Park as T),
        _ => None,
    })(relationship_tags, way_tags, range)
}

#[flame]
fn process_coastline_and_parks(results: Vec<Kind>, geometry: &Geometry) -> Vec<Kind> {
    let mut coastlines: Vec<Vec<_>> = vec![];
    let mut disconnected_parks: Vec<Vec<_>> = vec![];
    let mut acc = vec![];
    for kind in results {
        match kind {
            Kind::Coastline(idx) => {
                coastlines.push(geometry.resolve_coords(idx).into_iter().cloned().collect())
            }
            Kind::Park(idx) => {
                let geometry = geometry.resolve_coords(idx);
                if geometry[0] == geometry[geometry.len() - 1] {
                    acc.push(Kind::Park(idx));
                } else {
                    let geometry = geometry.into_iter().cloned().collect();
                    disconnected_parks.push(geometry);
                }
            }
            other => acc.push(other),
        }
    }

    // Coastlines
    let coastlines = linemath::dedup(coastlines);
    let mut coastlines = linemath::connect(coastlines);
    for coastline in coastlines.iter_mut() {
        linemath::equalize(coastline);
    }

    for coastline in coastlines {
        acc.push(Kind::ProcessedCoastline(coastline));
    }

    // Parks
    let disconnected_parks = linemath::dedup(disconnected_parks);
    let mut disconnected_parks = linemath::connect(disconnected_parks);
    for park in disconnected_parks.iter_mut() {
        linemath::equalize(park);
    }

    for park in disconnected_parks {
        acc.push(Kind::ProcessedPark(park));
    }

    acc
}

#[flame]
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filename = &args[1].to_string();
    let osm_file = format!("./{}.osm", filename.as_str());
    let (geometry, results) = Geometry::from_file(&osm_file, &filter, 1000.0);
    let bounds = geometry.bounds;
    let results = process_coastline_and_parks(results, &geometry);

    let mut svg = Svg::new(bounds);

    svg.set_clippings_layer(Layer::ParkBuilding, Layer::Park);
    svg.set_clippings_layer(Layer::ParkPath, Layer::Park);

    theme::gray_theme(&mut svg);

    for kind in &results {
        let layer = kind.to_layer();
        let coords = kind.resolve_coords(&geometry);
        svg.draw_polyline(layer, coords)?;

        match layer {
            Layer::Building => svg.draw_polyline(Layer::ParkBuilding, coords)?,
            Layer::Road => svg.draw_polyline(Layer::ParkPath, coords)?,
            _ => (),
        }
    }

    let layer_order = &[
        Layer::Coastline,
        Layer::Park,
        Layer::Road,
        Layer::Building,
        Layer::ParkBuilding,
        Layer::ParkPath,
    ];

    svg.export_to_file(&format!("./{}.svg", filename), layer_order)?;
    flame::dump_html(std::fs::File::create("./flame.html")?)?;

    Ok(())
}
