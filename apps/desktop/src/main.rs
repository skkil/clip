mod action;
mod app;
mod capture;
mod cli;
mod config;
mod daemon;
mod geom;
mod ipc;
mod lock;
mod menu;
mod naming;
mod ocr;
mod platform;
mod raster;
mod select;
mod service;
mod shot;
mod text;

use anyhow::{Context, Result, bail};
use clap::Parser;

use cli::{Cli, Command, ConfigCommand, ServiceCommand, ShotTarget};
use config::Config;

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Daemon => daemon::run(),
        Command::Status => daemon::status(),
        Command::Shot { target, options } => {
            shot::run(target.unwrap_or(ShotTarget::Region), options)
        }
        Command::Ocr { image } => {
            let bytes =
                std::fs::read(&image).with_context(|| format!("read {}", image.display()))?;
            let image = raster::Image::from_png(&bytes)?;
            println!("{}", ocr::recognize(&Config::load()?.ocr, &image)?);
            Ok(())
        }
        Command::Service(ServiceCommand::Install) => service::install(),
        Command::Service(ServiceCommand::Uninstall) => service::uninstall(),
        Command::Config(ConfigCommand::Init) => init_config(),
        Command::Config(ConfigCommand::Show) => {
            println!("# {}\n{:#?}", Config::path().display(), Config::load()?);
            Ok(())
        }
        #[cfg(target_os = "linux")]
        Command::ServeClipboard => platform::wayland::clipboard::serve_from_stdin(),
    }
}

fn init_config() -> Result<()> {
    let path = Config::path();
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, config::TEMPLATE)?;
    println!("wrote {}", path.display());
    Ok(())
}
