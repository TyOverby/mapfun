use osm_load::Bounds;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;

pub struct Svg<T: Hash + Eq> {
    bounds: Bounds,
    layers: HashMap<T, Vec<u8>>,
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

    pub fn draw_polyline(&mut self, layer: T, polyline: &[(f64, f64)]) -> std::io::Result<()> {
        if polyline.len() == 0 {
            return Ok(());
        };
        let complete = polyline[0] == polyline[polyline.len() - 1];
        let style_class = self.styles.get(&layer);
        let layer = self.layers.entry(layer).or_insert_with(|| vec![]);
        match style_class {
            Some((class, _)) => write!(layer, r#"<path class="{}" d=""#, class)?,
            None => write!(layer, r#"<path d=""#)?,
        }
        let mut first = true;
        for (lon, lat) in polyline {
            let (lon, lat) = self
                .bounds
                .transform_lat_lon_to_screen_coordinate((*lon, *lat));
            let movement = if first { "M" } else { "L" };
            first = false;
            write!(layer, "{}{},{} ", movement, lon, self.bounds.height - lat)?;
        }
        if complete {
            write!(layer, "z")?;
        }
        writeln!(layer, r#"" />"#)?;
        Ok(())
    }

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
        if let Some(layer) = self.layers.get(layer) {
            file.write_all(&layer)?;
        }
if should_print_group {
        writeln!(file, "</g>")?;
}
        Ok(())
    }

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
            writeln!(file, "svg {{background-color: {}}}", background_color)?;
        }

        for (_, (classname, style)) in self.styles.iter() {
            writeln!(file, ".{} {{{}}}", classname, style)?;
        }
        writeln!(file, "</style>")?;

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
