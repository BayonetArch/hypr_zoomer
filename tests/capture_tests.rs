use hypr_zoomer::capture::{parse_ppm, ScreenImage};

#[test]
fn test_parse_ppm_valid() {
    
    let header = b"P6\n# Captured with grim\n2 2\n255\n";
    let pixels: [u8; 12] = [
        255, 0, 0,     
        0, 255, 0,     
        0, 0, 255,     
        255, 255, 255, 
    ];

    let mut data = Vec::new();
    data.extend_from_slice(header);
    data.extend_from_slice(&pixels);

    let img = parse_ppm(&data).expect("Should parse valid PPM data");
    assert_eq!(img.width, 2);
    assert_eq!(img.height, 2);
    assert_eq!(img.get_pixel(0, 0), Some([255, 0, 0, 255]));
    assert_eq!(img.get_pixel(1, 0), Some([0, 255, 0, 255]));
    assert_eq!(img.get_pixel(0, 1), Some([0, 0, 255, 255]));
    assert_eq!(img.get_pixel(1, 1), Some([255, 255, 255, 255]));
}

#[test]
fn test_parse_ppm_with_multiple_comments_and_spaces() {
    let raw = b"P6\n# First comment\n# Second comment\n  3  2  \n 255 \n\
        \x01\x02\x03\x04\x05\x06\x07\x08\x09\
        \x0A\x0B\x0C\x0D\x0E\x0F\x10\x11\x12";

    let img = parse_ppm(raw).expect("Should parse PPM with spaces and comments");
    assert_eq!(img.width, 3);
    assert_eq!(img.height, 2);
    assert_eq!(img.get_pixel(0, 0), Some([1, 2, 3, 255]));
    assert_eq!(img.get_pixel(2, 1), Some([0x10, 0x11, 0x12, 255]));
}

#[test]
fn test_parse_ppm_invalid_headers() {
    
    assert!(parse_ppm(b"P3\n2 2\n255\n1 2 3").is_err());

    
    assert!(parse_ppm(b"P6\n2").is_err());

    
    assert!(parse_ppm(b"P6\n2 2\n255\n12345").is_err());
}

#[test]
fn test_screen_image_crop() {
    
    let mut img = ScreenImage::new(4, 4);
    for y in 0..4 {
        for x in 0..4 {
            img.set_pixel(x, y, [x as u8 * 10, y as u8 * 10, 0, 255]);
        }
    }

    let cropped = img.crop(1, 1, 2, 2).expect("Cropping within bounds should succeed");
    assert_eq!(cropped.width, 2);
    assert_eq!(cropped.height, 2);
    assert_eq!(cropped.get_pixel(0, 0), Some([10, 10, 0, 255]));
    assert_eq!(cropped.get_pixel(1, 1), Some([20, 20, 0, 255]));
}
