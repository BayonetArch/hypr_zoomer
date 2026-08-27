use anyhow::{bail, Context, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ScreenImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, 
}

impl ScreenImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        }
    }

    pub fn from_rgb(width: u32, height: u32, rgb: &[u8]) -> Result<Self> {
        let expected_len = (width * height * 3) as usize;
        if rgb.len() < expected_len {
            bail!("Insufficient RGB data: expected {} bytes, got {}", expected_len, rgb.len());
        }

        let mut data = Vec::with_capacity((width * height * 4) as usize);
        for chunk in rgb[..expected_len].chunks_exact(3) {
            data.push(chunk[0]); 
            data.push(chunk[1]); 
            data.push(chunk[2]); 
            data.push(255);      
        }

        Ok(Self { width, height, data })
    }

    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ])
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

    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
        if x + width > self.width || y + height > self.height || width == 0 || height == 0 {
            return None;
        }

        let mut cropped = Self::new(width, height);
        for cy in 0..height {
            let src_y = y + cy;
            let src_start = ((src_y * self.width + x) * 4) as usize;
            let src_end = src_start + (width * 4) as usize;
            let dst_start = (cy * width * 4) as usize;
            let dst_end = dst_start + (width * 4) as usize;

            cropped.data[dst_start..dst_end].copy_from_slice(&self.data[src_start..src_end]);
        }

        Some(cropped)
    }
}

pub fn parse_ppm(data: &[u8]) -> Result<ScreenImage> {
    let mut pos = 0;

    
    fn skip_whitespace_and_comments(data: &[u8], pos: &mut usize) {
        while *pos < data.len() {
            let b = data[*pos];
            if b.is_ascii_whitespace() {
                *pos += 1;
            } else if b == b'#' {
                
                while *pos < data.len() && data[*pos] != b'\n' {
                    *pos += 1;
                }
                if *pos < data.len() && data[*pos] == b'\n' {
                    *pos += 1;
                }
            } else {
                break;
            }
        }
    }

    
    fn next_token<'a>(data: &'a [u8], pos: &mut usize) -> Result<&'a str> {
        skip_whitespace_and_comments(data, pos);
        if *pos >= data.len() {
            bail!("Unexpected EOF while reading PPM header");
        }
        let start = *pos;
        while *pos < data.len() && !data[*pos].is_ascii_whitespace() && data[*pos] != b'#' {
            *pos += 1;
        }
        let token = std::str::from_utf8(&data[start..*pos])
            .context("Invalid UTF-8 in PPM header")?;
        Ok(token)
    }

    
    let magic = next_token(data, &mut pos)?;
    if magic != "P6" {
        bail!("Invalid PPM magic: expected P6, got '{}'", magic);
    }

    
    let width_str = next_token(data, &mut pos)?;
    let width: u32 = width_str.parse().context("Invalid PPM width")?;

    let height_str = next_token(data, &mut pos)?;
    let height: u32 = height_str.parse().context("Invalid PPM height")?;

    let maxval_str = next_token(data, &mut pos)?;
    let maxval: u32 = maxval_str.parse().context("Invalid PPM maxval")?;
    if maxval != 255 {
        bail!("Unsupported PPM maxval: expected 255, got {}", maxval);
    }

    
    skip_whitespace_and_comments(data, &mut pos);
    if pos >= data.len() {
        bail!("Missing binary pixel data in PPM");
    }

    let payload = &data[pos..];
    ScreenImage::from_rgb(width, height, payload)
}

pub fn capture_screen(geometry: Option<&str>, output_name: Option<&str>, include_cursor: bool) -> Result<ScreenImage> {
    let mut cmd = Command::new("grim");
    cmd.arg("-t").arg("ppm");

    if include_cursor {
        cmd.arg("-c");
    }

    if let Some(geom) = geometry {
        cmd.arg("-g").arg(geom);
    }

    if let Some(out) = output_name {
        cmd.arg("-o").arg(out);
    }

    cmd.arg("-"); 

    let output = cmd.output().context("Failed to execute grim. Is grim installed?")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("grim capture failed: {}", stderr);
    }

    parse_ppm(&output.stdout)
}
