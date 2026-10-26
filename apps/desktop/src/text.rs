use std::collections::HashMap;

use fontdue::{Font, FontSettings, Metrics};

use crate::geom::Rect;
use crate::raster::{Image, Rgba};

const FONTS: &[&str] = &[
    "/usr/share/fonts/TTF/Pretendard-Regular.ttf",
    "/usr/share/fonts/noto/NotoSans-Regular.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "C:\\Windows\\Fonts\\segoeui.ttf",
];

#[derive(Default)]
pub struct TextRenderer {
    font: Option<Font>,
    glyphs: HashMap<(char, u32), (Metrics, Vec<u8>)>,
}

impl TextRenderer {
    pub fn new() -> Self {
        let font = FONTS.iter().find_map(|path| {
            let bytes = std::fs::read(path).ok()?;
            Font::from_bytes(bytes, FontSettings::default()).ok()
        });
        Self {
            font,
            glyphs: HashMap::new(),
        }
    }

    fn glyph(&mut self, ch: char, size: u32) -> Option<&(Metrics, Vec<u8>)> {
        let font = self.font.as_ref()?;
        Some(
            self.glyphs
                .entry((ch, size))
                .or_insert_with(|| font.rasterize(ch, size as f32)),
        )
    }

    pub fn measure(&self, s: &str, size: u32) -> (u32, u32) {
        let Some(font) = &self.font else {
            return (0, size);
        };
        let width: f32 = s
            .chars()
            .map(|ch| font.metrics(ch, size as f32).advance_width.round())
            .sum();
        (width as u32, size)
    }

    pub fn draw(&mut self, img: &mut Image, s: &str, x: i32, y: i32, size: u32, color: Rgba) {
        let baseline = y + size as i32;
        let mut pen = x;
        for ch in s.chars() {
            let Some((m, bitmap)) = self.glyph(ch, size) else {
                continue;
            };
            let left = pen + m.xmin;
            let top = baseline - m.height as i32 - m.ymin;
            for row in 0..m.height {
                for col in 0..m.width {
                    let coverage = bitmap[row * m.width + col] as u32;
                    if coverage == 0 {
                        continue;
                    }
                    let mut c = color;
                    c[3] = (coverage * color[3] as u32 / 255) as u8;
                    img.fill_rect(Rect::new(left + col as i32, top + row as i32, 1, 1), c);
                }
            }
            pen += m.advance_width.round() as i32;
        }
    }

    pub fn pill_size(&self, s: &str, size: u32) -> (u32, u32) {
        let (tw, th) = self.measure(s, size);
        (tw + size * 3 / 2, th + size)
    }

    pub fn draw_pill(
        &mut self,
        img: &mut Image,
        s: &str,
        at: (i32, i32),
        size: u32,
        fg: Rgba,
        bg: Rgba,
    ) {
        let (w, h) = self.pill_size(s, size);
        let x = at.0.clamp(0, (img.width as i32 - w as i32).max(0));
        let y = at.1.clamp(0, (img.height as i32 - h as i32).max(0));
        img.fill_rounded(Rect::new(x, y, w, h), h / 2, bg);
        let (tw, _) = self.measure(s, size);
        let ty = text_top(y + h as i32 / 2, size);
        self.draw(img, s, x + (w as i32 - tw as i32) / 2, ty, size, fg);
    }
}

pub fn text_top(centre_y: i32, size: u32) -> i32 {
    centre_y - (size as f32 * 0.64).round() as i32
}
