use crate::capture::ScreenImage;
use anyhow::{bail, Context, Result};
use image::{ImageBuffer, Rgba};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn encode_to_png(img: &ScreenImage) -> Result<Vec<u8>> {
    let rgba_img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(img.width, img.height, img.data.clone())
        .context("Failed to construct image buffer from ScreenImage data")?;

    let mut buffer = std::io::Cursor::new(Vec::new());
    rgba_img.write_to(&mut buffer, image::ImageFormat::Png)
        .context("Failed to encode image to PNG format")?;

    Ok(buffer.into_inner())
}

pub fn save_to_png(img: &ScreenImage, path: impl AsRef<Path>) -> Result<()> {
    let png_bytes = encode_to_png(img)?;
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(p, png_bytes).with_context(|| format!("Failed to write PNG file to {:?}", p))?;
    Ok(())
}

pub fn copy_to_clipboard(img: &ScreenImage) -> Result<()> {
    let png_bytes = encode_to_png(img)?;

    
    let mut child = Command::new("wl-copy")
        .arg("-t")
        .arg("image/png")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn wl-copy. Is wl-clipboard installed?")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&png_bytes).context("Failed to pipe PNG bytes to wl-copy stdin")?;
    }

    let status = child.wait().context("Failed waiting for wl-copy process")?;
    if !status.success() {
        bail!("wl-copy exited with error: {:?}", status.code());
    }

    Ok(())
}

pub fn copy_text_to_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn wl-copy. Is wl-clipboard installed?")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).context("Failed to pipe text to wl-copy stdin")?;
    }

    let status = child.wait().context("Failed waiting for wl-copy process")?;
    if !status.success() {
        bail!("wl-copy exited with error: {:?}", status.code());
    }

    Ok(())
}
