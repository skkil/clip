use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

use crate::{Engine, Line, to_text};

#[wasm_bindgen]
#[derive(Debug)]
pub struct OcrEngine(Engine);

#[wasm_bindgen]
impl OcrEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(det_onnx: &[u8]) -> Result<OcrEngine, JsError> {
        Ok(Self(Engine::new(det_onnx)?))
    }

    #[wasm_bindgen(js_name = addRecognizer)]
    pub fn add_recognizer(
        &mut self,
        script: &str,
        rec_onnx: &[u8],
        dict: &str,
    ) -> Result<(), JsError> {
        Ok(self.0.add_recognizer(script, rec_onnx, dict)?)
    }

    pub fn recognize(&self, rgba: &[u8], width: u32, height: u32) -> Result<Object, JsError> {
        let lines = self
            .0
            .recognize_rgba(rgba, width as usize, height as usize)?;
        let result = Object::new();
        set(&result, "text", &to_text(&lines).into())?;
        set(
            &result,
            "lines",
            &lines.iter().map(line).collect::<Result<Array, _>>()?.into(),
        )?;
        Ok(result)
    }
}

fn line(line: &Line) -> Result<JsValue, JsError> {
    let object = Object::new();
    set(&object, "text", &line.text.as_str().into())?;
    set(&object, "score", &line.score.into())?;
    set(&object, "script", &line.script.as_str().into())?;
    set(&object, "x", &line.bounds.x.into())?;
    set(&object, "y", &line.bounds.y.into())?;
    set(&object, "width", &line.bounds.w.into())?;
    set(&object, "height", &line.bounds.h.into())?;
    Ok(object.into())
}

fn set(object: &Object, key: &str, value: &JsValue) -> Result<(), JsError> {
    Reflect::set(object, &key.into(), value)
        .map(drop)
        .map_err(|_| JsError::new("failed to build result"))
}
