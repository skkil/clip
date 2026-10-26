use std::path::Path;

use anyhow::{Context, Result, bail};
use clip_ocr::{Engine, Line};

use crate::config::Ocr;
use crate::raster::Image;

pub fn recognize(config: &Ocr, image: &Image) -> Result<String> {
    Ok(clip_ocr::to_text(&lines(config, image)?))
}

pub fn lines(config: &Ocr, image: &Image) -> Result<Vec<Line>> {
    load(config)?
        .recognize_rgba(&image.pixels, image.width as usize, image.height as usize)
        .context("recognise text")
}

fn load(config: &Ocr) -> Result<Engine> {
    let dir = &config.models;
    let det = dir.join("det.onnx");
    if !det.is_file() {
        bail!(
            "no OCR models in {}; fetch them with `apps/shared/ocr/scripts/fetch-models.py --out {}`",
            dir.display(),
            dir.display()
        );
    }

    let mut engine = Engine::new(&read(&det)?).context("load detection model")?;
    for script in &config.scripts {
        if !clip_ocr::SCRIPTS.contains(&script.as_str()) {
            bail!(
                "unknown OCR script {script:?}; expected one of {:?}",
                clip_ocr::SCRIPTS
            );
        }
        let rec = dir.join("rec").join(format!("{script}.onnx"));
        let dict = dir.join("rec").join(format!("{script}.txt"));
        let dict = String::from_utf8(read(&dict)?).context("dictionary is not UTF-8")?;
        engine
            .add_recognizer(script, &read(&rec)?, &dict)
            .with_context(|| format!("load {script} recognition model"))?;
    }
    Ok(engine)
}

fn read(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("read {}", path.display()))
}
