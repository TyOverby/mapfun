use osm_xml::{Member, Reference, Way, OSM};

use proj5::FromLonLat;
use proj5::{crs::MercatorSystem, *};

use std::fs::File;
use std::io::BufReader;
use std::ops::Range;

pub use osm_xml::Tag;

pub type Callback<'a, T> = &'a Fn(&[Tag], &[Tag], RangeIdx) -> Option<T>;
pub type RangeIdx = usize;

pub struct Geometry {
    pub bounds: Bounds,
    pub coords: Vec<(f64, f64)>,
    pub polys: Vec<Range<usize>>,
}

#[derive(Copy, Clone)]
pub struct Bounds {
    pub width: f64,
    pub height: f64,
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
    pub scale_x: f64,
    pub scale_y: f64,
}

impl Geometry {
    pub fn resolve_coords(&self, range_idx: RangeIdx) -> &[(f64, f64)] {
        &self.coords[self.polys[range_idx].clone()]
    }

    #[flame]
    pub fn from_file<T>(path: &str, callback: Callback<T>, target_h: f64) -> (Geometry, Vec<T>) {
        let f = File::open(path).unwrap();
        let br = BufReader::new(f);
        let doc = flame::span_of("reading osm data", || OSM::parse(br).unwrap());
        let bounds = doc.bounds.unwrap();

        let bounds_converted = coord_convert(vec![
            (bounds.maxlon, bounds.maxlat),
            (bounds.minlon, bounds.minlat),
        ]);
        let (b_max_lon, b_max_lat) = bounds_converted[0];
        let (b_min_lon, b_min_lat) = bounds_converted[1];
        let target_w = ((b_max_lon - b_min_lon) / (b_max_lat - b_min_lat)) * target_h;
        let scale_x = target_w / (b_max_lon - b_min_lon);
        let scale_y = target_h / (b_max_lat - b_min_lat);

        let mut all_coords = Vec::new();
        let mut all_polys = Vec::new();
        let mut all_values = Vec::new();

        flame::span_of("finding relations", || {
            for rel in doc.relations.values() {
                let rel_tags = &rel.tags;
                for member in &rel.members {
                    if let &Member::Way(ref reference, _) = member {
                        let member = doc.resolve_reference(reference);
                        if let Reference::Way(way) = member {
                            collect_ways(
                                Some(rel_tags),
                                way,
                                callback,
                                &mut all_coords,
                                &mut all_polys,
                                &mut all_values,
                                &doc,
                            );
                        }
                    }
                }
            }
        });

        flame::span_of("finding ways", || {
            for way in doc.ways.values() {
                collect_ways(
                    None,
                    way,
                    callback,
                    &mut all_coords,
                    &mut all_polys,
                    &mut all_values,
                    &doc,
                );
            }
        });

        let all_coords = coord_convert(all_coords);

        (
            Geometry {
                bounds: Bounds {
                    min_lon: b_min_lon,
                    min_lat: b_min_lat,
                    width: target_w,
                    height: target_h,
                    max_lon: b_max_lon,
                    max_lat: b_max_lat,
                    scale_x,
                    scale_y,
                },
                coords: all_coords,
                polys: all_polys,
            },
            all_values,
        )
    }
}

impl Bounds {
    pub fn transform_lat_lon_to_screen_coordinate(&self, (lon, lat): (f64, f64)) -> (f64, f64) {
        (
            (lon - self.min_lon) * self.scale_x,
            (lat - self.min_lat) * self.scale_y,
        )
    }
}
pub fn simple_filterer<T, F>(f: F) -> impl Fn(&[Tag], &[Tag], RangeIdx) -> Option<T>
where
    F: Fn((&str, &str)) -> Option<fn(RangeIdx) -> T>,
{
    move |relationship_tags, way_tags, range| {
        for &tags in &[relationship_tags, way_tags] {
            for tag in tags {
                let computed = f((&tag.key, &tag.val));
                if computed.is_some() {
                    return computed.map(|f| f(range));
                }
            }

        }
        None
    }
}
fn collect_ways<T>(
    relationship_tags: Option<&[Tag]>,
    way: &Way,
    callback: Callback<T>,
    all_coords: &mut Vec<(f64, f64)>,
    all_polys: &mut Vec<Range<usize>>,
    all_values: &mut Vec<T>,
    doc: &OSM,
) {
    let tags = &way.tags;
    let start = all_coords.len();
    let relationship_tags = relationship_tags.unwrap_or(&[]);

    if let Some(v) = callback(relationship_tags, tags, all_polys.len()) {
        all_values.push(v);
        for node in &way.nodes {
            let node = doc.resolve_reference(node);
            if let Reference::Node(node) = node {
                all_coords.push((node.lon, node.lat));
            }
        }
        let end = all_coords.len();
        all_polys.push(start..end);
    }
}

#[flame]
fn coord_convert(input: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let ellipsoid = WGS_1984_ELLIPSOID;
    //let system = UTMSystem { utm_zone: 10 };
    let system = MercatorSystem;

    let mut strategy = MultithreadingStrategy::MultiCore(ThreadPool::new(8));
    let out = system.from_lon_lat(input, &ellipsoid, &mut strategy);
    out.data
}
