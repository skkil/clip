# clip-ocr

The OCR engine shared by the desktop app and (later) the Chrome extension. One
Rust crate, compiled natively into `clip-desktop` and to WebAssembly for the
browser, so both produce the same text from the same pixels.

| | |
| --- | --- |
| Models | PaddleOCR PP-OCRv5 mobile: one detector, one recogniser per script family |
| Runtime | [`tract`](https://github.com/sonos/tract), pure Rust, no native dependency |
| Licences | crate MIT; `tract` MIT/Apache-2.0; models Apache-2.0 (see `NOTICE`) |

Apache-2.0 requires shipping its licence text and `NOTICE` alongside the
models; `scripts/fetch-models.py` copies both next to them.

## Languages

Each recogniser covers a script family, around 100 languages in total.

| Script | Covers |
| --- | --- |
| `korean` | Korean, English |
| `chinese` | Simplified and Traditional Chinese, Japanese, English |
| `latin` | English, French, German, Spanish, Italian, Portuguese and 30+ more |
| `cyrillic` | Russian, Ukrainian, Belarusian, Serbian, Bulgarian, Mongolian and more |
| `eslav` | Russian, Ukrainian, Belarusian |
| `arabic` | Arabic, Persian, Urdu, Uyghur, Kurdish |
| `devanagari` | Hindi, Marathi, Nepali, Sanskrit |
| `greek` `thai` `tamil` `telugu` | that script, plus English |
| `english` | English only |

Every loaded recogniser is tried on each line and the best score wins,
stopping early on a confident read. Fewer scripts is faster.

## Models

Not committed (≈109 MB). Fetch the pinned revisions, verified by SHA-256
(needs `pyyaml`):

```
scripts/fetch-models.py                                  # into ./models
scripts/fetch-models.py --out ~/.local/share/clip/ocr    # for the desktop app
scripts/fetch-models.py korean latin                     # a subset
```

## Usage

```
cargo run --release --example ocr -- image.png korean latin
```

```rust
let mut engine = clip_ocr::Engine::new(&det_onnx)?;
engine.add_recognizer("korean", &rec_onnx, &dict)?;
let lines = engine.recognize_rgba(&rgba, width, height)?;
println!("{}", clip_ocr::to_text(&lines));
```

## WebAssembly

The `wasm` feature exposes `OcrEngine` through `wasm-bindgen`:

```
RUSTFLAGS="-C target-feature=+simd128" \
  cargo build --release --target wasm32-unknown-unknown --features wasm
wasm-bindgen --target web --out-dir pkg \
  target/wasm32-unknown-unknown/release/clip_ocr.wasm
```

`simd128` roughly halves inference time and is supported by every browser
CLIP targets.

## Implementation notes

- **tract cannot prove the detector's shapes symbolically.** Its FPN concat
  needs `(32h+31)/32 = h`, which tract does not simplify, so a plan is compiled
  per concrete input shape (≈100 ms) and the last eight are cached. Recogniser
  widths are rounded up to multiples of 160 to keep the cache hit rate high.
- **Detection runs at native resolution**, clamped to 64–4000 px per side,
  because screen text is already sharp; downscaling a 4K screen loses it.
- **Boxes are axis-aligned.** Screen text is not rotated, so DB
  post-processing keeps the bounding box of each component and skips polygon
  unclipping.
- **Arabic comes out in visual order** and is reversed the way PaddleOCR does,
  keeping Latin and digit runs intact.
- **PP-OCRv6 is not used**: its multilingual recogniser does not cover Korean.
