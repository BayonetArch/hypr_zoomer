use crate::math::{Vec2, Rect};

#[derive(Debug, Clone)]
pub struct Camera {
    viewport_width: f32,
    viewport_height: f32,

    scale: f32,
    target_scale: f32,
    min_scale: f32,
    max_scale: f32,

    offset: Vec2,
    target_offset: Vec2,

    scale_friction: f32,
    drag_friction: f32,

    is_dragging: bool,
    drag_start_cursor: Vec2,
    drag_start_offset: Vec2,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            scale: 1.0,
            target_scale: 1.0,
            min_scale: 1.0,
            max_scale: 64.0,
            offset: Vec2::ZERO,
            target_offset: Vec2::ZERO,
            scale_friction: 0.85,
            drag_friction: 0.85,
            is_dragging: false,
            drag_start_cursor: Vec2::ZERO,
            drag_start_offset: Vec2::ZERO,
        }
    }

    #[inline]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    #[inline]
    pub fn target_scale(&self) -> f32 {
        self.target_scale
    }

    #[inline]
    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    #[inline]
    pub fn target_offset(&self) -> Vec2 {
        self.target_offset
    }

    #[inline]
    pub fn is_animating(&self) -> bool {
        (self.scale - self.target_scale).abs() > 1e-4
            || (self.offset - self.target_offset).length_squared() > 1e-2
    }

    #[inline]
    pub fn viewport_size(&self) -> Vec2 {
        Vec2::new(self.viewport_width, self.viewport_height)
    }

    #[inline]
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    #[inline]
    pub fn set_scale_friction(&mut self, friction: f32) {
        self.scale_friction = friction.clamp(0.0, 0.99);
    }

    #[inline]
    pub fn set_drag_friction(&mut self, friction: f32) {
        self.drag_friction = friction.clamp(0.0, 0.99);
    }

    #[inline]
    pub fn set_min_scale(&mut self, min: f32) {
        self.min_scale = min.max(0.1);
    }

    #[inline]
    pub fn set_max_scale(&mut self, max: f32) {
        self.max_scale = max.max(self.min_scale);
    }

    #[inline]
    pub fn set_scale_instant(&mut self, scale: f32) {
        let clamped = scale.clamp(self.min_scale, self.max_scale);
        self.scale = clamped;
        self.target_scale = clamped;
    }

    #[inline]
    pub fn set_offset_instant(&mut self, offset: Vec2) {
        self.offset = offset;
        self.target_offset = offset;
    }

    #[inline]
    pub fn apply_instant_target(&mut self) {
        self.scale = self.target_scale;
        self.offset = self.target_offset;
    }

    #[inline]
    pub fn screen_to_world(&self, screen_pt: Vec2) -> Vec2 {
        (screen_pt - self.offset) / self.scale
    }

    #[inline]
    pub fn world_to_screen(&self, world_pt: Vec2) -> Vec2 {
        world_pt * self.scale + self.offset
    }

    pub fn zoom_at(&mut self, pivot: Vec2, factor: f32) {
        let new_target_scale = (self.target_scale * factor).clamp(self.min_scale, self.max_scale);
        if (new_target_scale - self.target_scale).abs() < 1e-6 {
            return;
        }

        
        let scale_ratio = new_target_scale / self.target_scale;
        self.target_offset = pivot - (pivot - self.target_offset) * scale_ratio;
        self.target_scale = new_target_scale;
    }

    pub fn drag_start(&mut self, cursor_pos: Vec2) {
        self.is_dragging = true;
        self.drag_start_cursor = cursor_pos;
        self.drag_start_offset = self.target_offset;
    }

    pub fn drag_update(&mut self, cursor_pos: Vec2) {
        if !self.is_dragging {
            return;
        }
        let delta = cursor_pos - self.drag_start_cursor;
        self.target_offset = self.drag_start_offset + delta;
        self.offset = self.target_offset;
    }

    pub fn drag_end(&mut self) {
        self.is_dragging = false;
    }

    #[inline]
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    pub fn pan_by(&mut self, delta: Vec2) {
        self.target_offset = self.target_offset + delta;
        self.offset = self.target_offset;
    }

    pub fn reset(&mut self) {
        self.target_scale = 1.0;
        self.target_offset = Vec2::ZERO;
        self.is_dragging = false;
    }

    pub fn zoom_to_rect(&mut self, world_rect: Rect, padding_px: f32) {
        if world_rect.width <= 0.0 || world_rect.height <= 0.0 {
            return;
        }

        let avail_w = (self.viewport_width - padding_px * 2.0).max(1.0);
        let avail_h = (self.viewport_height - padding_px * 2.0).max(1.0);

        let scale_x = avail_w / world_rect.width;
        let scale_y = avail_h / world_rect.height;
        let target_scale = scale_x.min(scale_y).clamp(self.min_scale, self.max_scale);

        let world_center = world_rect.center();
        let screen_center = Vec2::new(self.viewport_width * 0.5, self.viewport_height * 0.5);

        
        let target_offset = screen_center - world_center * target_scale;

        self.target_scale = target_scale;
        self.target_offset = target_offset;
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_dragging {
            return;
        }

        
        let dt_clamped = dt.clamp(0.001, 0.1);
        let scale_blend = 1.0 - self.scale_friction.powf(dt_clamped * 60.0);
        let drag_blend = 1.0 - self.drag_friction.powf(dt_clamped * 60.0);

        self.scale += (self.target_scale - self.scale) * scale_blend;
        self.offset = self.offset + (self.target_offset - self.offset) * drag_blend;
    }
}
