use crate::annotations::{Annotation, Stroke, Arrow, RectAnnotation};
use crate::camera::Camera;
use crate::capture::ScreenImage;
use crate::config::FilterMode;
use crate::effects::{Flashlight, invert_color};
use crate::math::Vec2;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, 
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.data.resize((width * height * 4) as usize, 0);
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ]
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = pixel[0];
        self.data[idx + 1] = pixel[1];
        self.data[idx + 2] = pixel[2];
        self.data[idx + 3] = pixel[3];
    }

    #[inline]
    pub fn copy_to_softbuffer(&self, dest: &mut [u32]) {
        let count = (self.width * self.height) as usize;
        let dst_len = dest.len().min(count);
        
        
        dest[..dst_len]
            .par_chunks_mut(1024)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                let start_pixel = chunk_idx * 1024;
                for (i, dst_pixel) in chunk.iter_mut().enumerate() {
                    let src_idx = (start_pixel + i) * 4;
                    if src_idx + 3 < self.data.len() {
                        let r = self.data[src_idx] as u32;
                        let g = self.data[src_idx + 1] as u32;
                        let b = self.data[src_idx + 2] as u32;
                        *dst_pixel = (r << 16) | (g << 8) | b;
                    }
                }
            });
    }
}

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    pub fn blit_image(
        &self,
        src: &ScreenImage,
        fb: &mut Framebuffer,
        camera: &Camera,
        filter_mode: FilterMode,
        invert: bool,
        mirror_h: bool,
        mirror_v: bool,
    ) {
        let fb_width = fb.width;
        let src_width = src.width as f32;
        let src_height = src.height as f32;

        let scale = camera.scale();
        let offset = camera.offset();

        
        fb.data
            .par_chunks_exact_mut((fb_width * 4) as usize)
            .enumerate()
            .for_each(|(dst_y_usize, row_bytes)| {
                let dst_y = dst_y_usize as f32;

                for dst_x_usize in 0..fb_width as usize {
                    let dst_x = dst_x_usize as f32;

                    
                    let mut wx = (dst_x - offset.x) / scale;
                    let mut wy = (dst_y - offset.y) / scale;

                    if mirror_h {
                        wx = src_width - 1.0 - wx;
                    }
                    if mirror_v {
                        wy = src_height - 1.0 - wy;
                    }

                    let pixel = if wx < 0.0 || wx >= src_width || wy < 0.0 || wy >= src_height {
                        
                        let check = ((dst_x_usize / 16) + (dst_y_usize / 16)) % 2 == 0;
                        if check { [24, 24, 28, 255] } else { [18, 18, 22, 255] }
                    } else {
                        match filter_mode {
                            FilterMode::Nearest => {
                                let sx = wx.floor() as u32;
                                let sy = wy.floor() as u32;
                                src.get_pixel(sx, sy).unwrap_or([0, 0, 0, 255])
                            }
                            FilterMode::Bilinear => {
                                let x0 = wx.floor() as u32;
                                let y0 = wy.floor() as u32;
                                let x1 = (x0 + 1).min(src.width - 1);
                                let y1 = (y0 + 1).min(src.height - 1);

                                let fx = wx - wx.floor();
                                let fy = wy - wy.floor();

                                let p00 = src.get_pixel(x0, y0).unwrap_or([0, 0, 0, 255]);
                                let p10 = src.get_pixel(x1, y0).unwrap_or([0, 0, 0, 255]);
                                let p01 = src.get_pixel(x0, y1).unwrap_or([0, 0, 0, 255]);
                                let p11 = src.get_pixel(x1, y1).unwrap_or([0, 0, 0, 255]);

                                let mut res = [0u8; 4];
                                for c in 0..3 {
                                    let top = p00[c] as f32 * (1.0 - fx) + p10[c] as f32 * fx;
                                    let bot = p01[c] as f32 * (1.0 - fx) + p11[c] as f32 * fx;
                                    let val = (top * (1.0 - fy) + bot * fy).round();
                                    res[c] = val.clamp(0.0, 255.0) as u8;
                                }
                                res[3] = 255;
                                res
                            }
                        }
                    };

                    let final_pixel = if invert {
                        invert_color(pixel)
                    } else {
                        pixel
                    };

                    let px_idx = dst_x_usize * 4;
                    row_bytes[px_idx] = final_pixel[0];
                    row_bytes[px_idx + 1] = final_pixel[1];
                    row_bytes[px_idx + 2] = final_pixel[2];
                    row_bytes[px_idx + 3] = final_pixel[3];
                }
            });
    }

    pub fn apply_flashlight(&self, fb: &mut Framebuffer, cursor: Vec2, flashlight: &Flashlight) {
        let width = fb.width;
        let radius = flashlight.radius;
        let feather = flashlight.feather;
        let ambient = flashlight.ambient;

        fb.data
            .par_chunks_exact_mut((width * 4) as usize)
            .enumerate()
            .for_each(|(y_usize, row)| {
                let y = y_usize as f32;
                let dy = y - cursor.y;
                let dy_sq = dy * dy;

                for x_usize in 0..width as usize {
                    let x = x_usize as f32;
                    let dx = x - cursor.x;
                    let dist = (dx * dx + dy_sq).sqrt();

                    let factor = if dist <= radius {
                        1.0
                    } else if dist >= radius + feather {
                        ambient
                    } else {
                        let t = (dist - radius) / feather;
                        let smooth = t * t * (3.0 - 2.0 * t);
                        1.0 - smooth * (1.0 - ambient)
                    };

                    let idx = x_usize * 4;

                    if factor < 0.999 {
                        row[idx] = ((row[idx] as f32) * factor).round() as u8;
                        row[idx + 1] = ((row[idx + 1] as f32) * factor).round() as u8;
                        row[idx + 2] = ((row[idx + 2] as f32) * factor).round() as u8;
                    }
                }
            });
    }

    pub fn draw_pixel_grid(&self, fb: &mut Framebuffer, camera: &Camera, src: &ScreenImage) {
        if camera.scale() < 8.0 {
            return;
        }

        let scale = camera.scale();
        let offset = camera.offset();

        
        let top_left = camera.screen_to_world(Vec2::ZERO);
        let bottom_right = camera.screen_to_world(Vec2::new(fb.width as f32, fb.height as f32));

        let start_x = (top_left.x.floor() as i32).max(0);
        let end_x = (bottom_right.x.ceil() as i32).min(src.width as i32);
        let start_y = (top_left.y.floor() as i32).max(0);
        let end_y = (bottom_right.y.ceil() as i32).min(src.height as i32);

        let grid_color = [120, 120, 120, 90]; 

        
        for wx in start_x..=end_x {
            let sx = (wx as f32 * scale + offset.x).round() as i32;
            if sx >= 0 && sx < fb.width as i32 {
                for y in 0..fb.height {
                    blend_pixel(fb, sx as u32, y, grid_color);
                }
            }
        }

        
        for wy in start_y..=end_y {
            let sy = (wy as f32 * scale + offset.y).round() as i32;
            if sy >= 0 && sy < fb.height as i32 {
                for x in 0..fb.width {
                    blend_pixel(fb, x, sy as u32, grid_color);
                }
            }
        }
    }

    pub fn render_annotations(&self, fb: &mut Framebuffer, camera: &Camera, items: &[Annotation]) {
        for item in items {
            match item {
                Annotation::Stroke(stroke) => {
                    self.draw_stroke(fb, camera, stroke);
                }
                Annotation::Arrow(arrow) => {
                    self.draw_arrow(fb, camera, arrow);
                }
                Annotation::Rect(rect_ann) => {
                    self.draw_rect(fb, camera, rect_ann);
                }
            }
        }
    }

    fn draw_stroke(&self, fb: &mut Framebuffer, camera: &Camera, stroke: &Stroke) {
        if stroke.points.len() < 2 {
            return;
        }

        for i in 0..stroke.points.len() - 1 {
            let p1 = camera.world_to_screen(stroke.points[i]);
            let p2 = camera.world_to_screen(stroke.points[i + 1]);
            draw_thick_line(fb, p1, p2, stroke.thickness, stroke.color.to_array());
        }
    }

    fn draw_arrow(&self, fb: &mut Framebuffer, camera: &Camera, arrow: &Arrow) {
        let p1 = camera.world_to_screen(arrow.start);
        let p2 = camera.world_to_screen(arrow.end);
        let color = arrow.color.to_array();

        
        draw_thick_line(fb, p1, p2, arrow.thickness, color);

        
        let dir = (p1 - p2).normalize();
        if dir.length_squared() > 0.0 {
            let head_len = (arrow.thickness * 4.0).clamp(12.0, 36.0);
            let angle: f32 = 0.5; 

            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let wing1 = Vec2::new(
                dir.x * cos_a - dir.y * sin_a,
                dir.x * sin_a + dir.y * cos_a,
            ) * head_len;

            let wing2 = Vec2::new(
                dir.x * cos_a + dir.y * sin_a,
                -dir.x * sin_a + dir.y * cos_a,
            ) * head_len;

            draw_thick_line(fb, p2, p2 + wing1, arrow.thickness, color);
            draw_thick_line(fb, p2, p2 + wing2, arrow.thickness, color);
        }
    }

    fn draw_rect(&self, fb: &mut Framebuffer, camera: &Camera, ann: &RectAnnotation) {
        let p1 = camera.world_to_screen(Vec2::new(ann.rect.x, ann.rect.y));
        let p2 = camera.world_to_screen(Vec2::new(ann.rect.right(), ann.rect.bottom()));
        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let max_x = p1.x.max(p2.x);
        let max_y = p1.y.max(p2.y);

        let color = ann.color.to_array();

        if ann.filled {
            let fill_color = [color[0], color[1], color[2], 75]; 
            let x_start = (min_x as i32).max(0) as u32;
            let x_end = (max_x as i32).min(fb.width as i32 - 1).max(0) as u32;
            let y_start = (min_y as i32).max(0) as u32;
            let y_end = (max_y as i32).min(fb.height as i32 - 1).max(0) as u32;

            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    blend_pixel(fb, x, y, fill_color);
                }
            }
        }

        
        let tl = Vec2::new(min_x, min_y);
        let tr = Vec2::new(max_x, min_y);
        let br = Vec2::new(max_x, max_y);
        let bl = Vec2::new(min_x, max_y);

        draw_thick_line(fb, tl, tr, ann.thickness, color);
        draw_thick_line(fb, tr, br, ann.thickness, color);
        draw_thick_line(fb, br, bl, ann.thickness, color);
        draw_thick_line(fb, bl, tl, ann.thickness, color);
    }
}

