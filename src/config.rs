use serde::{Deserialize, Serialize};
use std::{fs, env};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Corner {
    TopLeft, TopRight, BottomLeft, BottomRight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub sample_corner: Corner,
    pub replace_source: bool,
    pub png_compression_level: u8, // 0-6
    pub jpeg_quality: u8,          // 1-100
    pub tolerance: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sample_corner: Corner::TopLeft,
            replace_source: false,
            png_compression_level: 2,
            jpeg_quality: 85,
            tolerance: 20,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let mut config_path = env::current_exe().unwrap_or_default();
        config_path.set_file_name("config.toml");

        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(cfg) = toml::from_str(&content) {
                return cfg;
            }
        }

        // If file missing/corrupt, generate default next to .exe
        let default_cfg = Self::default();
        if let Ok(toml_str) = toml::to_string_pretty(&default_cfg) {
            let _ = fs::write(config_path, toml_str);
        }
        default_cfg
    }
}
