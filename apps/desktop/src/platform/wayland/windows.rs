use std::collections::HashMap;
use std::process::Command;

use serde_json::Value;

use crate::geom::Rect;
use crate::platform::WindowInfo;

pub fn list() -> Vec<WindowInfo> {
    hyprland_windows().unwrap_or_default()
}

fn hyprctl(args: &[&str]) -> Option<Value> {
    let out = Command::new("hyprctl").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

fn visible_workspaces() -> Option<HashMap<i64, Vec<i64>>> {
    let mut shown: HashMap<i64, Vec<i64>> = HashMap::new();
    for monitor in hyprctl(&["monitors", "-j"])?.as_array()? {
        let ids = shown.entry(monitor.get("id")?.as_i64()?).or_default();
        ids.extend(
            monitor
                .pointer("/activeWorkspace/id")
                .and_then(Value::as_i64),
        );
        ids.extend(
            monitor
                .pointer("/specialWorkspace/id")
                .and_then(Value::as_i64)
                .filter(|&id| id != 0),
        );
    }
    Some(shown)
}

fn hyprland_windows() -> Option<Vec<WindowInfo>> {
    let clients = hyprctl(&["clients", "-j"])?;
    let shown = visible_workspaces()?;
    let text = |c: &Value, key: &str| {
        c.get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };

    let windows = clients
        .as_array()?
        .iter()
        .filter_map(|c| {
            if !c.get("mapped")?.as_bool()? || c.get("hidden")?.as_bool()? {
                return None;
            }
            let monitor = c.get("monitor")?.as_i64()?;
            let workspace = c.pointer("/workspace/id")?.as_i64()?;
            if !shown.get(&monitor)?.contains(&workspace) {
                return None;
            }
            let at = c.get("at")?.as_array()?;
            let size = c.get("size")?.as_array()?;
            let (w, h) = (size.first()?.as_i64()?, size.get(1)?.as_i64()?);
            if w <= 0 || h <= 0 {
                return None;
            }
            Some(WindowInfo {
                rect: Rect::new(
                    at.first()?.as_i64()? as i32,
                    at.get(1)?.as_i64()? as i32,
                    w as u32,
                    h as u32,
                ),
                title: text(c, "title"),
                app_id: text(c, "class"),
            })
        })
        .collect();
    Some(windows)
}
