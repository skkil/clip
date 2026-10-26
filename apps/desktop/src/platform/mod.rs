#[cfg(target_os = "linux")]
pub mod wayland;

use std::path::Path;
use std::sync::mpsc::Sender;

use anyhow::{Result, bail};
use clip_ocr::Line;

use crate::action::Action;
use crate::capture::Capture;
use crate::geom::Rect;
use crate::select::{Format, Selection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectMode {
    #[default]
    Region,
    Screen,
}

impl SelectMode {
    pub fn next(self) -> Self {
        match self {
            SelectMode::Region => SelectMode::Screen,
            SelectMode::Screen => SelectMode::Region,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SelectOptions {
    pub mode: SelectMode,
    pub cursor: bool,
    pub format: Format,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub rect: Rect,
    pub title: String,
    pub app_id: String,
}

pub fn window_at(windows: &[WindowInfo], x: i32, y: i32) -> Option<&WindowInfo> {
    windows
        .iter()
        .filter(|w| w.rect.contains(x, y))
        .min_by_key(|w| w.rect.area())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardItem {
    pub mime: String,
    pub data: Vec<u8>,
}

impl ClipboardItem {
    pub fn new(mime: &str, data: impl Into<Vec<u8>>) -> Self {
        Self {
            mime: mime.to_owned(),
            data: data.into(),
        }
    }
}

pub type ReadText<'a> = dyn Fn(Rect) -> Result<Vec<Line>> + 'a;

pub trait Platform {
    fn capture(&self, cursor: bool) -> Result<Capture>;

    fn select(
        &self,
        capture: &Capture,
        opts: SelectOptions,
        read: &ReadText,
    ) -> Result<Option<Selection>>;

    fn copy(&self, items: &[ClipboardItem]) -> Result<()>;

    fn notify(&self, title: &str, body: &str, icon: Option<&Path>);

    fn shortcuts(&self, actions: &[Action], sink: Sender<Action>) -> Result<()>;
}

pub fn current() -> Result<Box<dyn Platform>> {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return Ok(Box::new(wayland::Wayland));
    }
    bail!("unsupported session: only Wayland is implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(app_id: &str, rect: Rect) -> WindowInfo {
        WindowInfo {
            rect,
            title: String::new(),
            app_id: app_id.into(),
        }
    }

    #[test]
    fn the_smallest_window_under_the_point_wins() {
        let windows = [
            win("dialog", Rect::new(100, 100, 200, 200)),
            win("kitty", Rect::new(0, 0, 2560, 1440)),
        ];
        assert_eq!(window_at(&windows, 150, 150).unwrap().app_id, "dialog");
        assert_eq!(window_at(&windows, 900, 900).unwrap().app_id, "kitty");
        assert!(window_at(&windows, 5000, 5000).is_none());
    }
}
