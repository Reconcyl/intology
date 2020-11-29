use png::Encoder;

use std::io::Write;
use std::convert::TryInto;

use crate::expr::IExpr;

/// Encode a buffer of pixel data as a PNG and write it to `w`.
pub fn write_rgba_image_data(w: impl Write, width: u32, height: u32, data: &[u8]) {
    let mut encoder = Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap()
        .write_image_data(data).unwrap();
}

impl IExpr {
    pub fn write_image_data(&self, w: impl Write, width: u32, height: u32, scale: u32) {
        let image_width = scale * width;
        let image_height = scale * height;

        let mut data        = Vec::with_capacity((image_width * image_height * 4) as usize);
        let mut current_row = Vec::with_capacity((image_width * 4) as usize);

        let batch = (0..height).flat_map(|y| (0..width).map(move |x| (x as i32, y as i32)));
        let colors = self.eval_batch(batch);

        for (i, [r, g, b]) in colors.enumerate() {
            for _ in 0..scale {
                current_row.extend(&[
                    r.try_into().unwrap(),
                    g.try_into().unwrap(),
                    b.try_into().unwrap(),
                    0xff]);
            }
            if (i + 1) % (width as usize) == 0 {
                for _ in 0..scale {
                    data.extend(&current_row);
                }
                current_row.clear();
            }
        }

        write_rgba_image_data(w, image_width, image_height, &data)
    }
}
