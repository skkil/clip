use crate::{Box2, Error, Result};

#[derive(Debug, Clone)]
pub struct Bgr {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Bgr {
    pub fn from_rgba(rgba: &[u8], width: usize, height: usize) -> Result<Self> {
        if width == 0 || height == 0 || rgba.len() != width * height * 4 {
            return Err(Error::Image(format!(
                "expected {}x{} RGBA ({} bytes), got {} bytes",
                width,
                height,
                width * height * 4,
                rgba.len()
            )));
        }
        let data = rgba.as_chunks::<4>().0.iter().flat_map(composite).collect();
        Ok(Self {
            width,
            height,
            data,
        })
    }

    fn pixel(&self, x: usize, y: usize) -> [f32; 3] {
        let i = (y * self.width + x) * 3;
        [
            self.data[i] as f32,
            self.data[i + 1] as f32,
            self.data[i + 2] as f32,
        ]
    }

    pub fn crop(&self, b: Box2) -> Bgr {
        let x0 = (b.x.floor().max(0.0) as usize).min(self.width - 1);
        let y0 = (b.y.floor().max(0.0) as usize).min(self.height - 1);
        let x1 = ((b.x + b.w).ceil() as usize).clamp(x0 + 1, self.width);
        let y1 = ((b.y + b.h).ceil() as usize).clamp(y0 + 1, self.height);
        let (width, height) = (x1 - x0, y1 - y0);
        let mut data = Vec::with_capacity(width * height * 3);
        for y in y0..y1 {
            let row = (y * self.width + x0) * 3;
            data.extend_from_slice(&self.data[row..row + width * 3]);
        }
        Bgr {
            width,
            height,
            data,
        }
    }

    pub fn resized(&self, width: usize, height: usize) -> Vec<[f32; 3]> {
        let sx = self.width as f32 / width as f32;
        let sy = self.height as f32 / height as f32;
        let mut out = Vec::with_capacity(width * height);
        for y in 0..height {
            let fy = ((y as f32 + 0.5) * sy - 0.5).clamp(0.0, (self.height - 1) as f32);
            let (y0, ty) = (fy.floor() as usize, fy.fract());
            let y1 = (y0 + 1).min(self.height - 1);
            for x in 0..width {
                let fx = ((x as f32 + 0.5) * sx - 0.5).clamp(0.0, (self.width - 1) as f32);
                let (x0, tx) = (fx.floor() as usize, fx.fract());
                let x1 = (x0 + 1).min(self.width - 1);
                let (a, b, c, d) = (
                    self.pixel(x0, y0),
                    self.pixel(x1, y0),
                    self.pixel(x0, y1),
                    self.pixel(x1, y1),
                );
                out.push(std::array::from_fn(|k| {
                    let top = a[k] + (b[k] - a[k]) * tx;
                    let bottom = c[k] + (d[k] - c[k]) * tx;
                    top + (bottom - top) * ty
                }));
            }
        }
        out
    }
}

fn composite(p: &[u8; 4]) -> [u8; 3] {
    let a = p[3] as u16;
    let over_white = |c: u8| ((c as u16 * a + 255 * (255 - a)) / 255) as u8;
    [over_white(p[2]), over_white(p[1]), over_white(p[0])]
}
