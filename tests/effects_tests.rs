use hypr_zoomer::effects::{Flashlight, invert_color, format_hex_color, calculate_luminance};
use hypr_zoomer::math::Vec2;

#[test]
fn test_flashlight_intensity() {
    let fl = Flashlight::new(100.0, 20.0, 0.2); 
    let center = Vec2::new(500.0, 500.0);

    
    assert_eq!(fl.intensity_at(center, center), 1.0);

    
    let inner_pt = Vec2::new(550.0, 500.0);
    assert_eq!(fl.intensity_at(center, inner_pt), 1.0);

    
    let outer_pt = Vec2::new(700.0, 500.0);
    assert_eq!(fl.intensity_at(center, outer_pt), 0.2);

    
    let edge_pt = Vec2::new(610.0, 500.0);
    let intensity = fl.intensity_at(center, edge_pt);
    assert!(intensity > 0.2 && intensity < 1.0);
}

#[test]
fn test_invert_color() {
    let orig = [100, 150, 200, 255];
    let inverted = invert_color(orig);
    assert_eq!(inverted, [155, 105, 55, 255]);
    assert_eq!(invert_color(inverted), orig);
}

#[test]
fn test_format_hex_color() {
    assert_eq!(format_hex_color([255, 0, 128, 255]), "#FF0080");
    assert_eq!(format_hex_color([0, 255, 0, 255]), "#00FF00");
    assert_eq!(format_hex_color([15, 16, 17, 255]), "#0F1011");
}

#[test]
fn test_calculate_luminance() {
    let black_lum = calculate_luminance([0, 0, 0, 255]);
    assert_eq!(black_lum, 0.0);

    let white_lum = calculate_luminance([255, 255, 255, 255]);
    assert!((white_lum - 255.0).abs() < 1e-4);

    let green_lum = calculate_luminance([0, 255, 0, 255]);
    let blue_lum = calculate_luminance([0, 0, 255, 255]);
    assert!(green_lum > blue_lum); 
}

#[test]
fn test_rgb_to_hsl_and_format() {
    use hypr_zoomer::effects::format_hsl_color;
    assert_eq!(format_hsl_color([255, 0, 0, 255]), "hsl(0, 100%, 50%)");
    assert_eq!(format_hsl_color([0, 255, 0, 255]), "hsl(120, 100%, 50%)");
    assert_eq!(format_hsl_color([0, 0, 255, 255]), "hsl(240, 100%, 50%)");
    assert_eq!(format_hsl_color([255, 255, 255, 255]), "hsl(0, 0%, 100%)");
    assert_eq!(format_hsl_color([0, 0, 0, 255]), "hsl(0, 0%, 0%)");
}
