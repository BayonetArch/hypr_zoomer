use crate::math::{Vec2, Rect};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct HyprWindow {
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub at: [i32; 2],
    #[serde(default)]
    pub size: [i32; 2],
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub class: String,
}

impl HyprWindow {
    pub fn to_rect(&self) -> Rect {
        Rect::new(
            self.at[0] as f32,
            self.at[1] as f32,
            self.size[0] as f32,
            self.size[1] as f32,
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyprMonitor {
    pub id: i32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    #[serde(default = "default_monitor_scale")]
    pub scale: f32,
    #[serde(default)]
    pub focused: bool,
}

fn default_monitor_scale() -> f32 { 1.0 }

impl HyprMonitor {
    pub fn to_rect(&self) -> Rect {
        Rect::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

pub fn get_active_window() -> Option<HyprWindow> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

pub fn get_monitors() -> Vec<HyprMonitor> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok();

    if let Some(out) = output {
        if out.status.success() {
            if let Ok(monitors) = serde_json::from_slice(&out.stdout) {
                return monitors;
            }
        }
    }
    Vec::new()
}

pub fn get_cursor_position() -> Option<Vec2> {
    let output = Command::new("hyprctl")
        .arg("cursorpos")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = text.trim().split(',').collect();
    if parts.len() == 2 {
        let x: f32 = parts[0].trim().parse().ok()?;
        let y: f32 = parts[1].trim().parse().ok()?;
        return Some(Vec2::new(x, y));
    }
    None
}
