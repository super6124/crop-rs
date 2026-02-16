#![windows_subsystem = "windows"]

mod config;
mod processor;
mod io;

use crate::config::{AppConfig, Corner};
use crate::processor::parallel_autocrop;
use crate::io::save_png_optimized;

use image::GenericImageView;
use rayon::prelude::*;
use std::env;
use std::path::Path;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let cfg = AppConfig::load();
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() { return; }

    args.par_iter().for_each(|path_str| {
        let path = Path::new(path_str);
        let _ = process_image(path, &cfg);
    });
}

fn process_image(path: &Path, cfg: &AppConfig) -> anyhow::Result<()> {
    let img = image::open(path)?;
    let (w, h) = img.dimensions();

    let (sx, sy) = match cfg.sample_corner {
        Corner::TopLeft => (0, 0),
        Corner::TopRight => (w - 1, 0),
        Corner::BottomLeft => (0, h - 1),
        Corner::BottomRight => (w - 1, h - 1),
    };
    
    let bg_pixel = img.get_pixel(sx, sy);

    if let Some(cropped) = parallel_autocrop(&img, bg_pixel, cfg.tolerance) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
        let out_path = if cfg.replace_source {
            path.to_path_buf()
        } else {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            path.with_file_name(format!("{}_cropped.{}", stem, ext))
        };

        if ext == "png" {
            save_png_optimized(&cropped, &out_path, cfg.png_compression_level)?;
        } else {
            cropped.save(&out_path)?; // JPG will use internal defaults
        }
    }
    Ok(())
}
