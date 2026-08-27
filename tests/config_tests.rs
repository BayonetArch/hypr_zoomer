use hypr_zoomer::config::{Config, FilterMode};

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert_eq!(cfg.general.scroll_speed, 0.15);
    assert_eq!(cfg.general.scale_friction, 0.85);
    assert_eq!(cfg.general.drag_friction, 0.85);
    assert_eq!(cfg.general.min_scale, 1.0);
    assert_eq!(cfg.general.max_scale, 64.0);
    assert_eq!(cfg.effects.flashlight_radius, 150.0);
    assert_eq!(cfg.effects.flashlight_feather, 25.0);
    assert!(cfg.hud.enabled);
    assert_eq!(cfg.render.filter_mode, FilterMode::Bilinear);
}

#[test]
fn test_toml_serialize_deserialize() {
    let mut cfg = Config::default();
    cfg.general.scroll_speed = 0.25;
    cfg.effects.flashlight_radius = 200.0;
    cfg.render.filter_mode = FilterMode::Nearest;

    let toml_str = toml::to_string(&cfg).expect("Should serialize config to TOML");
    assert!(toml_str.contains("scroll_speed = 0.25"));
    assert!(toml_str.contains("flashlight_radius = 200.0"));

    let loaded: Config = toml::from_str(&toml_str).expect("Should deserialize config from TOML");
    assert_eq!(loaded.general.scroll_speed, 0.25);
    assert_eq!(loaded.effects.flashlight_radius, 200.0);
    assert_eq!(loaded.render.filter_mode, FilterMode::Nearest);
}

#[test]
fn test_config_from_toml_with_partial_fields() {
    let partial_toml = r#"
    [general]
    scroll_speed = 0.3
    "#;

    let cfg = Config::from_toml_str(partial_toml).expect("Should parse partial TOML with defaults");
    assert_eq!(cfg.general.scroll_speed, 0.3);
    assert_eq!(cfg.general.scale_friction, 0.85); 
}
