extern crate flame;
extern crate osm_xml;
extern crate proj5;

#[macro_use]
extern crate flamer;
mod osm_load;
mod svg_exporter;
use osm_load::*;
use svg_exporter::*;

enum Kind {
    Building(RangeIdx),
    Road(RangeIdx),
}

#[derive(Hash, Eq, PartialEq)]
enum Layer {
    Building,
    Road,
}

fn filter(_relationship_tags: &[Tag], way_tags: &[Tag], range: RangeIdx) -> Option<Kind> {
    if way_tags.iter().any(|tag| tag.key == "highway") {
        Some(Kind::Road(range))
    } else if way_tags.iter().any(|tag| tag.key == "building") {
        Some(Kind::Building(range))
    } else {
        None
    }
}

fn main() -> std::io::Result<()> {
    let geometry = load_osm_file("./nyc.osm", &filter, 1000.0);
    let bounds = geometry.bounds;

    let mut svg = Svg::new(bounds);

    svg.set_style(
        Layer::Road,
        "road",
        "fill:none; stroke:darkgrey; stroke-width:12px; stroke-linecap:round",
    );

    svg.set_style(
        Layer::Building,
        "building",
        "fill:lightgrey; stroke:lightgrey; stroke-width:1px",
    );

    for kind in &geometry.results {
        match kind {
            Kind::Road(range) => svg.draw_polyline(Layer::Road, geometry.resolve_coords(*range)),
            Kind::Building(range) => {
                svg.draw_polyline(Layer::Building, geometry.resolve_coords(*range))
            }
        }
    }
    svg.export_to_file("./nyc.svg", &[Layer::Road, Layer::Building])?;
    flame::dump_html(std::fs::File::create("./flame.html")?)?;

    Ok(())
}
