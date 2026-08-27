/*
 * hypr_zoomer
 * a zooming tool for wayland
 * Copyright (c) 2026 BayonetArch
 *
 * This software is released under the MIT License.
 * See LICENSE file for details.
 */
use crate::annotations::{Annotation, AnnotationManager, Arrow, Color, RectAnnotation, Stroke};
use crate::camera::Camera;
use crate::capture::{ScreenImage, capture_screen};
use crate::config::{Config, FilterMode};
use crate::effects::{Flashlight, format_hex_color, format_hsl_color};
use crate::export::{copy_text_to_clipboard, copy_to_clipboard, save_to_png};
use crate::hyprland::{get_active_window, get_cursor_position};
use crate::math::{Rect, Vec2};
use crate::render::{Framebuffer, Renderer};

use anyhow::Result;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Pan,
    Brush,
    Arrow,
    Box,
    SelectCrop,
}

pub struct App {
    config: Config,
    image: ScreenImage,
    camera: Camera,
    renderer: Renderer,
    framebuffer: Framebuffer,
    annotations: AnnotationManager,

    current_tool: ToolMode,
    active_color: Color,
    brush_thickness: f32,

    flashlight: Flashlight,
    flashlight_enabled: bool,
    filter_mode: FilterMode,
    invert_colors: bool,
    mirror_h: bool,
    mirror_v: bool,
    pixel_grid_enabled: bool,
    hud_enabled: bool,

    cursor_pos: Vec2,
    modifiers: ModifiersState,
    is_left_down: bool,
    is_right_down: bool,
    is_middle_down: bool,

    active_stroke: Option<Stroke>,
    arrow_start: Option<Vec2>,
    box_start: Option<Vec2>,

    status_message: Option<(String, Instant)>,
    help_visible: bool,

    last_frame_time: Instant,
    needs_redraw: bool,

    window: Option<Rc<Window>>,
    _graphics_context: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
}

impl App {
    pub fn new(
        config: Config,
        initial_scale: Option<f32>,
        geometry: Option<String>,
    ) -> Result<Self> {
        let image = capture_screen(geometry.as_deref(), None, false)
            .map_err(|e| anyhow::anyhow!("Screen capture failed: {}\n\nMake sure 'grim' is installed and you're running on a Wayland session.", e))?;

        let mut camera = Camera::new(image.width as f32, image.height as f32);
        camera.set_scale_friction(config.general.scale_friction);
        camera.set_drag_friction(config.general.drag_friction);
        camera.set_min_scale(config.general.min_scale);
        camera.set_max_scale(config.general.max_scale);

        if let Some(scale) = initial_scale {
            let cursor = get_cursor_position().unwrap_or(Vec2::new(
                image.width as f32 * 0.5,
                image.height as f32 * 0.5,
            ));
            camera.zoom_at(cursor, scale);
            camera.apply_instant_target();
        } else if config.general.auto_track_active_window {
            if let Some(win) = get_active_window() {
                camera.zoom_to_rect(win.to_rect(), 40.0);
            }
        }

        let framebuffer = Framebuffer::new(image.width, image.height);
        let flashlight = Flashlight::new(
            config.effects.flashlight_radius,
            config.effects.flashlight_feather,
            config.effects.flashlight_ambient,
        );

        Ok(Self {
            config: config.clone(),
            image,
            camera,
            renderer: Renderer::new(),
            framebuffer,
            annotations: AnnotationManager::new(),
            current_tool: ToolMode::Pan,
            active_color: Color::RED,
            brush_thickness: 3.5,
            flashlight,
            flashlight_enabled: false,
            filter_mode: config.render.filter_mode,
            invert_colors: false,
            mirror_h: false,
            mirror_v: false,
            pixel_grid_enabled: true,
            hud_enabled: config.hud.enabled,
            cursor_pos: Vec2::ZERO,
            modifiers: ModifiersState::default(),
            is_left_down: false,
            is_right_down: false,
            is_middle_down: false,
            active_stroke: None,
            arrow_start: None,
            box_start: None,
            status_message: None,
            help_visible: false,
            last_frame_time: Instant::now(),
            needs_redraw: true,
            window: None,
            _graphics_context: None,
            surface: None,
        })
    }

    pub fn show_status(&mut self, text: &str) {
        self.status_message = Some((text.to_string(), Instant::now()));
        self.needs_redraw = true;
    }

