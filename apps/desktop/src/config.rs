use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::app;

pub const TEMPLATE: &str = r#"# clip configuration. Every key is optional; delete a line to use its default.

[storage]
# Where screenshots are saved. CLIP_SHOT_DIR overrides this.
# screenshots = "~/Pictures/Screenshots"

# File name for new screenshots. Supports %Y %m %d %H %M %S (UTC).
# filename = "clip_%Y-%m-%d_%H-%M-%S.png"

# Save screenshots to disk as well as copying them.
# save = true

[ocr]
# Directory holding det.onnx and rec/<script>.onnx + rec/<script>.txt.
# models = "~/.local/share/clip/ocr"

# Recognition models to try, in order. Each covers a family of languages:
# korean chinese latin cyrillic eslav arabic devanagari greek thai tamil telugu english
# scripts = ["korean", "chinese", "latin"]
"#;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub storage: Storage,
    pub ocr: Ocr,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Storage {
    pub screenshots: PathBuf,
    pub filename: String,
    pub save: bool,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            screenshots: app::home().join("Pictures/Screenshots"),
            filename: "clip_%Y-%m-%d_%H-%M-%S.png".into(),
            save: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Ocr {
    pub models: PathBuf,
    pub scripts: Vec<String>,
}

impl Default for Ocr {
    fn default() -> Self {
        Self {
            models: app::data_dir().join("ocr"),
            scripts: clip_ocr::DEFAULT_SCRIPTS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        app::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text).with_context(|| format!("parse {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
        }
    }

    pub fn parse(text: &str) -> Result<Self> {
        let mut config: Config = toml::from_str(text)?;
        config.storage.screenshots = expand_home(&config.storage.screenshots);
        config.ocr.models = expand_home(&config.ocr.models);
        Ok(config)
    }

    pub fn screenshot_dir(&self) -> PathBuf {
        std::env::var_os("CLIP_SHOT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.storage.screenshots.clone())
    }
}

fn expand_home(path: &Path) -> PathBuf {
    match path.strip_prefix("~") {
        Ok(rest) => app::home().join(rest),
        Err(_) => path.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_and_template_are_the_default() {
        assert_eq!(Config::parse("").unwrap(), Config::default());
        assert_eq!(Config::parse(TEMPLATE).unwrap(), Config::default());
    }

    #[test]
    fn reads_every_section() {
        let config = Config::parse(
            r#"
            [storage]
            screenshots = "/data/shots"
            filename = "%Y.png"
            save = false
            [ocr]
            models = "~/ocr"
            scripts = ["latin"]
            "#,
        )
        .unwrap();
        assert_eq!(config.storage.screenshots, PathBuf::from("/data/shots"));
        assert_eq!(config.storage.filename, "%Y.png");
        assert!(!config.storage.save);
        assert_eq!(config.ocr.models, app::home().join("ocr"));
        assert_eq!(config.ocr.scripts, ["latin"]);
    }

    #[test]
    fn rejects_unknown_keys() {
        assert!(Config::parse("[storage]\nscreenshot = \"/x\"").is_err());
    }
}
