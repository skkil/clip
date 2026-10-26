use std::path::PathBuf;

use anyhow::{Context, Result};

pub const NAME: &str = "clip";
pub const APP_ID: &str = "org.skkil.clip";
pub const BIN: &str = "clip-desktop";

pub fn runtime_dir() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join(NAME);
    create_private_dir(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
}

pub fn config_dir() -> PathBuf {
    user_dir("XDG_CONFIG_HOME", ".config")
}

pub fn data_dir() -> PathBuf {
    user_dir("XDG_DATA_HOME", ".local/share")
}

fn user_dir(xdg_var: &str, home_fallback: &str) -> PathBuf {
    #[cfg(windows)]
    if let Some(dir) = std::env::var_os("APPDATA") {
        return PathBuf::from(dir).join(NAME);
    }
    if let Some(dir) = std::env::var_os(xdg_var) {
        return PathBuf::from(dir).join(NAME);
    }
    home().join(home_fallback).join(NAME)
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

#[cfg(unix)]
fn create_private_dir(dir: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn create_private_dir(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}
