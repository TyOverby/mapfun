extern crate flame;
extern crate osm_xml;
extern crate proj5;

#[macro_use]
extern crate flamer;
mod osm_load;
use osm_load::*;

use std::ops::Range;

const TARGET_H: f64 = 1000.0f64;

enum Kind {
    Building(Range<usize>),
    Road(Range<usize>),
}

fn filter(_relationship_tags: &[Tag], way_tags: &[Tag], range: Range<usize>) -> Option<Kind> {
    if way_tags.iter().any(|tag| tag.key == "highway") {
        Some(Kind::Road(range))
    } else if way_tags.iter().any(|tag| tag.key == "building") {
        Some(Kind::Building(range))
    } else {
        None
    }
}
fn print_path<I>(path: I, is_building: bool)
where
    I: Iterator<Item = (f64, f64)>,
{
    if is_building {
        print!(r#"<path style="fill:lightgrey; stroke:lightgrey; stroke-width:1px" d=""#);
    } else {
        print!(r#"<path style="fill:none; stroke:darkgrey; stroke-width:12px; stroke-linecap:round" d=""#);
    }
    let mut first = true;
    for (lon, lat) in path {
        let movement = if first { "M" } else { "L" };
        first = false;
        print!("{}{},{} ", movement, lon, TARGET_H - lat);
    }
    println!(r#"" />"#);
}

fn main() {
    let Geometry {
        bounds,
        coords,
        polys,
        results,
    } = load_osm_file("./nyc.osm", &filter, 1000.0);
    let osm_load::Bounds {
        width,
        height,
        min_lon,
        min_lat,
        scale_x,
        scale_y,
        ..
    } = bounds;

    println!(
        r#"<svg viewBox="0 0 {} {} " xmlns="http://www.w3.org/2000/svg">"#,
        width, height
    );

    let transform = |&(lon, lat)| ((lon - min_lon) * scale_x, (lat - min_lat) * scale_y);

    for kind in &results {
        match kind {
            Kind::Road(range) => {
                let range = range.clone();
                let scaled = coords[range].iter().map(transform);
                print_path(scaled, false)
            }
            _ => {}
        }

    }
    for kind in &results {
        match kind {
            Kind::Building(range) => {
                let range = range.clone();
                let scaled = coords[range].iter().map(transform);
                print_path(scaled, true)
            }
            _ => {}
        }
    }

    /*

    for poly in polys {
        let scaled = coords[poly]
            .iter()
            .map(|&(lon, lat)| ((lon - min_lon) * scale_x, (lat - min_lat) * scale_y));
        print_path(scaled);
    }
    */

    println!("</svg>");

    flame::dump_html(std::fs::File::create("./flame.html").unwrap()).unwrap();
}
