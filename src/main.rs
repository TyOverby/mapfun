extern crate flame;
extern crate osm_xml;
extern crate proj5;

#[macro_use]
extern crate flamer;
mod osm_load;
mod svg_exporter;
use osm_load::*;
use svg_exporter::*;

#[derive(Clone, Copy)]
enum Kind {
    Building(RangeIdx),
    Road(RangeIdx),
    Coastline(RangeIdx),
    Park(RangeIdx),
}

#[derive(Hash, Eq, PartialEq)]
enum Layer {
    Building,
    Road,
    Coastline,
    Park,
}

impl Kind {
    fn to_layer(&self) -> Layer {
        match self {
            Kind::Building(_) => Layer::Building,
            Kind::Road(_) => Layer::Road,
            Kind::Coastline(_) => Layer::Coastline,
            Kind::Park(_) => Layer::Park,
        }
    }
    fn range_idx(&self) -> RangeIdx {
        match self {
            Kind::Building(r) | Kind::Road(r) | Kind::Coastline(r) | Kind::Park(r) => *r,
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

fn main() -> std::io::Result<()> {
    let geometry = Geometry::from_file("./nyc.osm", &filter, 1000.0);
    let bounds = geometry.bounds;

    let mut svg = Svg::new(bounds);

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
        Layer::Coastline,
        "coastline",
        "fill:none; stroke:black; stroke-width:0.1%",
    );
    svg.set_style(Layer::Park, "park", "fill: #adbfad; stroke:none;");

    for kind in &geometry.results {
        let layer = kind.to_layer();
        let range = kind.range_idx();
        svg.draw_polyline(layer, geometry.resolve_coords(range))?;
    }

    let layer_order = &[Layer::Park, Layer::Road, Layer::Building, Layer::Coastline];
    svg.export_to_file("./nyc.svg", layer_order)?;
    flame::dump_html(std::fs::File::create("./flame.html")?)?;

    Ok(())
}
