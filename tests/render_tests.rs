use hypr_zoomer::capture::ScreenImage;
use hypr_zoomer::camera::Camera;
use hypr_zoomer::config::FilterMode;
use hypr_zoomer::render::{Renderer, Framebuffer};
use hypr_zoomer::effects::Flashlight;
use hypr_zoomer::math::Vec2;

#[test]
fn test_nearest_neighbor_scaling() {
    
    let mut src = ScreenImage::new(2, 2);
    src.set_pixel(0, 0, [255, 0, 0, 255]);     
    src.set_pixel(1, 0, [0, 255, 0, 255]);     
    src.set_pixel(0, 1, [0, 0, 255, 255]);     
    src.set_pixel(1, 1, [255, 255, 255, 255]); 

    
    let mut fb = Framebuffer::new(4, 4);
    let mut cam = Camera::new(4.0, 4.0);
    cam.set_scale_instant(2.0);
    cam.set_offset_instant(Vec2::new(0.0, 0.0));

    let renderer = Renderer::new();
    renderer.blit_image(&src, &mut fb, &cam, FilterMode::Nearest, false, false, false);

    
    
    assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]);
    assert_eq!(fb.get_pixel(1, 0), [255, 0, 0, 255]);
    assert_eq!(fb.get_pixel(0, 1), [255, 0, 0, 255]);
    assert_eq!(fb.get_pixel(1, 1), [255, 0, 0, 255]);

    
    assert_eq!(fb.get_pixel(2, 0), [0, 255, 0, 255]);
    assert_eq!(fb.get_pixel(3, 0), [0, 255, 0, 255]);
    assert_eq!(fb.get_pixel(2, 1), [0, 255, 0, 255]);
    assert_eq!(fb.get_pixel(3, 1), [0, 255, 0, 255]);

    
    assert_eq!(fb.get_pixel(0, 2), [0, 0, 255, 255]);
    assert_eq!(fb.get_pixel(1, 2), [0, 0, 255, 255]);
    assert_eq!(fb.get_pixel(0, 3), [0, 0, 255, 255]);
    assert_eq!(fb.get_pixel(1, 3), [0, 0, 255, 255]);

    
    assert_eq!(fb.get_pixel(2, 2), [255, 255, 255, 255]);
    assert_eq!(fb.get_pixel(3, 2), [255, 255, 255, 255]);
    assert_eq!(fb.get_pixel(2, 3), [255, 255, 255, 255]);
    assert_eq!(fb.get_pixel(3, 3), [255, 255, 255, 255]);
}

#[test]
fn test_render_with_inversion() {
    let mut src = ScreenImage::new(2, 2);
    src.set_pixel(0, 0, [200, 100, 50, 255]);

    let mut fb = Framebuffer::new(2, 2);
    let mut cam = Camera::new(2.0, 2.0);
    cam.set_scale_instant(1.0);

    let renderer = Renderer::new();
    renderer.blit_image(&src, &mut fb, &cam, FilterMode::Nearest, true, false, false); 

    assert_eq!(fb.get_pixel(0, 0), [55, 155, 205, 255]);
}

#[test]
fn test_render_with_flashlight() {
    let mut src = ScreenImage::new(10, 10);
    for y in 0..10 {
        for x in 0..10 {
            src.set_pixel(x, y, [200, 200, 200, 255]);
        }
    }

    let mut fb = Framebuffer::new(10, 10);
    let mut cam = Camera::new(10.0, 10.0);
    cam.set_scale_instant(1.0);

    let fl = Flashlight::new(2.0, 1.0, 0.2); 
    let cursor = Vec2::new(0.0, 0.0);

    let renderer = Renderer::new();
    renderer.blit_image(&src, &mut fb, &cam, FilterMode::Nearest, false, false, false);
    renderer.apply_flashlight(&mut fb, cursor, &fl);

    
    assert_eq!(fb.get_pixel(0, 0), [200, 200, 200, 255]);

    
    let p_far = fb.get_pixel(9, 9);
    assert!((p_far[0] as i32 - 40).abs() <= 2);
}
