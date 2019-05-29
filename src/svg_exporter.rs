use osm_load::Bounds;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;

enum Element {
    LineSegment { points: Vec<(f64, f64)> },
    Polygon { points: Vec<(f64, f64)> },
}

pub struct Svg<T: Hash + Eq> {
    bounds: Bounds,
    layers: HashMap<T, Vec<Element>>,
    styles: HashMap<T, (String, String)>,
    clippings: HashMap<T, T>,
    background_color: Option<String>,
}

impl<T: Hash + Eq> Svg<T> {
    pub fn new(bounds: Bounds) -> Svg<T> {
        Svg {
            bounds,
            layers: HashMap::new(),
            styles: HashMap::new(),
            clippings: HashMap::new(),
            background_color: None,
        }
    }

    pub fn set_background_color(&mut self, color: &str) {
        self.background_color = Some(color.into());
    }

    pub fn set_style(&mut self, layer: T, classname: &str, style: &str) {
        self.styles.insert(layer, (classname.into(), style.into()));
    }

    pub fn set_clippings_layer(&mut self, layer: T, clipped_by: T) {
        self.clippings.insert(layer, clipped_by);
    }

    pub fn draw_polyline(&mut self, layer: T, polyline: &[(f64, f64)]) {
        let len = polyline.len();
        if len == 0 || len == 1 {
            return;
        }

        let transformed = polyline
            .iter()
            .map(|(lon, lat)| {
                self.bounds
                    .transform_lat_lon_to_screen_coordinate((*lon, *lat))
            })
            .collect();
        let layer = self.layers.entry(layer).or_insert_with(|| vec![]);
        if polyline[0] == polyline[len - 1] {
            layer.push(Element::Polygon {
                points: transformed,
            })
        } else {
            layer.push(Element::LineSegment {
                points: transformed,
            })
        }
    }
    fn draw_element<W: Write>(
        &self,
        style_class: Option<String>,
        out: &mut W,
        element: &Element,
    ) -> std::io::Result<()> {
        match element {
            Element::LineSegment { points } | Element::Polygon { points } if points.is_empty() => {
                return Ok(())
            }
            _ => (),
        }

        let draw_polyline = |polyline: &[(f64, f64)]| -> std::io::Result<()> {
            match style_class {
                Some(class) => write!(out, r#"<path class="{}" d=""#, class)?,
                None => write!(out, r#"<path d=""#)?,
            }
            let mut first = true;
            for (x, y) in polyline {
                let movement = if first { "M" } else { "L" };
                first = false;
                write!(out, "{}{:.2},{:.2} ", movement, x, self.bounds.height - y)?;
            }
            Ok(())
        };

        match element {
            Element::LineSegment { points } => draw_polyline(points)?,
            Element::Polygon { points } => {
                draw_polyline(points)?;
                write!(out, "z")?;
            }
        }

        writeln!(out, r#"" />"#)?;
        Ok(())
    }

    #[flame]
    fn export_layer<W: Write>(
        &self,
        layer: &T,
        should_print_group: bool,
        file: &mut W,
    ) -> std::io::Result<()> {
        let additional_info = if let Some(clipped_by) = self.clippings.get(layer) {
            let id = get_unique_id();
            writeln!(file, r#"<clipPath id="{}">"#, id)?;
            self.export_layer(clipped_by, false, file)?;
            writeln!(file, "</clipPath>")?;
            format!(r#"clip-path="url(#{})""#, id)
        } else {
            "".into()
        };

        if should_print_group {
            writeln!(file, "<g {}>", additional_info)?;
        }
        if let Some(elements) = self.layers.get(layer) {
            for element in elements {
                let style = self.styles.get(layer).cloned().map(|(a, _)| a);
                self.draw_element(style, file, element)?;
            }
        }
        if should_print_group {
            writeln!(file, "</g>")?;
        }
        Ok(())
    }

    /*fn draw_clipped_elements(draw: &[Element], clip: &[Element]) -> std::io::Result<()> {
        //let aabb = aabb_quadtree::QuadTree::default(unimplemented!(), clip.len());
        unimplemented!();
    }*/

    #[flame]
    pub fn export_to_file(&self, file: &str, layer_order: &[T]) -> std::io::Result<()> {
        let file = std::fs::File::create(file)?;
        let mut file = std::io::BufWriter::new(file);

        writeln!(
            file,
            r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
            self.bounds.width, self.bounds.height
        )?;

        writeln!(file, "<style>")?;
        if let Some(background_color) = &self.background_color {
            writeln!(file, ".background {{fill: {}}}", background_color)?;
        }

        for (_, (classname, style)) in self.styles.iter() {
            writeln!(file, ".{} {{{}}}", classname, style)?;
        }
        writeln!(file, "</style>")?;

        if self.background_color.is_some() {
            writeln!(
                file,
                r#"<rect class="background" x="0" y="0" width="{}" height="{}" />"#,
                self.bounds.width, self.bounds.height
            )?;
        }

        for layer in layer_order {
            self.export_layer(layer, true, &mut file)?;
        }

        writeln!(file, "</svg>")?;
        Ok(())
    }
}

use std::sync::atomic::{AtomicU32, Ordering};
static mut ID: AtomicU32 = AtomicU32::new(0);

fn get_unique_id() -> String {
    let id = unsafe { ID.fetch_add(1, Ordering::Relaxed) };
    return format!("a_{}", id);
}
