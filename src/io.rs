use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use oxipng::{BitDepth, ColorType, Options, RawImage};
use std::path::Path;

pub fn save_png_optimized(img: &DynamicImage, path: &Path, level: u8) -> Result<()> {
    let (w, h) = img.dimensions();
    let raw_data = img.to_rgba8().into_raw();
    let raw_img = RawImage::new(w, h, ColorType::RGBA, BitDepth::Eight, raw_data)
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut options = Options::from_preset(level);
    options.fix_errors = true;
    let optimized = raw_img.create_optimized_png(&options).map_err(|e| anyhow::anyhow!(e))?;
    std::fs::write(path, optimized)?;
    Ok(())
}
