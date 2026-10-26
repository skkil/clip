use std::borrow::Cow;
use std::sync::Arc;

use anyhow::{Result, bail};

use crate::geom::Rect;
use crate::raster::{Image, Rgba};
use crate::select::{EditTool, Shot};

const REDACT: Rgba = [0, 0, 0, 255];
pub const MOSAIC_BLOCK: u32 = 12;

#[derive(Clone)]
pub struct Screen {
    pub name: String,
    pub logical: Rect,
    pub scale: f64,
    pub image: Arc<Image>,
    pub cursor: Option<CursorLayer>,
}

impl Screen {
    pub fn image_with(&self, cursor: bool) -> Cow<'_, Image> {
        match &self.cursor {
            Some(layer) if cursor => {
                let mut image = (*self.image).clone();
                layer.paint(&mut image, (0, 0));
                Cow::Owned(image)
            }
            _ => Cow::Borrowed(&self.image),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CursorLayer {
    pub area: Rect,
    pub image: Image,
}

impl CursorLayer {
    pub fn between(plain: &Image, with_cursor: &Image) -> Option<Self> {
        if (plain.width, plain.height) != (with_cursor.width, with_cursor.height) {
            return None;
        }
        let stride = plain.stride();
        let (mut x0, mut x1, mut y0, mut y1) = (usize::MAX, 0, usize::MAX, 0);
        let rows = plain
            .pixels
            .chunks_exact(stride)
            .zip(with_cursor.pixels.chunks_exact(stride));
        for (y, (a, b)) in rows.enumerate() {
            if a == b {
                continue;
            }
            let differs = |(p, q): (&[u8; 4], &[u8; 4])| p != q;
            let pixels = || a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0);
            x0 = x0.min(pixels().position(differs)?);
            x1 = x1.max(pixels().rposition(differs)? + 1);
            y0 = y0.min(y);
            y1 = y + 1;
        }
        if y1 == 0 {
            return None;
        }
        let area = Rect::new(x0 as i32, y0 as i32, (x1 - x0) as u32, (y1 - y0) as u32);
        Some(Self {
            area,
            image: with_cursor.crop(area).ok()?,
        })
    }

    pub fn paint(&self, target: &mut Image, origin: (i32, i32)) {
        target.blend(&self.image, self.area.x - origin.0, self.area.y - origin.1);
    }
}

pub struct Capture {
    pub screens: Vec<Screen>,
}

impl Capture {
    pub fn new(mut screens: Vec<Screen>) -> Self {
        screens.sort_by_key(|s| (s.logical.x, s.logical.y));
        Self { screens }
    }

    pub fn bounds(&self) -> Rect {
        let mut rects = self.screens.iter().map(|s| s.logical);
        let Some(first) = rects.next() else {
            return Rect::default();
        };
        let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.right(), first.bottom());
        for r in rects {
            x0 = x0.min(r.x);
            y0 = y0.min(r.y);
            x1 = x1.max(r.right());
            y1 = y1.max(r.bottom());
        }
        Rect::new(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32)
    }

    pub fn render_scale(&self) -> f64 {
        self.screens.iter().map(|s| s.scale).fold(1.0, f64::max)
    }

    pub fn composite(&self, cursor: bool) -> Result<Image> {
        if self.screens.is_empty() {
            bail!("no displays captured");
        }
        let bounds = self.bounds();
        let scale = self.render_scale();
        let size = Rect::new(0, 0, bounds.w, bounds.h).scaled(scale);

        if let [only] = self.screens.as_slice()
            && only.logical == bounds
            && (only.image.width, only.image.height) == (size.w, size.h)
        {
            return Ok(only.image_with(cursor).into_owned());
        }

        let mut canvas = Image::new(size.w.max(1), size.h.max(1));
        for screen in &self.screens {
            let want = Rect::new(0, 0, screen.logical.w, screen.logical.h).scaled(scale);
            let image = screen.image_with(cursor).resized(want.w, want.h);
            let origin = screen
                .logical
                .relative_to((bounds.x, bounds.y))
                .scaled(scale);
            canvas.blend(&image, origin.x, origin.y);
        }
        Ok(canvas)
    }

