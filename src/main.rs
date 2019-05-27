extern crate flame;
extern crate osm_xml;
extern crate proj5;

#[macro_use]
extern crate flamer;
mod osm_load;
mod svg_exporter;
use osm_load::*;
use svg_exporter::*;

#[derive(Clone, PartialEq)]
enum Kind {
    Building(RangeIdx),
    Road(RangeIdx),
    Coastline(RangeIdx),
    CoastlineProcessed(Vec<(f64, f64)>),
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
            Kind::CoastlineProcessed(_) => Layer::Coastline,
            Kind::Park(_) => Layer::Park,
        }
    }
    fn range_idx(&self) -> RangeIdx {
        match self {
            Kind::Building(r) | Kind::Road(r) | Kind::Coastline(r) | Kind::Park(r) => *r,
            Kind::CoastlineProcessed(_) => 0,
        }
    }
}

fn filter(_relationship_tags: &[Tag], way_tags: &[Tag], range: RangeIdx) -> Option<Kind> {
    // if way_tags.iter().any(|tag| tag.key == "highway") {
    //     Some(Kind::Road(range))
    // } else if way_tags.iter().any(|tag| tag.key == "building") {
    //     Some(Kind::Building(range))
    // } else if way_tags.iter().any(|tag| tag.val == "coastline") {
    //     Some(Kind::Coastline(range))
    // } else if way_tags.iter().any(|tag| tag.val == "park") {
    //     Some(Kind::Park(range))
    // } else {
    //     None
    // }
    if way_tags.iter().any(|tag| tag.val == "coastline") {
        Some(Kind::Coastline(range))
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Vector2 {
    x: f64,
    y: f64,
}

impl Vector2 {
    fn distance(&self, other: &Vector2) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

fn main() -> std::io::Result<()> {
    let geometry = load_osm_file("./nyc.osm", &filter, 2000.0);
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
        "fill:lightgray; fill-rule:evenodd; stroke:black; stroke-width:0.1%",
    );
    svg.set_style(Layer::Park, "park", "fill: #adbfad; stroke:none;");

    let mut coastlines: Vec<Vec<Vector2>> = geometry
        .results
        .iter()
        .filter_map(|result| match result {
            Kind::Coastline(range) => Some(range),
            _ => None,
        })
        .map(|range| {
            geometry
                .resolve_coords(*range)
                .to_vec()
                .iter()
                .map(|(x, y)| Vector2 { x: *x, y: *y })
                .collect()
        })
        .collect();
    eprintln!("{}", coastlines.len());

    // Dedupe lines
    coastlines.dedup_by(|a, b| {
        let same_length = a.len() == b.len();
        let same_first_point = a[0].distance(&b[0]) <= 0.0001;
        println!("{}", same_length && same_first_point);
        same_length && same_first_point
    });
    println!("{}", coastlines.len());

    let mut should_be_combined = Vec::new();
    // Compare start point of a given line to every other line
    let magic_closeness_threshold = 0.01;
    for (i, line_i) in coastlines.iter().enumerate() {
        for (j, line_j) in coastlines.iter().enumerate() {
            if line_i[0].distance(&line_j[line_j.len() - 1]) < magic_closeness_threshold {
                println!("we should combine these: {} {} ", i, j);
                should_be_combined.push((i, j));
            }
        }
    }

    for kind in &geometry.results {
        match kind {
            Kind::Coastline(_) => (),
            _ => {
                let layer = kind.to_layer();
                let range = kind.range_idx();
                let points = geometry.resolve_coords(range);
                svg.draw_polyline(layer, points)?;
            }
        }
    }

    let layer_order = &[Layer::Road, Layer::Building, Layer::Coastline, Layer::Park];
    svg.export_to_file("./nyc.svg", layer_order)?;
    flame::dump_html(std::fs::File::create("./flame.html")?)?;

    Ok(())
}
