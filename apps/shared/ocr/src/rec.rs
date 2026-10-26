use crate::model::{Model, as_slice};
use crate::{Bgr, Error, Result};

const HEIGHT: usize = 48;
const MIN_WIDTH: usize = 320;
const MAX_WIDTH: usize = 3200;
const WIDTH_STEP: usize = 160;

#[derive(Debug)]
pub(crate) struct Recognizer {
    script: String,
    model: Model,
    charset: Vec<String>,
}

impl Recognizer {
    pub(crate) fn load(script: &str, onnx_bytes: &[u8], dict: &str) -> Result<Self> {
        let mut charset: Vec<String> = std::iter::once(String::new())
            .chain(dict.lines().map(str::to_owned))
            .collect();
        charset.push(" ".to_owned());
        Ok(Self {
            script: script.to_owned(),
            model: Model::load(onnx_bytes)?,
            charset,
        })
    }

    pub(crate) fn script(&self) -> &str {
        &self.script
    }

    pub(crate) fn read(&self, crop: &Bgr) -> Result<(String, f32)> {
        let resized = ((HEIGHT as f32 * crop.width as f32 / crop.height as f32).ceil() as usize)
            .clamp(1, MAX_WIDTH);
        let width = resized.max(MIN_WIDTH).next_multiple_of(WIDTH_STEP);
        let pixels = crop.resized(resized, HEIGHT);
        let plane = width * HEIGHT;
        let mut input = vec![0f32; 3 * plane];
        for y in 0..HEIGHT {
            for x in 0..resized {
                let px = pixels[y * resized + x];
                for c in 0..3 {
                    input[c * plane + y * width + x] = px[c] / 127.5 - 1.0;
                }
            }
        }
        let output = self.model.run([1, 3, HEIGHT, width], &input)?;
        let classes = *output
            .shape()
            .last()
            .ok_or_else(|| Error::Model("empty output".into()))?;
        if classes != self.charset.len() {
            return Err(Error::Model(format!(
                "{}: model has {classes} classes, dictionary has {}",
                self.script,
                self.charset.len()
            )));
        }
        let (text, score) = ctc_decode(as_slice(&output)?, classes, &self.charset);
        Ok((
            if self.script == "arabic" {
                visual_to_logical(&text)
            } else {
                text
            },
            score,
        ))
    }
}

fn visual_to_logical(text: &str) -> String {
    let mut runs: Vec<String> = Vec::new();
    let mut ltr = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || " :*./%+-".contains(c) {
            ltr.push(c);
            continue;
        }
        if !ltr.is_empty() {
            runs.push(std::mem::take(&mut ltr));
        }
        runs.push(c.to_string());
    }
    if !ltr.is_empty() {
        runs.push(ltr);
    }
    runs.into_iter().rev().collect()
}

fn ctc_decode(probs: &[f32], classes: usize, charset: &[String]) -> (String, f32) {
    let mut text = String::new();
    let mut scores = Vec::new();
    let mut previous = 0;
    for step in probs.chunks_exact(classes) {
        let (index, score) = step
            .iter()
            .copied()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((0, 0.0));
        if index != 0 && index != previous {
            text.push_str(&charset[index]);
            scores.push(score);
        }
        previous = index;
    }
    let score = if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f32>() / scores.len() as f32
    };
    (text, score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtl_reverses_characters_but_keeps_ltr_runs() {
        assert_eq!(visual_to_logical("cba 12"), "cba 12");
        assert_eq!(visual_to_logical("ملاعلا"), "العالم");
        assert_eq!(visual_to_logical("2026 ابحرم"), "مرحبا2026 ");
    }

    #[test]
    fn ctc_collapses_repeats_and_blanks() {
        let charset: Vec<String> = ["", "a", "b"].map(String::from).to_vec();
        let probs = [
            0.1, 0.8, 0.1, //
            0.1, 0.8, 0.1, //
            0.9, 0.05, 0.05, //
            0.1, 0.8, 0.1, //
            0.1, 0.1, 0.8,
        ];
        let (text, score) = ctc_decode(&probs, 3, &charset);
        assert_eq!(text, "aab");
        assert!((score - 0.8).abs() < 1e-6);
    }
}
