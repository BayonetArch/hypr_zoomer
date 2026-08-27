use crate::math::{Vec2, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const RED: Self = Self { r: 235, g: 50, b: 50, a: 255 };
    pub const GREEN: Self = Self { r: 50, g: 205, b: 50, a: 255 };
    pub const BLUE: Self = Self { r: 50, g: 120, b: 240, a: 255 };
    pub const YELLOW: Self = Self { r: 245, g: 215, b: 40, a: 255 };
    pub const MAGENTA: Self = Self { r: 220, g: 50, b: 220, a: 255 };
    pub const CYAN: Self = Self { r: 40, g: 220, b: 240, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub points: Vec<Vec2>,
    pub color: Color,
    pub thickness: f32,
}

impl Stroke {
    pub fn new(color: Color, thickness: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            thickness: thickness.max(1.0),
        }
    }

    pub fn add_point(&mut self, pt: Vec2) {
        if let Some(last) = self.points.last() {
            if last.distance_to(pt) < 1.0 {
                return; 
            }
        }
        self.points.push(pt);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arrow {
    pub start: Vec2,
    pub end: Vec2,
    pub color: Color,
    pub thickness: f32,
}

impl Arrow {
    pub fn new(start: Vec2, end: Vec2, color: Color, thickness: f32) -> Self {
        Self {
            start,
            end,
            color,
            thickness: thickness.max(1.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RectAnnotation {
    pub rect: Rect,
    pub color: Color,
    pub thickness: f32,
    pub filled: bool,
}

impl RectAnnotation {
    pub fn new(rect: Rect, color: Color, thickness: f32, filled: bool) -> Self {
        Self {
            rect,
            color,
            thickness: thickness.max(1.0),
            filled,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    Stroke(Stroke),
    Arrow(Arrow),
    Rect(RectAnnotation),
}

#[derive(Debug, Clone, Default)]
pub struct AnnotationManager {
    items: Vec<Annotation>,
    redo_stack: Vec<Annotation>,
}

impl AnnotationManager {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, item: Annotation) {
        self.items.push(item);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(item) = self.items.pop() {
            self.redo_stack.push(item);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(item) = self.redo_stack.pop() {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.redo_stack.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn items(&self) -> &[Annotation] {
        &self.items
    }
}
