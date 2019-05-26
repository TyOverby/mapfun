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
fn print_path<I>(path: I, bounds: &Bounds, layer: Layer) -> String
where
    I: Iterator<Item = (f64, f64)>,
{
    use std::io::Write;

    let path = path.map(|a| bounds.transform_lat_lon_to_screen_coordinate(a));

    let s =  match layer {
        Layer::Building => r#"<path style="fill:lightgrey; stroke:lightgrey; stroke-width:1px" d=""#,
        Layer::Road => r#"<path style="fill:none; stroke:darkgrey; stroke-width:12px; stroke-linecap:round" d=""#,
    };
    let mut s = s.to_string().into_bytes();

    let mut first = true;
    for (lon, lat) in path {
        let movement = if first { "M" } else { "L" };
        first = false;
        write!(s, "{}{},{} ", movement, lon, bounds.height - lat).unwrap();
    }
    write!(s, r#"" />"#).unwrap();
    String::from_utf8(s).unwrap()
}

fn main() {
    let geometry = load_osm_file("./nyc.osm", &filter, 1000.0);
    let bounds = geometry.bounds;

    let mut svg = Svg::new(bounds.width, bounds.height);

    for kind in &geometry.results {
        match kind {
            Kind::Road(range) => svg.draw_to(
                Layer::Road,
                print_path(
                    geometry.resolve_coords(*range).iter().cloned(),
                    &bounds,
                    Layer::Road,
                ),
            ),
            Kind::Building(range) => svg.draw_to(
                Layer::Building,
                print_path(
                    geometry.resolve_coords(*range).iter().cloned(),
                    &bounds,
                    Layer::Building,
                ),
            ),
        }

    }

    svg.export_to_file("./nyc.svg", &[Layer::Road, Layer::Building])
        .unwrap();
    flame::dump_html(std::fs::File::create("./flame.html").unwrap()).unwrap();
}
