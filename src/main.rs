extern crate osm_xml;
extern crate proj5;

use osm_xml::{Member, Reference, Tag, Way, OSM};

use proj5::FromLonLat;
use proj5::{crs::MercatorSystem, *};

use std::fs::File;
use std::io::BufReader;
use std::ops::Range;
const TARGET_H: f64 = 1000.0f64;

fn filter(tag: &Tag) -> bool {
    eprintln!("k {} v {}", tag.key, tag.val);
    eprintln!("key {}", tag.key);
    eprintln!("val {}", tag.val);
    match tag.key.as_str() {
        "building" | "pier" | "lanes" | "highway" | "subway" => return true,
        _ => (),
    };
    match tag.val.as_str() {
        "pier" | "waterway" | "riverbank" | "water" | "coastline" => true,
        _ => false,
    }
}

fn convert(input: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let ellipsoid = WGS_1984_ELLIPSOID;
    //let system = UTMSystem { utm_zone: 10 };
    let system = MercatorSystem;

    let mut strategy = MultithreadingStrategy::MultiCore(ThreadPool::new(8));
    let out = system.from_lon_lat(input, &ellipsoid, &mut strategy);
    out.data
}
fn print_path<I>(path: I)
where
    I: Iterator<Item = (f64, f64)>,
{
    print!(r#"<path style="fill:none; stroke:black;" d=""#);
    let mut first = true;
    for (lon, lat) in path {
        let movement = if first { "M" } else { "L" };
        first = false;
        print!("{}{},{} ", movement, lon, TARGET_H - lat);
    }
    println!(r#"" />"#);
}

fn print_way(
    way: &Way,
    always_print: bool,
    all_coords: &mut Vec<(f64, f64)>,
    all_polys: &mut Vec<Range<usize>>,
    doc: &OSM,
) {
    if !always_print && !way.tags.iter().any(filter) {
        return;
    }
    let start = all_coords.len();

    for node in &way.nodes {
        let node = doc.resolve_reference(node);
        if let Reference::Node(node) = node {
            all_coords.push((node.lon, node.lat));
        }
    }
    let end = all_coords.len();
    all_polys.push(start..end);
}

fn main() {
    // curl 'https://www.openstreetmap.org/api/0.6/map?bbox=-73.9846%2C40.6791%2C-73.9737%2C40.6911' > nyc.osm
    let f = File::open("./nyc.osm").unwrap();
    let br = BufReader::new(f);
    eprintln!("reading");
    let doc = OSM::parse(br).unwrap();
    eprintln!(
        "{} nodes, {} relations, {} ways",
        doc.nodes.len(),
        doc.relations.len(),
        doc.ways.len()
    );
    eprintln!("done reading");

    let bounds = doc.bounds.unwrap();

    let bounds_converted = convert(vec![
        (bounds.maxlon, bounds.maxlat),
        (bounds.minlon, bounds.minlat),
    ]);
    let (b_max_lon, b_max_lat) = bounds_converted[0];
    let (b_min_lon, b_min_lat) = bounds_converted[1];
    let target_w = ((b_max_lon - b_min_lon) / (b_max_lat - b_min_lat)) * TARGET_H;
    let scale_x = target_w / (b_max_lon - b_min_lon);
    let scale_y = TARGET_H / (b_max_lat - b_min_lat);

    println!(
        r#"<svg viewBox="0 0 {} {} " xmlns="http://www.w3.org/2000/svg">"#,
        target_w, TARGET_H
    );

    let mut all_coords = Vec::new();
    let mut all_polys = Vec::new();

    eprintln!("finding relations");

    for rel in doc.relations.values() {
        if !rel.tags.iter().any(filter) {
            continue;
        }
        for member in &rel.members {
            if let &Member::Way(ref reference, _) = member {
                let member = doc.resolve_reference(reference);
                if let Reference::Way(way) = member {
                    print_way(way, true, &mut all_coords, &mut all_polys, &doc);
                }
            }
        }
    }
    eprintln!("done finding relations");
    eprintln!("finding ways");

    for way in doc.ways.values() {
        print_way(way, false, &mut all_coords, &mut all_polys, &doc);
    }
    eprintln!("done finding ways");

    eprintln!("converting");
    let all_coords = convert(all_coords);
    eprintln!("done converting");

    for poly in all_polys {
        let scaled = all_coords[poly]
            .iter()
            .map(|&(lon, lat)| ((lon - b_min_lon) * scale_x, (lat - b_min_lat) * scale_y));
        print_path(scaled);
    }

    println!("</svg>");
}