    pub fn render_frame(&mut self) {
        let dt = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();

        self.camera.update(dt);

        self.renderer.blit_image(
            &self.image,
            &mut self.framebuffer,
            &self.camera,
            self.filter_mode,
            self.invert_colors,
            self.mirror_h,
            self.mirror_v,
        );

        if self.pixel_grid_enabled {
            self.renderer
                .draw_pixel_grid(&mut self.framebuffer, &self.camera, &self.image);
        }

        self.renderer.render_annotations(
            &mut self.framebuffer,
            &self.camera,
            self.annotations.items(),
        );

        if let Some(ref stroke) = self.active_stroke {
            self.renderer.render_annotations(
                &mut self.framebuffer,
                &self.camera,
                &[Annotation::Stroke(stroke.clone())],
            );
        }

        if let Some(start) = self.arrow_start {
            let arrow = Arrow::new(
                start,
                self.camera.screen_to_world(self.cursor_pos),
                self.active_color,
                self.brush_thickness,
            );
            self.renderer.render_annotations(
                &mut self.framebuffer,
                &self.camera,
                &[Annotation::Arrow(arrow)],
            );
        }

        if let Some(start) = self.box_start {
            let world_curr = self.camera.screen_to_world(self.cursor_pos);
            let rect = Rect::from_points(start, world_curr);
            let box_ann = RectAnnotation::new(rect, self.active_color, self.brush_thickness, false);
            self.renderer.render_annotations(
                &mut self.framebuffer,
                &self.camera,
                &[Annotation::Rect(box_ann)],
            );
        }

        if self.flashlight_enabled {
            
            self.renderer.apply_flashlight(
                &mut self.framebuffer,
                self.cursor_pos,
                &self.flashlight,
            );
        }

        if self.hud_enabled {
            self.draw_hud_overlay();
        }

        if self.help_visible {
            draw_help_overlay(&mut self.framebuffer);
        }

        if let Some(ref mut surface) = self.surface {
            if let Ok(mut buffer) = surface.buffer_mut() {
                self.framebuffer.copy_to_softbuffer(&mut buffer);
                let _ = buffer.present();
            }
        }
    }

    pub fn get_pixel_under_cursor(&self) -> (u32, u32, [u8; 4]) {
        let world_cursor = self.camera.screen_to_world(self.cursor_pos);
        let mut wx = world_cursor.x.floor();
        let mut wy = world_cursor.y.floor();

        if self.mirror_h {
            wx = (self.image.width as f32 - 1.0) - wx;
        }
        if self.mirror_v {
            wy = (self.image.height as f32 - 1.0) - wy;
        }

        let clamped_x = wx.clamp(0.0, self.image.width as f32 - 1.0) as u32;
        let clamped_y = wy.clamp(0.0, self.image.height as f32 - 1.0) as u32;

        let mut pixel = self
            .image
            .get_pixel(clamped_x, clamped_y)
            .unwrap_or([0, 0, 0, 255]);
        if self.invert_colors {
            pixel = crate::effects::invert_color(pixel);
        }

        (clamped_x, clamped_y, pixel)
    }

