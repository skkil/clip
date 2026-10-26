use tract_onnx::prelude::Tensor;

use crate::model::{Model, as_slice};
use crate::{Bgr, Box2, Result};

const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];
const MIN_SIDE: usize = 64;
const MAX_SIDE: usize = 4000;
const THRESH: f32 = 0.3;
const BOX_THRESH: f32 = 0.6;
const UNCLIP_RATIO: f32 = 1.5;
const MIN_BOX: usize = 3;
const STRIDE: usize = 32;

#[derive(Debug)]
pub(crate) struct Detector {
    model: Model,
}

impl Detector {
    pub(crate) fn load(onnx_bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            model: Model::load(onnx_bytes)?,
        })
    }

    pub(crate) fn detect(&self, image: &Bgr) -> Result<Vec<Box2>> {
        let (width, height) = input_size(image.width, image.height);
        let pixels = image.resized(width, height);
        let plane = width * height;
        let mut input = vec![0f32; 3 * plane];
        for (i, px) in pixels.iter().enumerate() {
            for c in 0..3 {
                input[c * plane + i] = (px[c] / 255.0 - MEAN[c]) / STD[c];
            }
        }
        let output = self.run([1, 3, height, width], &input)?;
        let prob = as_slice(&output)?;

        let sx = image.width as f32 / width as f32;
        let sy = image.height as f32 / height as f32;
        let mut boxes: Vec<Box2> = components(prob, width, height)
            .into_iter()
            .map(|b| Box2 {
                x: b.x * sx,
                y: b.y * sy,
                w: b.w * sx,
                h: b.h * sy,
            })
            .collect();
        boxes.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
        Ok(boxes)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn run(&self, shape: [usize; 4], input: &[f32]) -> Result<Tensor> {
        use tract_linalg::multithread::{Executor, multithread_tract_scope};

        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        multithread_tract_scope(Executor::multithread(threads), || {
            self.model.run(shape, input)
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn run(&self, shape: [usize; 4], input: &[f32]) -> Result<Tensor> {
        self.model.run(shape, input)
    }
}

fn input_size(width: usize, height: usize) -> (usize, usize) {
    let mut ratio = 1.0f32;
    let short = width.min(height) as f32;
    let long = width.max(height) as f32;
    if short < MIN_SIDE as f32 {
        ratio = MIN_SIDE as f32 / short;
    }
    if long * ratio > MAX_SIDE as f32 {
        ratio = MAX_SIDE as f32 / long;
    }
    let round =
        |v: usize| ((((v as f32 * ratio) / STRIDE as f32).round() as usize) * STRIDE).max(STRIDE);
    (round(width), round(height))
}

fn components(prob: &[f32], width: usize, height: usize) -> Vec<Box2> {
    let mut seen = vec![false; prob.len()];
    let mut stack = Vec::new();
    let mut boxes = Vec::new();
    for start in 0..prob.len() {
        if seen[start] || prob[start] <= THRESH {
            continue;
        }
        seen[start] = true;
        stack.push(start);
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % width, i / width);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
            let neighbours = [
                (x > 0).then(|| i - 1),
                (x + 1 < width).then(|| i + 1),
                (y > 0).then(|| i - width),
                (y + 1 < height).then(|| i + width),
            ];
            for n in neighbours.into_iter().flatten() {
                if !seen[n] && prob[n] > THRESH {
                    seen[n] = true;
                    stack.push(n);
                }
            }
        }
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        if w.min(h) < MIN_BOX || mean(prob, width, x0, y0, x1, y1) < BOX_THRESH {
            continue;
        }
        let (w, h) = (w as f32, h as f32);
        let d = w * h * UNCLIP_RATIO / (2.0 * (w + h));
        let x = (x0 as f32 - d).max(0.0);
        let y = (y0 as f32 - d).max(0.0);
        boxes.push(Box2 {
            x,
            y,
            w: (x1 as f32 + 1.0 + d).min(width as f32) - x,
            h: (y1 as f32 + 1.0 + d).min(height as f32) - y,
        });
    }
    boxes
}

fn mean(prob: &[f32], width: usize, x0: usize, y0: usize, x1: usize, y1: usize) -> f32 {
    let mut sum = 0.0;
    for y in y0..=y1 {
        sum += prob[y * width + x0..=y * width + x1].iter().sum::<f32>();
    }
    sum / ((x1 - x0 + 1) * (y1 - y0 + 1)) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_is_a_multiple_of_32() {
        assert_eq!(input_size(1000, 30), (2144, 64));
        assert_eq!(input_size(2560, 1440), (2560, 1440));
        assert_eq!(input_size(8000, 100), (4000, 64));
    }

    #[test]
    fn finds_one_component_per_blob() {
        let (w, h) = (20, 10);
        let mut prob = vec![0.0; w * h];
        for y in 2..6 {
            for x in 3..15 {
                prob[y * w + x] = 0.9;
            }
        }
        let boxes = components(&prob, w, h);
        assert_eq!(boxes.len(), 1);
        assert!(boxes[0].x < 3.0 && boxes[0].x + boxes[0].w > 15.0);
    }
}
