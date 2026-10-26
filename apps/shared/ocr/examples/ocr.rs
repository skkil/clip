use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let image_path = PathBuf::from(args.next().ok_or("usage: ocr <image.png> [script...]")?);
    let scripts: Vec<String> = args.collect();
    let scripts: Vec<&str> = if scripts.is_empty() {
        clip_ocr::DEFAULT_SCRIPTS.to_vec()
    } else {
        scripts.iter().map(String::as_str).collect()
    };
    let models = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("models");

    let started = Instant::now();
    let mut engine = clip_ocr::Engine::new(&std::fs::read(models.join("det.onnx"))?)?;
    for script in scripts {
        engine.add_recognizer(
            script,
            &std::fs::read(models.join("rec").join(format!("{script}.onnx")))?,
            &std::fs::read_to_string(models.join("rec").join(format!("{script}.txt")))?,
        )?;
    }
    eprintln!("load: {:?}", started.elapsed());

    let image = image::open(image_path)?.to_rgba8();
    let started = Instant::now();
    let lines = engine.recognize_rgba(
        image.as_raw(),
        image.width() as usize,
        image.height() as usize,
    )?;
    eprintln!("recognize: {:?}", started.elapsed());
    for line in &lines {
        eprintln!("{:.3} {:<8} {:?}", line.score, line.script, line.text);
    }
    println!("{}", clip_ocr::to_text(&lines));
    Ok(())
}