    fn draw_hud_overlay(&mut self) {
        let scale_percent = (self.camera.scale() * 100.0).round() as i32;
        let (wx, wy, pixel_under_cursor) = self.get_pixel_under_cursor();

        let hex = format_hex_color(pixel_under_cursor);
        let filter_str = match self.filter_mode {
            FilterMode::Nearest => "Nearest (Crisp)",
            FilterMode::Bilinear => "Bilinear (Smooth)",
        };

        let mut lines = Vec::new();
        lines.push(format!(
            "Zoom: {}% ({:.1}x)",
            scale_percent,
            self.camera.scale()
        ));
        lines.push(format!("Pos: X: {} Y: {}", wx, wy));
        lines.push(format!(
            "Color: {} [x:hex, Shift+x:rgb, Ctrl+Shift+x:hsl]",
            hex
        ));
        lines.push(format!("Filter: {} [Tab/p]", filter_str));
        if self.flashlight_enabled {
            lines.push(format!(
                "Flashlight: ON (R: {:.0}px) [f / [ ]]",
                self.flashlight.radius
            ));
        }

        if let Some((ref msg, time)) = self.status_message {
            if time.elapsed().as_secs_f32() < 3.0 {
                lines.push(format!("> {}", msg));
            }
        }

        draw_hud_panel(&mut self.framebuffer, 16, 16, &lines, pixel_under_cursor);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("Hypr Zoomer")
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .with_decorations(false)
            .with_transparent(false);

        let window = match event_loop.create_window(window_attrs) {
            Ok(w) => {
                w.set_cursor(winit::window::CursorIcon::Crosshair);
                Rc::new(w)
            }
            Err(e) => {
                eprintln!("Failed to create window: {:?}", e);
                event_loop.exit();
                return;
            }
        };

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        self.camera.set_viewport_size(width as f32, height as f32);
        self.framebuffer.resize(width, height);

        let context =
            softbuffer::Context::new(window.clone()).expect("Failed to create softbuffer context");
        let mut surface = softbuffer::Surface::new(&context, window.clone())
            .expect("Failed to create softbuffer surface");

        let _ = surface.resize(
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        self.window = Some(window);
        self._graphics_context = Some(context);
        self.surface = Some(surface);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                let width = size.width.max(1);
                let height = size.height.max(1);
                self.camera.set_viewport_size(width as f32, height as f32);
                self.framebuffer.resize(width, height);
                if let Some(ref mut surface) = self.surface {
                    let _ = surface.resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    );
                }
                self.needs_redraw = true;
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                self.modifiers = new_mods.state();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = Vec2::new(position.x as f32, position.y as f32);
                self.needs_redraw = true;

                if self.is_left_down {
                    if self.current_tool == ToolMode::Pan {
                        self.camera.drag_update(self.cursor_pos);
                    } else if self.current_tool == ToolMode::Brush {
                        let world_pos = self.camera.screen_to_world(self.cursor_pos);
                        if let Some(ref mut stroke) = self.active_stroke {
                            stroke.add_point(world_pos);
                        }
                    }
                }

                if self.is_right_down {
                    let world_pos = self.camera.screen_to_world(self.cursor_pos);
                    if let Some(ref mut stroke) = self.active_stroke {
                        stroke.add_point(world_pos);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.needs_redraw = true;
                let is_pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Left => {
                        self.is_left_down = is_pressed;
                        if is_pressed {
                            if self.modifiers.shift_key()
                                || self.current_tool == ToolMode::SelectCrop
                            {
                                self.box_start = Some(self.camera.screen_to_world(self.cursor_pos));
                            } else if self.current_tool == ToolMode::Arrow {
                                self.arrow_start =
                                    Some(self.camera.screen_to_world(self.cursor_pos));
                            } else if self.current_tool == ToolMode::Box {
                                self.box_start = Some(self.camera.screen_to_world(self.cursor_pos));
                            } else if self.current_tool == ToolMode::Brush {
                                let world_pos = self.camera.screen_to_world(self.cursor_pos);
                                let mut stroke =
                                    Stroke::new(self.active_color, self.brush_thickness);
                                stroke.add_point(world_pos);
                                self.active_stroke = Some(stroke);
                            } else {
                                self.camera.drag_start(self.cursor_pos);
                            }
                        } else {
                            if self.current_tool == ToolMode::Brush {
                                if let Some(stroke) = self.active_stroke.take() {
                                    if stroke.points.len() >= 2 {
                                        self.annotations.push(Annotation::Stroke(stroke));
                                    }
                                }
                            } else if let Some(start) = self.box_start.take() {
                                let curr = self.camera.screen_to_world(self.cursor_pos);
                                let rect = Rect::from_points(start, curr);
                                if self.modifiers.shift_key() {
                                    self.camera.zoom_to_rect(rect, 40.0);
                                } else {
                                    self.annotations.push(Annotation::Rect(RectAnnotation::new(
                                        rect,
                                        self.active_color,
                                        self.brush_thickness,
                                        false,
                                    )));
                                }
                            } else if let Some(start) = self.arrow_start.take() {
                                let curr = self.camera.screen_to_world(self.cursor_pos);
                                self.annotations.push(Annotation::Arrow(Arrow::new(
                                    start,
                                    curr,
                                    self.active_color,
                                    self.brush_thickness,
                                )));
                            } else {
                                self.camera.drag_end();
                            }
                        }
                    }
                    MouseButton::Right => {
                        self.is_right_down = is_pressed;
                        if is_pressed {
                            let world_pos = self.camera.screen_to_world(self.cursor_pos);
                            let mut stroke = Stroke::new(self.active_color, self.brush_thickness);
                            stroke.add_point(world_pos);
                            self.active_stroke = Some(stroke);
                        } else {
                            if let Some(stroke) = self.active_stroke.take() {
                                if stroke.points.len() >= 2 {
                                    self.annotations.push(Annotation::Stroke(stroke));
                                }
                            }
                        }
                    }
                    MouseButton::Middle => {
                        self.is_middle_down = is_pressed;
                        if is_pressed {
                            self.camera.drag_start(self.cursor_pos);
                        } else {
                            self.camera.drag_end();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.needs_redraw = true;
                let scroll_val = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 20.0) as f32,
                };

                if self.modifiers.control_key() {
                    self.flashlight.radius =
                        (self.flashlight.radius + scroll_val * 20.0).clamp(30.0, 800.0);
                } else {
                    let speed = self.config.general.scroll_speed;
                    let factor = if scroll_val > 0.0 {
                        1.0 + speed * scroll_val.abs().min(3.0)
                    } else {
                        1.0 / (1.0 + speed * scroll_val.abs().min(3.0))
                    };
                    self.camera.zoom_at(self.cursor_pos, factor);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                match key_code {
                    KeyCode::Slash if self.modifiers.shift_key() => {
                        self.help_visible = !self.help_visible;
                        self.needs_redraw = true;
                    }
                    KeyCode::Escape | KeyCode::KeyQ => {
                        if self.help_visible {
                            self.help_visible = false;
                        } else {
                            event_loop.exit();
                        }
                    }
                    KeyCode::Digit0 | KeyCode::Numpad0 => {
                        self.camera.reset();
                        self.show_status("Reset zoom (1:1)");
                    }
                    KeyCode::Equal | KeyCode::NumpadAdd => {
                        self.camera.zoom_at(self.cursor_pos, 1.25);
                    }
                    KeyCode::Minus | KeyCode::NumpadSubtract => {
                        self.camera.zoom_at(self.cursor_pos, 0.8);
                    }
                    KeyCode::KeyF => {
                        self.flashlight_enabled = !self.flashlight_enabled;
                        let state = if self.flashlight_enabled { "ON" } else { "OFF" };
                        self.show_status(&format!("Flashlight: {}", state));
                    }
                    KeyCode::BracketLeft => {
                        self.flashlight.radius = (self.flashlight.radius - 20.0).max(30.0);
                    }
                    KeyCode::BracketRight => {
                        self.flashlight.radius = (self.flashlight.radius + 20.0).min(1000.0);
                    }
                    KeyCode::KeyM => {
                        if self.modifiers.shift_key() {
                            self.mirror_v = !self.mirror_v;
                            self.show_status("Toggled vertical mirror");
                        } else {
                            self.mirror_h = !self.mirror_h;
                            self.show_status("Toggled horizontal mirror");
                        }
                    }
                    KeyCode::KeyI => {
                        self.invert_colors = !self.invert_colors;
                        self.show_status("Toggled color inversion");
                    }
                    KeyCode::Tab | KeyCode::KeyP => {
                        self.filter_mode = match self.filter_mode {
                            FilterMode::Bilinear => FilterMode::Nearest,
                            FilterMode::Nearest => FilterMode::Bilinear,
                        };
                        self.show_status(&format!("Filter: {:?}", self.filter_mode));
                    }
                    KeyCode::KeyG => {
                        self.pixel_grid_enabled = !self.pixel_grid_enabled;
                        self.show_status(&format!(
                            "Pixel Grid: {}",
                            if self.pixel_grid_enabled { "ON" } else { "OFF" }
                        ));
                    }
                    KeyCode::KeyH => {
                        self.hud_enabled = !self.hud_enabled;
                    }
                    KeyCode::KeyD => {
                        self.current_tool = ToolMode::Brush;
                        self.show_status("Tool: Brush [Draw]");
                    }
                    KeyCode::KeyA => {
                        self.current_tool = ToolMode::Arrow;
                        self.show_status("Tool: Arrow");
                    }
                    KeyCode::KeyB => {
                        self.current_tool = ToolMode::Box;
                        self.show_status("Tool: Rectangle Box");
                    }
                    KeyCode::KeyU => {
                        if self.annotations.undo() {
                            self.show_status("Undid annotation");
                        }
                    }
                    KeyCode::KeyZ => {
                        if self.modifiers.control_key() {
                            if self.modifiers.shift_key() {
                                self.annotations.redo();
                            } else {
                                self.annotations.undo();
                            }
                        }
                    }
                    KeyCode::KeyC => {
                        if self.modifiers.control_key() {
                            if let Ok(()) = copy_to_clipboard(&self.image) {
                                self.show_status("Copied screenshot to clipboard!");
                            }
                        } else {
                            self.annotations.clear();
                            self.show_status("Cleared annotations");
                        }
                    }
                    KeyCode::KeyX | KeyCode::KeyK => {
                        let (_wx, _wy, pixel) = self.get_pixel_under_cursor();
                        if self.modifiers.control_key() && self.modifiers.shift_key() {
                            let hsl_str = format_hsl_color(pixel);
                            if let Ok(()) = copy_text_to_clipboard(&hsl_str) {
                                self.show_status(&format!("Copied {} to clipboard!", hsl_str));
                            }
                        } else if self.modifiers.shift_key() {
                            let rgb_str = format!("rgb({}, {}, {})", pixel[0], pixel[1], pixel[2]);
                            if let Ok(()) = copy_text_to_clipboard(&rgb_str) {
                                self.show_status(&format!("Copied {} to clipboard!", rgb_str));
                            }
                        } else {
                            let hex_str = format_hex_color(pixel);
                            if let Ok(()) = copy_text_to_clipboard(&hex_str) {
                                self.show_status(&format!("Copied {} to clipboard!", hex_str));
                            }
                        }
                    }
                    KeyCode::KeyY => {
                        if let Ok(()) = copy_to_clipboard(&self.image) {
                            self.show_status("Copied screenshot to clipboard!");
                        }
                    }
                    KeyCode::KeyS => {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                        let path = format!("{}/Pictures/Screenshots/zoom_{}.png", home, timestamp);
                        if let Ok(()) = save_to_png(&self.image, &path) {
                            self.show_status(&format!("Saved to {}", path));
                        }
                    }
                    KeyCode::KeyW => {
                        if let Some(win) = get_active_window() {
                            self.camera.zoom_to_rect(win.to_rect(), 40.0);
                            self.show_status(&format!("Focused window: {}", win.class));
                        }
                    }
                    KeyCode::Digit1 => {
                        self.active_color = Color::RED;
                        self.show_status("Color: Red");
                    }
                    KeyCode::Digit2 => {
                        self.active_color = Color::GREEN;
                        self.show_status("Color: Green");
                    }
                    KeyCode::Digit3 => {
                        self.active_color = Color::BLUE;
                        self.show_status("Color: Blue");
                    }
                    KeyCode::Digit4 => {
                        self.active_color = Color::YELLOW;
                        self.show_status("Color: Yellow");
                    }
                    KeyCode::Digit5 => {
                        self.active_color = Color::MAGENTA;
                        self.show_status("Color: Magenta");
                    }
                    _ => {}
                }
                self.needs_redraw = true;
            }
            WindowEvent::RedrawRequested => {
                self.needs_redraw = false;
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.camera.is_animating() {
            self.needs_redraw = true;
        }

        if let Some((_, time)) = self.status_message {
            if time.elapsed().as_secs_f32() < 3.0 {
                self.needs_redraw = true;
            }
        }

        if self.needs_redraw {
            if let Some(ref window) = self.window {
                window.request_redraw();
            }
            let target_fps = self.config.render.target_fps.clamp(30, 240);
            let frame_dur = std::time::Duration::from_nanos(1_000_000_000 / target_fps as u64);
            event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + frame_dur));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

pub fn run(config: Config, initial_scale: Option<f32>, geometry: Option<String>) -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(config, initial_scale, geometry)?;
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn draw_help_overlay(fb: &mut Framebuffer) {
    let entries: &[(&str, &str)] = &[
        ("?",             "Toggle help"),
        ("Scroll / = / -","Zoom in / out"),
        ("0",             "Reset zoom"),
        ("Drag L/M btn",  "Pan"),
        ("d",             "Freehand brush"),
        ("a",             "Arrow"),
        ("b",             "Box"),
        ("Shift+drag",    "Zoom selection"),
        ("f",             "Flashlight"),
        ("[ / ]",         "Flashlight radius"),
        ("Ctrl+scroll",   "Flashlight radius"),
        ("i",             "Invert colors"),
        ("m / Shift+m",   "Mirror H / V"),
        ("Tab / p",       "Filter mode"),
        ("g",             "Pixel grid"),
        ("h",             "Toggle HUD"),
        ("u / Ctrl+Z",    "Undo"),
        ("Ctrl+Shift+Z",  "Redo"),
        ("c",             "Clear annots"),
        ("1-5",           "Colors r/g/b/y/m"),
        ("x / k",         "Copy hex"),
        ("Shift+x",       "Copy rgb"),
        ("Ctrl+Shift+x",  "Copy hsl"),
        ("y / Ctrl+C",    "Copy to clipboard"),
        ("s",             "Save screenshot"),
        ("w",             "Frame window"),
        ("Esc / q",       "Quit"),
    ];

    let cols: u32 = 2;
    let rows_per_col = (entries.len() as u32 + cols - 1) / cols;
    let title = "Keyboard Shortcuts   (? to close)";

    
    
    let measure = |size: f32| -> (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32) {
        let title_h    = size * 1.25;
        let line_h     = size * 1.4;
        let max_key_w  = entries.iter().map(|(k, _)| text_width(k, size)).fold(0.0, f32::max);
        let max_desc_w = entries.iter().map(|(_, d)| text_width(d, size)).fold(0.0, f32::max);
        let title_w    = text_width(title, size);
        let key_col_w  = max_key_w + 24.0;
        let col_w      = key_col_w + max_desc_w;
        let col_gap    = 44.0;
        let pad_x      = 30.0;
        let pad_y      = 22.0;
        let content_w  = (cols as f32 * col_w + (cols as f32 - 1.0) * col_gap).max(title_w);
        let panel_w    = content_w + pad_x * 2.0;
        let panel_h    = title_h + line_h + 10.0 + rows_per_col as f32 * line_h + pad_y * 2.0;
        (size, title_h, line_h, key_col_w, col_w, col_gap, pad_x, pad_y, title_w, content_w, panel_w, panel_h)
    };

    let candidates = [28.0f32, 24.0, 20.0, 16.0, 13.0];
    let mut chosen = measure(13.0);
    for &s in &candidates {
        let m = measure(s);
        if m.10 <= fb.width as f32 && m.11 <= fb.height as f32 {
            chosen = m;
            break;
        }
    }
    let (size, title_h, line_h, key_col_w, col_w, col_gap, pad_x, pad_y, _title_w, content_w, panel_w, panel_h) = chosen;

    let cx = fb.width as f32 / 2.0;
    let cy = fb.height as f32 / 2.0;
    let x0 = (cx - panel_w / 2.0).round().max(0.0) as i32;
    let y0 = (cy - panel_h / 2.0).round().max(0.0) as i32;
    let x1 = ((x0 as f32 + panel_w).round() as i32).min(fb.width as i32 - 1);
    let y1 = ((y0 as f32 + panel_h).round() as i32).min(fb.height as i32 - 1);

    let bg     = [10, 12, 20, 235];
    let border = [80, 105, 165, 255];
    let title_color = [230, 235, 255, 255];
    let key_color   = [130, 195, 255, 255];
    let desc_color  = [195, 210, 230, 255];
    let sep_color   = [55, 65, 95, 255];

    
    for y in y0..=y1 {
        for x in x0..=x1 {
            let on_border = x == x0 || x == x1 || y == y0 || y == y1
                         || x == x0 + 1 || x == x1 - 1 || y == y0 + 1 || y == y1 - 1;
            crate::render::blend_pixel_pub(fb, x as u32, y as u32, if on_border { border } else { bg });
        }
    }

    
    draw_text(fb, x0 + pad_x as i32, y0 + pad_y as i32, title, size, title_color);

    
    let sep_y = (y0 as f32 + pad_y + title_h + 4.0).round() as i32;
    for x in (x0 + pad_x as i32)..(x1 - pad_x as i32) {
        crate::render::blend_pixel_pub(fb, x as u32, sep_y as u32, sep_color);
    }

    
    let content_x0 = x0 as f32 + pad_x + (content_w - (cols as f32 * col_w + (cols as f32 - 1.0) * col_gap)) / 2.0;
    for (i, (key, desc)) in entries.iter().enumerate() {
        let col = (i as u32) / rows_per_col;
        let row = (i as u32) % rows_per_col;
        let tx  = (content_x0 + col as f32 * (col_w + col_gap)).round() as i32;
        let ty  = (y0 as f32 + pad_y + title_h + 10.0 + row as f32 * line_h).round() as i32;
        draw_text(fb, tx, ty, key, size, key_color);
        draw_text(fb, (tx as f32 + key_col_w).round() as i32, ty, desc, size, desc_color);
    }
}

fn draw_hud_panel(
    fb: &mut Framebuffer,
    start_x: u32,
    start_y: u32,
    lines: &[String],
    color_swatch: [u8; 4],
) {
    let size: f32 = 16.0;
    let padding_x = 16u32;
    let padding_y = 12u32;
    let line_height = (size * 1.4) as u32;
    let swatch_size = 22u32;

    let max_text_width = lines
        .iter()
        .map(|l| text_width(l, size) as u32)
        .max()
        .unwrap_or(200);
    let width = (max_text_width + padding_x * 2 + swatch_size + 16).max(340);
    let height = (lines.len() as u32 * line_height) + padding_y * 2 + 4;

    let bg_color = [15, 18, 25, 220];
    let border_color = [70, 85, 110, 245];

    let end_x = (start_x + width).min(fb.width.saturating_sub(1));
    let end_y = (start_y + height).min(fb.height.saturating_sub(1));

    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let is_border = x == start_x || x == end_x || y == start_y || y == end_y;
            let c = if is_border { border_color } else { bg_color };
            crate::render::blend_pixel_pub(fb, x, y, c);
        }
    }

    let swatch_x = start_x + width - padding_x - swatch_size;
    let swatch_y = start_y + padding_y;
    for y in swatch_y..swatch_y + swatch_size {
        for x in swatch_x..swatch_x + swatch_size {
            if x < fb.width && y < fb.height {
                let is_border = x == swatch_x
                    || x == swatch_x + swatch_size - 1
                    || y == swatch_y
                    || y == swatch_y + swatch_size - 1;
                let c = if is_border {
                    [255, 255, 255, 255]
                } else {
                    color_swatch
                };
                crate::render::blend_pixel_pub(fb, x, y, c);
            }
        }
    }

    for (i, line) in lines.iter().enumerate() {
        let text_y = (start_y + padding_y + (i as u32 * line_height)) as i32;
        draw_text(fb, (start_x + padding_x) as i32, text_y, line, size, [235, 240, 250, 255]);
    }
}




fn get_font() -> &'static fontdue::Font {
    static FONT: OnceLock<fontdue::Font> = OnceLock::new();
    FONT.get_or_init(|| {
        let data = include_bytes!("../assets/LiberationSans-Regular.ttf");
        fontdue::Font::from_bytes(data.as_slice(), fontdue::FontSettings::default())
            .expect("failed to load embedded font")
    })
}


fn text_width(text: &str, size: f32) -> f32 {
    let font = get_font();
    text.chars()
        .map(|ch| font.metrics(ch, size).advance_width)
        .sum()
}




fn draw_text(fb: &mut Framebuffer, x: i32, y: i32, text: &str, size: f32, color: [u8; 4]) {
    let font = get_font();
    let ascent = font
        .vertical_line_metrics(size)
        .map(|m| m.ascent)
        .unwrap_or(size * 0.8);
    let baseline = y as f32 + ascent;
    let mut pen_x = x as f32;

    for ch in text.chars() {
        let (metrics, data) = font.rasterize(ch, size);
        let w = metrics.width as usize;
        let h = metrics.height as usize;
        if w > 0 && h > 0 {
            let off_x = (pen_x + metrics.xmin as f32) as i32;
            let off_y = (baseline - metrics.ymin as f32 - metrics.height as f32) as i32;
            for row in 0..h {
                let sy = off_y + row as i32;
                if sy < 0 || sy as u32 >= fb.height {
                    continue;
                }
                let base = row * w;
                for col in 0..w {
                    let a = data[base + col];
                    if a == 0 {
                        continue;
                    }
                    let sx = off_x + col as i32;
                    if sx < 0 || sx as u32 >= fb.width {
                        continue;
                    }
                    crate::render::blend_pixel_pub(
                        fb,
                        sx as u32,
                        sy as u32,
                        [color[0], color[1], color[2], a],
                    );
                }
            }
        }
        pen_x += metrics.advance_width;
    }
}
