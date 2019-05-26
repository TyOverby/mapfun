
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;

type Item = String;

pub struct Svg<T: Hash + Eq> {
    width: f64,
    height: f64,
    layers: HashMap<T, Vec<Item>>,
}

impl<T: Hash + Eq> Svg<T> {
    pub fn new(width: f64, height: f64) -> Svg<T> {
        Svg {
            width,
            height,
            layers: HashMap::new(),
        }
    }

    pub fn draw_to(&mut self, layer: T, item: String) {
        self.layers
            .entry(layer)
            .or_insert_with(|| vec![])
            .push(item)
    }

    pub fn export_to_file(&self, file: &str, layer_order: &[T]) -> std::io::Result<()> {
        let file = std::fs::File::create(file).unwrap();
        let mut file = std::io::BufWriter::new(file);

        writeln!(
            file,
            r#"<svg viewBox="0 0 {} {} " xmlns="http://www.w3.org/2000/svg">"#,
            self.width, self.height
        )?;

        for layer in layer_order {
            if let Some(layer) = self.layers.get(layer) {
                for item in layer {
                    writeln!(file, "{}", item)?;
                }
            }
        }

        writeln!(file, "</svg>")?;
        Ok(())
    }
}
