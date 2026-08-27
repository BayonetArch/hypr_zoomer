use crate::math::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Flashlight {
    pub radius: f32,
    pub feather: f32,
    pub ambient: f32,
}

impl Flashlight {
    pub fn new(radius: f32, feather: f32, ambient: f32) -> Self {
        Self {
            radius: radius.max(1.0),
            feather: feather.max(0.1),
            ambient: ambient.clamp(0.0, 1.0),
        }
    }

    #[inline]
    pub fn intensity_at(&self, center: Vec2, point: Vec2) -> f32 {
        let dist = center.distance_to(point);
        if dist <= self.radius {
            1.0
        } else if dist >= self.radius + self.feather {
            self.ambient
        } else {
            
            let t = (dist - self.radius) / self.feather;
            let smooth_step = t * t * (3.0 - 2.0 * t);
            1.0 - smooth_step * (1.0 - self.ambient)
        }
    }
}

#[inline]
pub fn invert_color(rgba: [u8; 4]) -> [u8; 4] {
    [255 - rgba[0], 255 - rgba[1], 255 - rgba[2], rgba[3]]
}

#[inline]
pub fn format_hex_color(rgba: [u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2])
}

#[inline]
pub fn calculate_luminance(rgba: [u8; 4]) -> f32 {
    0.299 * rgba[0] as f32 + 0.587 * rgba[1] as f32 + 0.114 * rgba[2] as f32
}

pub fn rgb_to_hsl(rgba: [u8; 4]) -> (f32, f32, f32) {
    let r = rgba[0] as f32 / 255.0;
    let g = rgba[1] as f32 / 255.0;
    let b = rgba[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) * 0.5;

    if delta < 1e-5 {
        return (0.0, 0.0, l * 100.0);
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let mut h = if (max - r).abs() < 1e-5 {
        let seg = (g - b) / delta;
        seg
    } else if (max - g).abs() < 1e-5 {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    } * 60.0;

    if h < 0.0 {
        h += 360.0;
    }

    (h, s * 100.0, l * 100.0)
}

#[inline]
pub fn format_hsl_color(rgba: [u8; 4]) -> String {
    let (h, s, l) = rgb_to_hsl(rgba);
    format!("hsl({:.0}, {:.0}%, {:.0}%)", h.round(), s.round(), l.round())
}
