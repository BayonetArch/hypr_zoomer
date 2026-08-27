use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterMode {
    Nearest,
    Bilinear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_scroll_speed")]
    pub scroll_speed: f32,
    #[serde(default = "default_scale_friction")]
    pub scale_friction: f32,
    #[serde(default = "default_drag_friction")]
    pub drag_friction: f32,
    #[serde(default = "default_min_scale")]
    pub min_scale: f32,
    #[serde(default = "default_max_scale")]
    pub max_scale: f32,
    #[serde(default)]
    pub auto_track_active_window: bool,
}

fn default_scroll_speed() -> f32 { 0.15 }
fn default_scale_friction() -> f32 { 0.85 }
fn default_drag_friction() -> f32 { 0.85 }
fn default_min_scale() -> f32 { 1.0 }
fn default_max_scale() -> f32 { 64.0 }

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            scroll_speed: default_scroll_speed(),
            scale_friction: default_scale_friction(),
            drag_friction: default_drag_friction(),
            min_scale: default_min_scale(),
            max_scale: default_max_scale(),
            auto_track_active_window: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsConfig {
    #[serde(default = "default_flashlight_radius")]
    pub flashlight_radius: f32,
    #[serde(default = "default_flashlight_feather")]
    pub flashlight_feather: f32,
    #[serde(default = "default_flashlight_ambient")]
    pub flashlight_ambient: f32,
    #[serde(default = "default_grid_min_scale")]
    pub grid_min_scale: f32,
}

fn default_flashlight_radius() -> f32 { 150.0 }
fn default_flashlight_feather() -> f32 { 25.0 }
fn default_flashlight_ambient() -> f32 { 0.2 }
fn default_grid_min_scale() -> f32 { 8.0 }

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            flashlight_radius: default_flashlight_radius(),
            flashlight_feather: default_flashlight_feather(),
            flashlight_ambient: default_flashlight_ambient(),
            grid_min_scale: default_grid_min_scale(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub show_color_picker: bool,
    #[serde(default = "default_true")]
    pub show_coords: bool,
}

fn default_true() -> bool { true }

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_color_picker: true,
            show_coords: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    #[serde(default = "default_filter_mode")]
    pub filter_mode: FilterMode,
    #[serde(default = "default_target_fps")]
    pub target_fps: u32,
}

fn default_filter_mode() -> FilterMode { FilterMode::Bilinear }
fn default_target_fps() -> u32 { 120 }

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            filter_mode: default_filter_mode(),
            target_fps: default_target_fps(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub effects: EffectsConfig,
    #[serde(default)]
    pub hud: HudConfig,
    #[serde(default)]
    pub render: RenderConfig,
}

impl Config {
    pub fn from_toml_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        Some(Path::new(&home).join(".config/hypr_zoomer/config.toml"))
    }

    pub fn load_or_default() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(cfg) = Self::from_toml_str(&contents) {
                        return cfg;
                    }
                }
            }
        }
        Self::default()
    }
}
