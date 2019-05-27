extern crate flame;
extern crate osm_xml;
extern crate proj5;

#[macro_use]
extern crate flamer;
mod linemath;

mod osm_load;
mod svg_exporter;

use osm_load::*;
use svg_exporter::*;

#[derive(Clone)]
enum Kind {
    Building(RangeIdx),
    Road(RangeIdx),
    Coastline(RangeIdx),
    Park(RangeIdx),
    ProcessedCoastline(Vec<(f64, f64)>),
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
enum Layer {
    Building,
    Road,
    Coastline,
    Park,
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
        }
    }
    fn resolve_coords<'a>(&'a self, geom: &'a Geometry) -> &'a [(f64, f64)] {
        match self {
            Kind::Building(r) | Kind::Road(r) | Kind::Coastline(r) | Kind::Park(r) => {
                geom.resolve_coords(*r)
            }
            Kind::ProcessedCoastline(v) => &v[..],
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
        _ => None,
    })(relationship_tags, way_tags, range)
}

#[flame]
fn process_coastline(results: Vec<Kind>, geometry: &Geometry) -> Vec<Kind> {

    let mut coastlines: Vec<Vec<_>> = vec![];
    let mut others = vec![];
    for kind in results {
        match kind {
            Kind::Coastline(idx) => {
                coastlines.push(geometry.resolve_coords(idx).into_iter().cloned().collect())
            }
            other => others.push(other),
        }
    }
    println!("coastlines before deduping: {}", coastlines.len());
    let coastlines = linemath::dedup(coastlines);
    println!("coastlines after deduping: {}", coastlines.len());
    let mut coastlines = linemath::connect(coastlines);
    println!("coastlines after connecting: {}", coastlines.len());

    for coastline in coastlines.iter_mut() {
        linemath::equalize(coastline);
    }


    for coastline in coastlines {
        others.push(Kind::ProcessedCoastline(coastline));
    }

    others
}

#[flame]
fn main() -> std::io::Result<()> {
    let (geometry, results) = Geometry::from_file("./nyc.osm", &filter, 1000.0);
    let bounds = geometry.bounds;
    let results = process_coastline(results, &geometry);

    let mut svg = Svg::new(bounds);

    svg.set_background_color("#000020");
    svg.set_clippings_layer(Layer::ParkBuilding, Layer::Park);
    svg.set_style(
        Layer::Road,
        "road",
        "fill:none; stroke:darkgrey; stroke-width:0.07%; stroke-linecap:round",
    );

    svg.set_style(
        Layer::Building,
        "building",
        "fill:lightgrey; stroke:lightgrey; stroke-width:1px",
    );

    svg.set_style(
        Layer::ParkBuilding,
        "park-building",
        "fill:#617d61; stroke:#617d61; stroke-width:1px",
    );

    svg.set_style(
        Layer::Coastline,
        "coastline",
        "fill:grey; stroke:white; stroke-width:1px",
    );
    svg.set_style(Layer::Park, "park", "fill: #adbfad; stroke:none;");

    for kind in &results {
        let layer = kind.to_layer();
        let coords = kind.resolve_coords(&geometry);
        svg.draw_polyline(layer, coords)?;
        if let Layer::Building = layer {
            svg.draw_polyline(Layer::ParkBuilding, coords)?;
        }
    }

    let layer_order = &[
        /*
        Layer::Park,
        Layer::Road,
        Layer::Building,
        Layer::ParkBuilding,*/
        Layer::Coastline,
    ];
    svg.export_to_file("./nyc.svg", layer_order)?;
    flame::dump_html(std::fs::File::create("./flame.html")?)?;

    Ok(())
}
