mod capture;
pub mod clipboard;
mod overlay;
mod shortcuts;
mod windows;

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use anyhow::Result;

use super::{ClipboardItem, Platform, ReadText, SelectOptions};
use crate::action::Action;
use crate::app;
use crate::capture::Capture;
use crate::select::Selection;

pub struct Wayland;

impl Platform for Wayland {
    fn capture(&self, cursor: bool) -> Result<Capture> {
        capture::capture_all(cursor)
    }

    fn select(
        &self,
        capture: &Capture,
        opts: SelectOptions,
        read: &ReadText,
    ) -> Result<Option<Selection>> {
        overlay::select(capture, opts, read)
    }

    fn copy(&self, items: &[ClipboardItem]) -> Result<()> {
        clipboard::copy(items)
    }

    fn notify(&self, title: &str, body: &str, icon: Option<&Path>) {
        let mut cmd = Command::new("notify-send");
        cmd.args(["--app-name", app::NAME]);
        if let Some(icon) = icon {
            cmd.arg("--icon").arg(icon);
        }
        cmd.args([title, body])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok();
    }

    fn shortcuts(&self, actions: &[Action], sink: Sender<Action>) -> Result<()> {
        shortcuts::run(actions, sink)
    }
}
