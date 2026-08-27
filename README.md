# Hypr Zoomer

A screen magnifier and annotation tool for wayland compositors(hyprland), inspired by Tsoding's [boomer](https://github.com/tsoding/boomer).

## Features
- Physics-based zoom and panning with damped spring and friction interpolation. zoom keeps the point under the cursor stationary.
- Flashlight / spotlight effect that darkens everything except a configurable circle around the cursor.
- Annotation tools: freehand pen, arrow, and rectangle.
- Bilinear and nearest-neighbor scaling modes.
- Pixel grid overlay at high magnification, plus an on-screen readout of cursor coordinates and the color under the cursor.
- Copy the zoomed view to the clipboard or save it to a file.

# Quick start

1. build

```bash
 git clone https://github.com/BayonetArch/hypr_zoomer.git && cd hypr_zoomer && cargo b --release 

```
*the bin is at* `target/release/hypr_zoomer`

2. from crates.io:

```bash
 cargo install hypr_zoomer
```

3. github

```bash
 cargo install --git "https://github.com/BayonetArch/hypr_zoomer.git"
```

## Controls

| Action | Shortcut |
|---|---|
| Zoom in / out | Mouse wheel or `=` / `-` |
| Pan / drag | Left click + drag or middle click + drag |
| Freehand draw | Right click + drag or `d` |
| Draw arrow | `a` then left click + drag |
| Draw box | `b` then left click + drag |
| Zoom to selection | Shift + left click + drag |
| Reset zoom (1:1) | `0` |
| Toggle flashlight | `f` |
| Flashlight radius | Ctrl + mouse wheel or `[` / `]` |
| Toggle nearest / bilinear | Tab or `p` |
| Mirror image | `m` (horizontal) / Shift + `m` (vertical) |
| Invert colors | `i` |
| Toggle pixel grid | `g` |
| Toggle HUD | `h` |
| Undo / redo | `u` / Ctrl+Z / Ctrl+Y |
| Clear annotations | `c` |
| Switch colors | `1`-`5` (red, green, blue, yellow, magenta) |
| Copy color under cursor | `x` / `k` (hex), Shift+`x` (rgb), Ctrl+Shift+`x` (hsl) |
| Copy image to clipboard | `y` or Ctrl+C |
| Save image to file | `s` or Ctrl+S |
| Frame active window | `w` (hyprland only) |
| Quit | Esc or `q` |


## Configuration

generate a default config:

```bash
mkdir -p ~/.config/hypr_zoomer
./target/release/hypr_zoomer --generate-config > ~/.config/hypr_zoomer/config.toml
```

Example:

```toml
[general]
scroll_speed = 0.15
scale_friction = 0.85
drag_friction = 0.85
min_scale = 1.0
max_scale = 64.0
auto_track_active_window = false

[effects]
flashlight_radius = 150.0
flashlight_feather = 25.0
flashlight_ambient = 0.2
grid_min_scale = 8.0

[hud]
enabled = true
show_color_picker = true
show_coords = true

[render]
filter_mode = "bilinear"
target_fps = 120
```


## License

This project is licensed under the [MIT License](./LICENSE).
