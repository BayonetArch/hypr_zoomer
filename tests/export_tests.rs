use hypr_zoomer::capture::ScreenImage;
use hypr_zoomer::export::{encode_to_png, save_to_png};
use std::fs;

#[test]
fn test_encode_to_png_roundtrip() {
    let mut img = ScreenImage::new(4, 4);
    img.set_pixel(0, 0, [255, 0, 0, 255]);
    img.set_pixel(1, 1, [0, 255, 0, 255]);
    img.set_pixel(2, 2, [0, 0, 255, 255]);
    img.set_pixel(3, 3, [255, 255, 255, 255]);

    let png_bytes = encode_to_png(&img).expect("Should encode image to PNG");
    assert!(!png_bytes.is_empty());
    
    assert_eq!(&png_bytes[0..4], b"\x89PNG");

    
    let loaded = image::load_from_memory(&png_bytes).expect("Should decode generated PNG").to_rgba8();
    assert_eq!(loaded.width(), 4);
    assert_eq!(loaded.height(), 4);
    assert_eq!(loaded.get_pixel(0, 0).0, [255, 0, 0, 255]);
    assert_eq!(loaded.get_pixel(1, 1).0, [0, 255, 0, 255]);
}

#[test]
fn test_save_to_png_file() {
    let img = ScreenImage::new(2, 2);
    let tmp_path = "/tmp/hypr_zoomer_test_export.png";

    save_to_png(&img, tmp_path).expect("Should save PNG to file");
    assert!(fs::metadata(tmp_path).is_ok());

    let _ = fs::remove_file(tmp_path);
}

#[test]
fn test_copy_text_to_clipboard_invocation() {
    use hypr_zoomer::export::copy_text_to_clipboard;
    
    let res = copy_text_to_clipboard("#FF0080");
    if std::process::Command::new("which").arg("wl-copy").output().map(|o| o.status.success()).unwrap_or(false) {
        assert!(res.is_ok());
    }
}
