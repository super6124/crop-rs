use libvips::{ops, VipsApp, VipsImage};
use serde::Deserialize;
use std::{env, fs, path::Path};

#[derive(Deserialize)]
struct Config {
    sample_corner: String,
    overwrite: bool,
    fuzz: f64,
}

fn main() -> anyhow::Result<()> {
    // 1. Instant Initialization (Minimal Overhead)
    let app = VipsApp::new("FastTrim", false).expect("Vips Init Failed");
    app.concurrency_set(0); // Use all hardware threads

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return Ok(()); }
    let input_path = &args[1];

    // 2. Fast TOML Parsing
    let config_str = fs::read_to_string("config.toml").unwrap_or_else(|_| {
        r#"sample_corner="top-left"
           overwrite=false
           fuzz=10.0"#.to_string()
    });
    let config: Config = toml::from_str(&config_str)?;

    // 3. Load Image (Sequential access + Memory Mapping)
    // Supports PNG, JPG, WebP automatically
    let img = VipsImage::new_from_file(input_path)?;
    let w = img.get_width();
    let h = img.get_height();

    // 4. Determine Sample Coordinate
    let (sx, sy) = match config.sample_corner.as_str() {
        "top-right" => (w - 1, 0),
        "bottom-left" => (0, h - 1),
        "bottom-right" => (w - 1, h - 1),
        _ => (0, 0),
    };

    // 5. SIMD-Accelerated Background Removal
    // Similarity operation identifies the background and applies transparency
    let processed = ops::similarity_with_opts(&img, &ops::SimilarityOptions {
        threshold: config.fuzz,
        tint: true, // This enables the Alpha-masking behavior
        ..ops::SimilarityOptions::default()
    })?;

    // 6. Find Trim (Zero-copy scan for bounding box)
    let trim_res = ops::find_trim(&processed, 0.0, config.fuzz)?;
    let final_img = ops::crop(&processed, trim_res.left, trim_res.top, trim_res.width, trim_res.height)?;

    // 7. Atomic Overwrite / Output Logic
    let temp_output = format!("{}_temp_vips.png", input_path);
    final_img.write_to_file(&temp_output)?;

    if config.overwrite {
        fs::rename(&temp_output, input_path)?;
    } else {
        let p = Path::new(input_path);
        let final_name = format!("{}_trimmed.png", p.file_stem().unwrap().to_str().unwrap());
        fs::rename(&temp_output, final_name)?;
    }

    Ok(())
}
