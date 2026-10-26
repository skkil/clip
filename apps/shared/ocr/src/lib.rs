mod det;
mod image;
mod layout;
mod model;
mod rec;
#[cfg(feature = "wasm")]
mod wasm;

pub use image::Bgr;
pub use layout::to_text;

use det::Detector;
use rec::Recognizer;

pub const SCRIPTS: &[&str] = &[
    "korean",
    "chinese",
    "latin",
    "cyrillic",
    "eslav",
    "arabic",
    "devanagari",
    "greek",
    "thai",
    "tamil",
    "telugu",
    "english",
];

pub const DEFAULT_SCRIPTS: &[&str] = &["korean", "chinese", "latin"];

const CONFIDENT: f32 = 0.97;
const MIN_SCORE: f32 = 0.5;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("model: {0}")]
    Model(String),
    #[error("image: {0}")]
    Image(String),
    #[error("no recognition model loaded")]
    NoRecognizer,
}

impl From<tract_onnx::prelude::TractError> for Error {
    fn from(e: tract_onnx::prelude::TractError) -> Self {
        Error::Model(format!("{e:#}"))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Box2 {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    pub score: f32,
    pub bounds: Box2,
    pub script: String,
}

#[derive(Debug)]
pub struct Engine {
    detector: Detector,
    recognizers: Vec<Recognizer>,
}

impl Engine {
    pub fn new(det_onnx: &[u8]) -> Result<Self> {
        Ok(Self {
            detector: Detector::load(det_onnx)?,
            recognizers: Vec::new(),
        })
    }

    pub fn add_recognizer(&mut self, script: &str, rec_onnx: &[u8], dict: &str) -> Result<()> {
        self.recognizers
            .push(Recognizer::load(script, rec_onnx, dict)?);
        Ok(())
    }

    pub fn scripts(&self) -> impl Iterator<Item = &str> {
        self.recognizers.iter().map(Recognizer::script)
    }

    pub fn recognize_rgba(&self, rgba: &[u8], width: usize, height: usize) -> Result<Vec<Line>> {
        self.recognize(&Bgr::from_rgba(rgba, width, height)?)
    }

    pub fn recognize(&self, image: &Bgr) -> Result<Vec<Line>> {
        if self.recognizers.is_empty() {
            return Err(Error::NoRecognizer);
        }
        let boxes = self.detector.detect(image)?;
        Ok(self
            .read_lines(image, &boxes)?
            .into_iter()
            .flatten()
            .collect())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn read_lines(&self, image: &Bgr, boxes: &[Box2]) -> Result<Vec<Option<Line>>> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let next = AtomicUsize::new(0);
        let work = || {
            let mut read = Vec::new();
            loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(&bounds) = boxes.get(i) else {
                    return read;
                };
                read.push((i, self.read_line(&image.crop(bounds), bounds)));
            }
        };
        let workers = std::thread::available_parallelism().map_or(1, |n| n.get());
        let mut read: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..workers.min(boxes.len()))
                .map(|_| scope.spawn(work))
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().expect("OCR worker panicked"))
                .collect()
        });
        read.sort_by_key(|(i, _)| *i);
        read.into_iter().map(|(_, line)| line).collect()
    }

    #[cfg(target_arch = "wasm32")]
    fn read_lines(&self, image: &Bgr, boxes: &[Box2]) -> Result<Vec<Option<Line>>> {
        boxes
            .iter()
            .map(|&bounds| self.read_line(&image.crop(bounds), bounds))
            .collect()
    }

    fn read_line(&self, crop: &Bgr, bounds: Box2) -> Result<Option<Line>> {
        let mut best: Option<(f32, String, &str)> = None;
        for recognizer in &self.recognizers {
            let (text, score) = recognizer.read(crop)?;
            if text.trim().is_empty() {
                continue;
            }
            if best.as_ref().is_none_or(|(s, _, _)| score > *s) {
                best = Some((score, text, recognizer.script()));
            }
            if score >= CONFIDENT {
                break;
            }
        }
        Ok(best
            .filter(|(score, _, _)| *score >= MIN_SCORE)
            .map(|(score, text, script)| Line {
                text,
                score,
                bounds,
                script: script.to_owned(),
            }))
    }
}
