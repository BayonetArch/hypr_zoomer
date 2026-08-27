use hypr_zoomer::annotations::{Annotation, AnnotationManager, Color, Stroke, Arrow, RectAnnotation};
use hypr_zoomer::math::{Vec2, Rect};

#[test]
fn test_stroke_creation_and_points() {
    let mut stroke = Stroke::new(Color::RED, 4.0);
    stroke.add_point(Vec2::new(10.0, 10.0));
    stroke.add_point(Vec2::new(20.0, 20.0));
    stroke.add_point(Vec2::new(30.0, 40.0));

    assert_eq!(stroke.points.len(), 3);
    assert_eq!(stroke.color, Color::RED);
    assert_eq!(stroke.thickness, 4.0);
}

#[test]
fn test_arrow_creation() {
    let arrow = Arrow::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0), Color::GREEN, 3.0);
    assert_eq!(arrow.start, Vec2::new(0.0, 0.0));
    assert_eq!(arrow.end, Vec2::new(100.0, 100.0));
    assert_eq!(arrow.color, Color::GREEN);
}

#[test]
fn test_rect_annotation() {
    let rect_ann = RectAnnotation::new(Rect::new(10.0, 10.0, 50.0, 50.0), Color::BLUE, 2.0, false);
    assert_eq!(rect_ann.rect, Rect::new(10.0, 10.0, 50.0, 50.0));
    assert!(!rect_ann.filled);
}

#[test]
fn test_annotation_manager_undo_redo_clear() {
    let mut manager = AnnotationManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);

    let ann1 = Annotation::Stroke(Stroke::new(Color::RED, 2.0));
    let ann2 = Annotation::Arrow(Arrow::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0), Color::GREEN, 2.0));

    manager.push(ann1);
    manager.push(ann2);
    assert_eq!(manager.len(), 2);

    
    assert!(manager.undo());
    assert_eq!(manager.len(), 1);

    
    assert!(manager.redo());
    assert_eq!(manager.len(), 2);

    
    manager.undo();
    assert_eq!(manager.len(), 1);
    let ann3 = Annotation::Rect(RectAnnotation::new(Rect::new(0.0, 0.0, 10.0, 10.0), Color::YELLOW, 2.0, true));
    manager.push(ann3);
    assert_eq!(manager.len(), 2);
    assert!(!manager.redo()); 

    
    manager.clear();
    assert!(manager.is_empty());
}
