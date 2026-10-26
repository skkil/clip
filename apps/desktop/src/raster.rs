use anyhow::{Context, Result, bail};

use crate::geom::{Rect, Transform};

pub type Rgba = [u8; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Bgra,
    Bgrx,
    Rgba,
    Rgbx,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width as usize * height as usize * 4],
        }
    }

    pub fn from_raw(
        width: u32,
        height: u32,
        stride: usize,
        format: SourceFormat,
        data: &[u8],
    ) -> Result<Self> {
        let row_bytes = width as usize * 4;
        if stride < row_bytes || data.len() < stride * height as usize {
            bail!(
                "buffer of {} bytes is too small for {width}x{height} stride {stride}",
                data.len()
            );
        }
        let mut pixels = vec![0u8; row_bytes * height as usize];
        for (dst, src) in pixels
            .chunks_exact_mut(row_bytes)
            .zip(data.chunks_exact(stride))
        {
            let pairs = dst
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(src[..row_bytes].as_chunks::<4>().0);
            match format {
                SourceFormat::Bgra => pairs.for_each(|(d, s)| *d = [s[2], s[1], s[0], s[3]]),
                SourceFormat::Bgrx => pairs.for_each(|(d, s)| *d = [s[2], s[1], s[0], 255]),
                SourceFormat::Rgba => pairs.for_each(|(d, s)| *d = *s),
                SourceFormat::Rgbx => pairs.for_each(|(d, s)| *d = [s[0], s[1], s[2], 255]),
            }
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn from_png(bytes: &[u8]) -> Result<Self> {
        let decoded = image::load_from_memory(bytes)
            .context("decode image")?
            .to_rgba8();
        Ok(Self {
            width: decoded.width(),
            height: decoded.height(),
            pixels: decoded.into_raw(),
        })
    }

    pub fn to_png(&self) -> Result<Vec<u8>> {
        let mut out = std::io::Cursor::new(Vec::new());
        image::write_buffer_with_format(
            &mut out,
            &self.pixels,
            self.width,
            self.height,
            image::ExtendedColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .context("encode png")?;
        Ok(out.into_inner())
    }

    pub fn stride(&self) -> usize {
        self.width as usize * 4
    }

    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    fn index(&self, x: i32, y: i32) -> usize {
        y as usize * self.stride() + x as usize * 4
    }

    pub fn pixel(&self, x: u32, y: u32) -> Rgba {
        let i = self.index(x as i32, y as i32);
        self.pixels[i..i + 4].try_into().unwrap()
    }

    pub fn crop(&self, rect: Rect) -> Result<Self> {
        let r = rect.intersect(&self.rect());
        if r.is_empty() {
            bail!(
                "crop {rect:?} lies outside the {}x{} image",
                self.width,
                self.height
            );
        }
        let mut pixels = Vec::with_capacity(r.area() as usize * 4);
        for y in r.y..r.bottom() {
            let start = self.index(r.x, y);
            pixels.extend_from_slice(&self.pixels[start..start + r.w as usize * 4]);
        }
        Ok(Self {
            width: r.w,
            height: r.h,
            pixels,
        })
    }

    pub fn flipped_vertically(&self) -> Self {
        let pixels = self
            .pixels
            .chunks_exact(self.stride())
            .rev()
            .flatten()
            .copied()
            .collect();
        Self { pixels, ..*self }
    }

    pub fn transformed(self, transform: Transform) -> Self {
        if transform == Transform::Normal {
            return self;
        }
        let (w, h) = if transform.swaps_axes() {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        let (mw, mh) = (self.width - 1, self.height - 1);
        let mut out = Image::new(w, h);
        for y in 0..self.height {
            for x in 0..self.width {
                let (nx, ny) = match transform {
                    Transform::Normal => (x, y),
                    Transform::Rotate90 => (mh - y, x),
                    Transform::Rotate180 => (mw - x, mh - y),
                    Transform::Rotate270 => (y, mw - x),
                    Transform::Flipped => (mw - x, y),
                    Transform::FlippedRotate90 => (y, x),
                    Transform::FlippedRotate180 => (x, mh - y),
                    Transform::FlippedRotate270 => (mh - y, mw - x),
                };
                let (src, dst) = (
                    self.index(x as i32, y as i32),
                    out.index(nx as i32, ny as i32),
                );
                out.pixels[dst..dst + 4].copy_from_slice(&self.pixels[src..src + 4]);
            }
        }
        out
    }

    pub fn resized(&self, width: u32, height: u32) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        if (width, height) == (self.width, self.height) {
            return self.clone();
        }
        let source = image::RgbaImage::from_raw(self.width, self.height, self.pixels.clone())
            .expect("pixel buffer matches dimensions");
        let resized = image::imageops::resize(
            &source,
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        );
        Self {
            width,
            height,
            pixels: resized.into_raw(),
        }
    }

    pub fn blend(&mut self, other: &Image, ox: i32, oy: i32) {
        let target = Rect::new(ox, oy, other.width, other.height).intersect(&self.rect());
        for y in target.y..target.bottom() {
            for x in target.x..target.right() {
                let s = other.index(x - ox, y - oy);
                let color = other.pixels[s..s + 4].try_into().unwrap();
                let d = self.index(x, y);
                blend_pixel(&mut self.pixels[d..d + 4], color);
            }
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Rgba) {
        let r = rect.intersect(&self.rect());
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                let i = self.index(x, y);
                blend_pixel(&mut self.pixels[i..i + 4], color);
            }
        }
    }

    pub fn fill_rounded(&mut self, rect: Rect, radius: u32, color: Rgba) {
        let r = rect.intersect(&self.rect());
        let radius = radius.min(rect.w / 2).min(rect.h / 2) as f32;
        let (left, top) = (rect.x as f32 + radius, rect.y as f32 + radius);
        let (right, bottom) = (rect.right() as f32 - radius, rect.bottom() as f32 - radius);
        for y in r.y..r.bottom() {
            let cy = y as f32 + 0.5;
            let dy = (top - cy).max(cy - bottom).max(0.0);
            for x in r.x..r.right() {
                let cx = x as f32 + 0.5;
                let dx = (left - cx).max(cx - right).max(0.0);
                let coverage = (radius + 0.5 - dx.hypot(dy)).clamp(0.0, 1.0);
                if coverage == 0.0 {
                    continue;
                }
                let mut c = color;
                c[3] = (color[3] as f32 * coverage).round() as u8;
                let i = self.index(x, y);
                blend_pixel(&mut self.pixels[i..i + 4], c);
            }
        }
    }

    pub fn stroke_rect(&mut self, rect: Rect, color: Rgba, width: u32) {
        let w = width.min(rect.w).min(rect.h);
        if w == 0 {
            return;
        }
        self.fill_rect(Rect::new(rect.x, rect.y, rect.w, w), color);
        self.fill_rect(
            Rect::new(rect.x, rect.bottom() - w as i32, rect.w, w),
            color,
        );
        self.fill_rect(Rect::new(rect.x, rect.y, w, rect.h), color);
        self.fill_rect(Rect::new(rect.right() - w as i32, rect.y, w, rect.h), color);
    }

    pub fn crosshair(&mut self, x: i32, y: i32, color: Rgba) {
        self.fill_rect(Rect::new(0, y, self.width, 1), color);
        self.fill_rect(Rect::new(x, 0, 1, self.height), color);
    }

    pub fn darken(&mut self, amount: f32) {
        let k = ((1.0 - amount.clamp(0.0, 1.0)) * 255.0) as u32;
        for px in self.pixels.as_chunks_mut::<4>().0 {
            for c in &mut px[..3] {
                *c = (*c as u32 * k / 255) as u8;
            }
        }
    }

    pub fn blit_magnified(&mut self, src: &Image, from: Rect, to: Rect) {
        if from.is_empty() || to.is_empty() {
            return;
        }
        let visible = to.intersect(&self.rect());
        for ty in visible.y..visible.bottom() {
            let sy = from.y + ((ty - to.y) as i64 * from.h as i64 / to.h as i64) as i32;
            for tx in visible.x..visible.right() {
                let sx = from.x + ((tx - to.x) as i64 * from.w as i64 / to.w as i64) as i32;
                if src.rect().contains(sx, sy) {
                    let (s, d) = (src.index(sx, sy), self.index(tx, ty));
                    self.pixels[d..d + 4].copy_from_slice(&src.pixels[s..s + 4]);
                }
            }
        }
    }

    pub fn pixelate(&mut self, rect: Rect, block: u32) {
        let r = rect.intersect(&self.rect());
        let block = block.max(2);
        for by in (r.y..r.bottom()).step_by(block as usize) {
            for bx in (r.x..r.right()).step_by(block as usize) {
                let cell = Rect::new(bx, by, block, block).intersect(&r);
                let mut sum = [0u64; 3];
                for y in cell.y..cell.bottom() {
                    for x in cell.x..cell.right() {
                        let i = self.index(x, y);
                        for (total, &value) in sum.iter_mut().zip(&self.pixels[i..i + 3]) {
                            *total += value as u64;
                        }
                    }
                }
                let n = cell.area().max(1);
                let average = [
                    (sum[0] / n) as u8,
                    (sum[1] / n) as u8,
                    (sum[2] / n) as u8,
                    255,
                ];
                self.fill_rect(cell, average);
            }
        }
    }
}

fn blend_pixel(dst: &mut [u8], color: Rgba) {
    let a = color[3] as u32;
    match a {
        0 => {}
        255 => dst.copy_from_slice(&color),
        _ => {
            for c in 0..3 {
                dst[c] = ((color[c] as u32 * a + dst[c] as u32 * (255 - a)) / 255) as u8;
            }
            dst[3] = (a + dst[3] as u32 * (255 - a) / 255) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_drops_row_padding() {
        let stride = 2 * 4 + 4;
        let mut data = vec![0u8; stride * 2];
        for y in 0..2 {
            for x in 0..2 {
                data[y * stride + x * 4..][..3].copy_from_slice(&[10, 20, 30]);
            }
            data[y * stride + 8] = 0xFF;
        }
        let img = Image::from_raw(2, 2, stride, SourceFormat::Bgrx, &data).unwrap();
        assert_eq!(img.pixels.len(), 16);
        assert_eq!(img.pixel(1, 1), [30, 20, 10, 255]);
    }

    #[test]
    fn crop_clamps_to_bounds() {
        let img = Image::new(10, 10);
        let c = img.crop(Rect::new(8, 8, 100, 100)).unwrap();
        assert_eq!((c.width, c.height), (2, 2));
        assert!(img.crop(Rect::new(50, 50, 4, 4)).is_err());
    }

    #[test]
    fn rotate90_swaps_axes() {
        let mut img = Image::new(3, 2);
        img.pixels[0..4].copy_from_slice(&[1, 2, 3, 255]);
        let r = img.transformed(Transform::Rotate90);
        assert_eq!((r.width, r.height), (2, 3));
        assert_eq!(r.pixel(1, 0), [1, 2, 3, 255]);
    }

    #[test]
    fn flip_reverses_rows() {
        let mut img = Image::new(1, 2);
        img.pixels[0..4].copy_from_slice(&[9, 9, 9, 255]);
        assert_eq!(img.flipped_vertically().pixel(0, 1), [9, 9, 9, 255]);
    }

    #[test]
    fn pixelate_flattens_detail_into_block_averages() {
        let mut img = Image::new(8, 8);
        img.fill_rect(Rect::new(0, 0, 4, 8), [255, 255, 255, 255]);
        img.pixelate(Rect::new(0, 0, 8, 8), 8);
        assert_eq!(img.pixel(0, 0), img.pixel(7, 7));
        assert!((img.pixel(0, 0)[0] as i32 - 127).abs() <= 2);
    }
}
