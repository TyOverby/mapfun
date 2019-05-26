use osm_load::Bounds;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;


pub struct Svg<T: Hash + Eq> {
    bounds: Bounds,
    layers: HashMap<T, Vec<u8>>,
    styles: HashMap<T, (String, String)>,
}

impl<T: Hash + Eq> Svg<T> {
    pub fn new(bounds: Bounds) -> Svg<T> {
        Svg {
            bounds,
            layers: HashMap::new(),
            styles: HashMap::new(),
        }
    }

    pub fn set_style(&mut self, layer: T, classname: &str, style: &str) {
        self.styles.insert(layer, (classname.into(), style.into()));
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

    pub fn export_to_file(&self, file: &str, layer_order: &[T]) -> std::io::Result<()> {
        let file = std::fs::File::create(file)?
        let mut file = std::io::BufWriter::new(file);

        writeln!(
            file,
            r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
            self.bounds.width, self.bounds.height
        )?;

        writeln!(file, "<style>")?;
        for (_, (classname, style)) in self.styles.iter() {
            writeln!(file, ".{} {{{}}}", classname, style)?;
        }
        writeln!(file, "</style>")?;

        for layer in layer_order {
            if let Some(layer) = self.layers.get(layer) {
                file.write_all(&layer)?;
            }
        }

        writeln!(file, "</svg>")?;
        Ok(())
    }
}