#[inline]
pub fn blend_pixel_pub(fb: &mut Framebuffer, x: u32, y: u32, src_color: [u8; 4]) {
    blend_pixel(fb, x, y, src_color);
}

#[inline]
fn blend_pixel(fb: &mut Framebuffer, x: u32, y: u32, src_color: [u8; 4]) {
    if x >= fb.width || y >= fb.height {
        return;
    }
    let idx = ((y * fb.width + x) * 4) as usize;
    let alpha = src_color[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let dst_r = fb.data[idx] as f32;
    let dst_g = fb.data[idx + 1] as f32;
    let dst_b = fb.data[idx + 2] as f32;

    fb.data[idx] = (src_color[0] as f32 * alpha + dst_r * inv_alpha).round() as u8;
    fb.data[idx + 1] = (src_color[1] as f32 * alpha + dst_g * inv_alpha).round() as u8;
    fb.data[idx + 2] = (src_color[2] as f32 * alpha + dst_b * inv_alpha).round() as u8;
}

fn draw_thick_line(fb: &mut Framebuffer, p1: Vec2, p2: Vec2, thickness: f32, color: [u8; 4]) {
    let radius = (thickness * 0.5).max(0.5);

    let min_x = (p1.x.min(p2.x) - radius - 1.0).floor().max(0.0) as i32;
    let max_x = (p1.x.max(p2.x) + radius + 1.0).ceil().min(fb.width as f32 - 1.0) as i32;
    let min_y = (p1.y.min(p2.y) - radius - 1.0).floor().max(0.0) as i32;
    let max_y = (p1.y.max(p2.y) + radius + 1.0).ceil().min(fb.height as f32 - 1.0) as i32;

    let line_vec = p2 - p1;
    let len_sq = line_vec.length_squared();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let pt = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let dist = if len_sq < 1e-4 {
                pt.distance_to(p1)
            } else {
                let t = ((pt - p1).dot(line_vec) / len_sq).clamp(0.0, 1.0);
                let proj = p1 + line_vec * t;
                pt.distance_to(proj)
            };

            if dist <= radius {
                blend_pixel(fb, x as u32, y as u32, color);
            } else if dist < radius + 1.0 {
                let aa_alpha = ((1.0 - (dist - radius)) * (color[3] as f32)).round() as u8;
                blend_pixel(fb, x as u32, y as u32, [color[0], color[1], color[2], aa_alpha]);
            }
        }
    }
}
