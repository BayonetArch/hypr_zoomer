use hypr_zoomer::camera::Camera;
use hypr_zoomer::math::{Vec2, Rect};

#[test]
fn test_camera_initial_state() {
    let cam = Camera::new(1920.0, 1080.0);
    assert_eq!(cam.scale(), 1.0);
    assert_eq!(cam.target_scale(), 1.0);
    assert_eq!(cam.offset(), Vec2::new(0.0, 0.0));
    assert_eq!(cam.viewport_size(), Vec2::new(1920.0, 1080.0));
}

#[test]
fn test_coordinate_transformations() {
    let mut cam = Camera::new(1920.0, 1080.0);
    cam.set_scale_instant(2.0);
    cam.set_offset_instant(Vec2::new(-100.0, -50.0));

    let screen_pt = Vec2::new(500.0, 400.0);
    let world_pt = cam.screen_to_world(screen_pt);

    
    
    
    assert_eq!(world_pt, Vec2::new(300.0, 225.0));

    let back_to_screen = cam.world_to_screen(world_pt);
    assert_eq!(back_to_screen, screen_pt);
}

#[test]
fn test_zoom_around_pivot_invariance() {
    let mut cam = Camera::new(1920.0, 1080.0);
    let pivot = Vec2::new(600.0, 400.0);

    
    let world_before = cam.screen_to_world(pivot);

    
    cam.zoom_at(pivot, 1.5);
    cam.apply_instant_target(); 

    
    let world_after = cam.screen_to_world(pivot);

    
    assert!((world_before.x - world_after.x).abs() < 1e-4);
    assert!((world_before.y - world_after.y).abs() < 1e-4);
}

#[test]
fn test_camera_smooth_interpolation_physics() {
    let mut cam = Camera::new(1920.0, 1080.0);
    cam.set_scale_friction(0.85);
    cam.set_drag_friction(0.85);

    let pivot = Vec2::new(960.0, 540.0);
    cam.zoom_at(pivot, 2.0);

    assert_eq!(cam.target_scale(), 2.0);
    assert_eq!(cam.scale(), 1.0); 

    
    for _ in 0..10 {
        cam.update(1.0 / 60.0);
    }

    
    assert!(cam.scale() > 1.0 && cam.scale() <= 2.0);

    
    for _ in 0..120 {
        cam.update(1.0 / 60.0);
    }

    assert!((cam.scale() - 2.0).abs() < 1e-3);
}

#[test]
fn test_camera_min_max_scale_clamping() {
    let mut cam = Camera::new(1920.0, 1080.0);
    cam.set_min_scale(0.5);
    cam.set_max_scale(10.0);

    let pivot = Vec2::new(0.0, 0.0);

    
    for _ in 0..20 {
        cam.zoom_at(pivot, 0.5);
    }
    assert!(cam.target_scale() >= 0.5);

    
    for _ in 0..50 {
        cam.zoom_at(pivot, 2.0);
    }
    assert!(cam.target_scale() <= 10.0);
}

#[test]
fn test_camera_drag_and_pan() {
    let mut cam = Camera::new(1920.0, 1080.0);
    let start_pos = Vec2::new(100.0, 100.0);
    cam.drag_start(start_pos);

    let current_pos = Vec2::new(150.0, 120.0);
    cam.drag_update(current_pos);

    assert_eq!(cam.offset(), Vec2::new(50.0, 20.0));

    cam.drag_end();
    assert!(!cam.is_dragging());
}

#[test]
fn test_camera_reset() {
    let mut cam = Camera::new(1920.0, 1080.0);
    cam.set_scale_instant(4.0);
    cam.set_offset_instant(Vec2::new(-200.0, -300.0));

    cam.reset();
    cam.apply_instant_target();

    assert_eq!(cam.scale(), 1.0);
    assert_eq!(cam.offset(), Vec2::new(0.0, 0.0));
}

#[test]
fn test_zoom_to_rect() {
    let mut cam = Camera::new(1000.0, 1000.0);
    let target_rect = Rect::new(200.0, 200.0, 200.0, 200.0);

    cam.zoom_to_rect(target_rect, 0.0); 
    cam.apply_instant_target();

    
    assert!((cam.scale() - 5.0).abs() < 1e-4);

    
    let screen_center = cam.world_to_screen(Vec2::new(300.0, 300.0));
    assert!((screen_center.x - 500.0).abs() < 1e-3);
    assert!((screen_center.y - 500.0).abs() < 1e-3);
}