    pub fn crop(&self, region: Rect, cursor: bool) -> Result<(Image, f64)> {
        if let Some(screen) = self.screens.iter().find(|s| s.logical.covers(&region)) {
            let local = region
                .relative_to((screen.logical.x, screen.logical.y))
                .scaled(screen.scale);
            let mut image = screen.image.crop(local)?;
            if cursor && let Some(layer) = &screen.cursor {
                layer.paint(&mut image, (local.x, local.y));
            }
            return Ok((image, screen.scale));
        }
        let bounds = self.bounds();
        let scale = self.render_scale();
        let local = region.relative_to((bounds.x, bounds.y)).scaled(scale);
        Ok((self.composite(cursor)?.crop(local)?, scale))
    }

    pub fn render(&self, shot: &Shot) -> Result<Image> {
        let (mut image, scale) = self.crop(shot.region, shot.cursor)?;
        for edit in &shot.edits {
            let local = edit
                .area
                .intersect(&shot.region)
                .relative_to((shot.region.x, shot.region.y))
                .scaled(scale);
            match edit.tool {
                EditTool::Redact => image.fill_rect(local, REDACT),
                EditTool::Mosaic => {
                    image.pixelate(local, (MOSAIC_BLOCK as f64 * scale).round() as u32)
                }
            }
        }
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::select::Edit;

    const RED: Rgba = [255, 0, 0, 255];
    const BLUE: Rgba = [0, 0, 255, 255];
    const WHITE: Rgba = [255, 255, 255, 255];
    const BLACK: Rgba = [0, 0, 0, 255];

    fn screen(logical: Rect, scale: f64, fill: Rgba) -> Screen {
        let size = Rect::new(0, 0, logical.w, logical.h).scaled(scale);
        let mut image = Image::new(size.w, size.h);
        image.fill_rect(size, fill);
        Screen {
            name: format!("{logical:?}"),
            logical,
            scale,
            image: Arc::new(image),
            cursor: None,
        }
    }

    fn with_cursor(mut s: Screen, x: i32, y: i32) -> Screen {
        let mut lit = (*s.image).clone();
        lit.fill_rect(Rect::new(x, y, 3, 4), WHITE);
        s.cursor = CursorLayer::between(&s.image, &lit);
        s
    }

    fn shot(region: Rect, edits: Vec<Edit>) -> Shot {
        Shot {
            region,
            edits,
            cursor: false,
        }
    }

    #[test]
    fn bounds_union_displays_not_at_the_origin() {
        let cap = Capture::new(vec![
            screen(Rect::new(0, 0, 1920, 1080), 1.0, RED),
            screen(Rect::new(1920, -200, 2560, 1440), 1.0, BLUE),
        ]);
        assert_eq!(cap.bounds(), Rect::new(0, -200, 4480, 1440));
    }

    #[test]
    fn composite_places_each_display_at_its_logical_offset() {
        let cap = Capture::new(vec![
            screen(Rect::new(0, 0, 4, 4), 1.0, RED),
            screen(Rect::new(4, 0, 4, 4), 1.0, BLUE),
        ]);
        let canvas = cap.composite(false).unwrap();
        assert_eq!((canvas.width, canvas.height), (8, 4));
        assert_eq!(canvas.pixel(1, 1), RED);
        assert_eq!(canvas.pixel(5, 1), BLUE);
    }

    #[test]
    fn composite_upscales_a_low_dpi_display_beside_a_hidpi_one() {
        let cap = Capture::new(vec![
            screen(Rect::new(0, 0, 4, 4), 1.0, RED),
            screen(Rect::new(4, 0, 4, 4), 2.0, BLUE),
        ]);
        let canvas = cap.composite(false).unwrap();
        assert_eq!((canvas.width, canvas.height), (16, 8));
        assert_eq!(canvas.pixel(2, 2), RED);
        assert_eq!(canvas.pixel(10, 2), BLUE);
    }

    #[test]
    fn fractional_scale_crops_the_whole_native_frame() {
        let cap = Capture::new(vec![
            screen(Rect::new(0, 0, 2560, 1440), 1.5, BLUE),
            screen(Rect::new(2560, 480, 1920, 1200), 1.0, RED),
        ]);
        let (out, scale) = cap.crop(Rect::new(0, 0, 2560, 1440), false).unwrap();
        assert_eq!((out.width, out.height, scale), (3840, 2160, 1.5));
    }

    #[test]
    fn a_region_straddling_displays_is_cropped_from_the_composite() {
        let cap = Capture::new(vec![
            screen(Rect::new(0, 0, 4, 4), 1.0, RED),
            screen(Rect::new(4, 0, 4, 4), 1.0, BLUE),
        ]);
        let (out, _) = cap.crop(Rect::new(2, 0, 4, 4), false).unwrap();
        assert_eq!(out.pixel(0, 0), RED);
        assert_eq!(out.pixel(3, 0), BLUE);
    }

    #[test]
    fn a_redaction_is_placed_relative_to_the_region() {
        let cap = Capture::new(vec![screen(Rect::new(100, 100, 20, 20), 1.0, RED)]);
        let edit = Edit {
            tool: EditTool::Redact,
            area: Rect::new(110, 110, 4, 4),
        };
        let out = cap
            .render(&shot(Rect::new(110, 110, 10, 10), vec![edit]))
            .unwrap();
        assert_eq!(out.pixel(1, 1), BLACK);
        assert_eq!(out.pixel(8, 8), RED);
    }

    #[test]
    fn a_redaction_is_scaled_with_the_display() {
        let cap = Capture::new(vec![screen(Rect::new(0, 0, 20, 20), 2.0, RED)]);
        let edit = Edit {
            tool: EditTool::Redact,
            area: Rect::new(0, 0, 5, 5),
        };
        let out = cap
            .render(&shot(Rect::new(0, 0, 20, 20), vec![edit]))
            .unwrap();
        assert_eq!(out.pixel(9, 9), BLACK);
        assert_eq!(out.pixel(11, 11), RED);
    }

    #[test]
    fn edits_apply_in_the_order_they_were_drawn() {
        let cap = Capture::new(vec![screen(Rect::new(0, 0, 20, 20), 1.0, RED)]);
        let edits = vec![
            Edit {
                tool: EditTool::Mosaic,
                area: Rect::new(0, 0, 20, 20),
            },
            Edit {
                tool: EditTool::Redact,
                area: Rect::new(5, 5, 10, 10),
            },
        ];
        let out = cap.render(&shot(Rect::new(0, 0, 20, 20), edits)).unwrap();
        assert_eq!(out.pixel(10, 10), BLACK);
    }

    #[test]
    fn the_cursor_layer_is_only_the_pixels_that_changed() {
        let s = with_cursor(screen(Rect::new(0, 0, 20, 20), 1.0, RED), 5, 6);
        let layer = s.cursor.unwrap();
        assert_eq!(layer.area, Rect::new(5, 6, 3, 4));
        assert_eq!(layer.image.pixel(0, 0), WHITE);
    }

    #[test]
    fn the_cursor_is_drawn_only_when_asked_for() {
        let cap = Capture::new(vec![with_cursor(
            screen(Rect::new(0, 0, 20, 20), 1.0, RED),
            5,
            6,
        )]);
        let hidden = shot(Rect::new(4, 4, 8, 8), Vec::new());
        let shown = Shot {
            cursor: true,
            ..hidden.clone()
        };
        assert_eq!(cap.render(&hidden).unwrap().pixel(1, 2), RED);
        assert_eq!(cap.render(&shown).unwrap().pixel(1, 2), WHITE);
        assert_eq!(cap.screens[0].image.pixel(5, 6), RED);
    }
}
