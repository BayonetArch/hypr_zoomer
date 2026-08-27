use hypr_zoomer::math::{Vec2, Rect};

#[test]
fn test_vec2_basic_operations() {
    let a = Vec2::new(10.0, 20.0);
    let b = Vec2::new(5.0, 15.0);

    let sum = a + b;
    assert_eq!(sum, Vec2::new(15.0, 35.0));

    let diff = a - b;
    assert_eq!(diff, Vec2::new(5.0, 5.0));

    let scaled = a * 2.5;
    assert_eq!(scaled, Vec2::new(25.0, 50.0));

    let div = a / 2.0;
    assert_eq!(div, Vec2::new(5.0, 10.0));
}

#[test]
fn test_vec2_magnitude_and_distance() {
    let v = Vec2::new(3.0, 4.0);
    assert!((v.length() - 5.0).abs() < 1e-6);
    assert!((v.length_squared() - 25.0).abs() < 1e-6);

    let norm = v.normalize();
    assert!((norm.length() - 1.0).abs() < 1e-6);
    assert!((norm.x - 0.6).abs() < 1e-6);
    assert!((norm.y - 0.8).abs() < 1e-6);

    let p1 = Vec2::new(0.0, 0.0);
    let p2 = Vec2::new(6.0, 8.0);
    assert!((p1.distance_to(p2) - 10.0).abs() < 1e-6);
}

#[test]
fn test_vec2_lerp() {
    let a = Vec2::new(0.0, 100.0);
    let b = Vec2::new(100.0, 200.0);

    let mid = Vec2::lerp(a, b, 0.5);
    assert_eq!(mid, Vec2::new(50.0, 150.0));

    let start = Vec2::lerp(a, b, 0.0);
    assert_eq!(start, a);

    let end = Vec2::lerp(a, b, 1.0);
    assert_eq!(end, b);
}

#[test]
fn test_rect_bounds_and_contains() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(r.x, 10.0);
    assert_eq!(r.y, 20.0);
    assert_eq!(r.width, 100.0);
    assert_eq!(r.height, 50.0);
    assert_eq!(r.right(), 110.0);
    assert_eq!(r.bottom(), 70.0);
    assert_eq!(r.center(), Vec2::new(60.0, 45.0));

    assert!(r.contains_point(Vec2::new(10.0, 20.0)));
    assert!(r.contains_point(Vec2::new(50.0, 40.0)));
    assert!(r.contains_point(Vec2::new(110.0, 70.0)));
    assert!(!r.contains_point(Vec2::new(9.9, 20.0)));
    assert!(!r.contains_point(Vec2::new(50.0, 70.1)));
}

#[test]
fn test_rect_from_points() {
    let p1 = Vec2::new(100.0, 200.0);
    let p2 = Vec2::new(50.0, 100.0);
    let r = Rect::from_points(p1, p2);

    assert_eq!(r.x, 50.0);
    assert_eq!(r.y, 100.0);
    assert_eq!(r.width, 50.0);
    assert_eq!(r.height, 100.0);
}

#[test]
fn test_rect_intersection_and_clamp() {
    let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
    let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);

    assert!(r1.intersects(&r2));
    let inter = r1.intersection(&r2).unwrap();
    assert_eq!(inter, Rect::new(50.0, 50.0, 50.0, 50.0));

    let r3 = Rect::new(200.0, 200.0, 50.0, 50.0);
    assert!(!r1.intersects(&r3));
    assert!(r1.intersection(&r3).is_none());

    let clamped = r1.clamp_point(Vec2::new(-10.0, 150.0));
    assert_eq!(clamped, Vec2::new(0.0, 100.0));
}
