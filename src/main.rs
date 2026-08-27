use clap::Parser;
use hypr_zoomer::app::run;
use hypr_zoomer::config::{Config, FilterMode};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hypr_zoomer")]
#[command(author = "DeepMind & Antigravity")]
#[command(version = "0.1.0")]
#[command(about = "High-performance Wayland screen zoomer & presentation magnification tool inspired by Boomer", long_about = None)]
struct Cli {
    
    #[arg(short = 's', long)]
    scale: Option<f32>,

    
    #[arg(short = 'g', long)]
    geometry: Option<String>,

    
    #[arg(short = 'w', long)]
    window: bool,

    
    #[arg(short = 'f', long)]
    flashlight: bool,

    
    #[arg(long)]
    nearest: bool,

    
    #[arg(long)]
    radius: Option<f32>,

    
    #[arg(short = 'c', long)]
    config: Option<PathBuf>,

    
    #[arg(long)]
    generate_config: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.generate_config {
        let default_cfg = Config::default();
        let toml_str = toml::to_string_pretty(&default_cfg)?;
        println!("{}", toml_str);
        return Ok(());
    }

    let mut config = if let Some(path) = args.config {
        let content = fs::read_to_string(&path)?;
        Config::from_toml_str(&content)?
    } else {
        Config::load_or_default()
    };

    if args.window {
        config.general.auto_track_active_window = true;
    }

    if args.nearest {
        config.render.filter_mode = FilterMode::Nearest;
    }

    if let Some(r) = args.radius {
        config.effects.flashlight_radius = r;
    }

    run(config, args.scale, args.geometry)
}

