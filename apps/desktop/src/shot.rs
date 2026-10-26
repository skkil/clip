use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, bail};
use clip_ocr::Line;

use crate::capture::Capture;
use crate::cli::{ShotOptions, ShotTarget};
use crate::config::Config;
use crate::geom::Rect;
use crate::platform::{self, ClipboardItem, Platform, SelectMode, SelectOptions};
use crate::raster::Image;
use crate::select::{Format, Selection};
use crate::{app, lock, naming, ocr};

const PREVIEW_CHARS: usize = 120;

enum Outcome {
    Image {
        width: u32,
        height: u32,
        copied: bool,
        saved: Option<PathBuf>,
    },
    Text {
        text: String,
        copied: bool,
    },
}

pub fn run(target: ShotTarget, options: ShotOptions) -> Result<()> {
    let Some(_lock) = lock::try_lock_screen()? else {
        return Ok(());
    };
    let platform = platform::current()?;
    match take(platform.as_ref(), target, &options) {
        Ok(Some(outcome)) => {
            report(platform.as_ref(), &outcome);
            Ok(())
        }
        Ok(None) => {
            eprintln!("cancelled");
            std::process::exit(1);
        }
        Err(e) => {
            platform.notify("Screenshot failed", &format!("{e:#}"), None);
            Err(e)
        }
    }
}

fn take(
    platform: &dyn Platform,
    target: ShotTarget,
    options: &ShotOptions,
) -> Result<Option<Outcome>> {
    let config = Config::load()?;
    let save = !options.no_save && (config.storage.save || options.output.is_some());
    if options.no_copy && !save {
        bail!("nothing to do: the capture would be neither copied nor saved");
    }
    if options.delay > 0.0 {
        std::thread::sleep(Duration::from_secs_f64(options.delay));
    }

    let interactive = !matches!(target, ShotTarget::Full | ShotTarget::Output { .. });
    let capture = platform.capture(options.cursor || interactive)?;

    let image = match target {
        ShotTarget::Full => capture.composite(options.cursor)?,
        ShotTarget::Output { name } => output(&capture, &name, options.cursor)?,
        target => {
            let opts = SelectOptions {
                mode: if target == ShotTarget::Screen {
                    SelectMode::Screen
                } else {
                    SelectMode::Region
                },
                cursor: options.cursor,
                format: if target == ShotTarget::Text {
                    Format::Text
                } else {
                    Format::Image
                },
            };
            let read = |region| read_text(&capture, &config, region);
            match platform.select(&capture, opts, &read)? {
                None => return Ok(None),
                Some(Selection::Text(text)) => return copy_text(platform, text, options).map(Some),
                Some(Selection::Image(shot)) => capture.render(&shot)?,
            }
        }
    };
    store(platform, &config, &image, options, save).map(Some)
}

fn output(capture: &Capture, name: &str, cursor: bool) -> Result<Image> {
    let screen = capture
        .screens
        .iter()
        .find(|s| s.name == name)
        .with_context(|| {
            let names: Vec<_> = capture.screens.iter().map(|s| s.name.as_str()).collect();
            format!("no display named {name:?}; have {names:?}")
        })?;
    Ok(screen.image_with(cursor).into_owned())
}

fn read_text(capture: &Capture, config: &Config, region: Rect) -> Result<Vec<Line>> {
    let (image, scale) = capture.crop(region, false)?;
    let mut lines = ocr::lines(&config.ocr, &image)?;
    let scale = scale as f32;
    for line in &mut lines {
        let b = &mut line.bounds;
        b.x = region.x as f32 + b.x / scale;
        b.y = region.y as f32 + b.y / scale;
        b.w /= scale;
        b.h /= scale;
    }
    Ok(lines)
}

fn copy_text(platform: &dyn Platform, text: String, options: &ShotOptions) -> Result<Outcome> {
    let text = text.trim().to_owned();
    if text.is_empty() {
        bail!("no text found");
    }
    if !options.no_copy {
        platform.copy(&[ClipboardItem::new(
            "text/plain;charset=utf-8",
            text.as_str(),
        )])?;
    }
    println!("{text}");
    Ok(Outcome::Text {
        text,
        copied: !options.no_copy,
    })
}

fn store(
    platform: &dyn Platform,
    config: &Config,
    image: &Image,
    options: &ShotOptions,
    save: bool,
) -> Result<Outcome> {
    let png = image.to_png()?;
    if !options.no_copy {
        platform.copy(&[ClipboardItem::new("image/png", png.clone())])?;
    }
    let saved = if save {
        let path = destination(config, options);
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        std::fs::write(&path, &png).with_context(|| format!("write {}", path.display()))?;
        println!("{}", path.display());
        Some(path)
    } else {
        None
    };
    Ok(Outcome::Image {
        width: image.width,
        height: image.height,
        copied: !options.no_copy,
        saved,
    })
}

fn report(platform: &dyn Platform, outcome: &Outcome) {
    match outcome {
        Outcome::Image {
            width,
            height,
            copied,
            saved,
        } => {
            let title = if *copied {
                "Screenshot copied"
            } else {
                "Screenshot saved"
            };
            let mut body = format!("{width} × {height}");
            if let Some(path) = saved {
                body += &format!(" · saved to {}", tilde(path));
            }
            platform.notify(title, &body, saved.as_deref());
        }
        Outcome::Text { text, copied } => {
            let title = if *copied { "Text copied" } else { "Text read" };
            platform.notify(title, &preview(text), None);
        }
    }
}

fn preview(text: &str) -> String {
    let mut short: String = text.chars().take(PREVIEW_CHARS).collect();
    if short.len() < text.len() {
        short.push('…');
    }
    short
}

fn tilde(path: &Path) -> String {
    match path.strip_prefix(app::home()) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

fn destination(config: &Config, options: &ShotOptions) -> PathBuf {
    let now = SystemTime::now();
    match &options.output {
        Some(path) => PathBuf::from(naming::expand(&path.to_string_lossy(), now)),
        None => config
            .screenshot_dir()
            .join(naming::expand(&config.storage.filename, now)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_text_preview_is_cut_with_an_ellipsis() {
        assert_eq!(preview("short"), "short");
        let long = "가".repeat(PREVIEW_CHARS + 5);
        assert_eq!(preview(&long).chars().count(), PREVIEW_CHARS + 1);
        assert!(preview(&long).ends_with('…'));
    }
}
