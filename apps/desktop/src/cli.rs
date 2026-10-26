use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

pub const SERVE_CLIPBOARD: &str = "__serve-clipboard";

#[derive(Parser, Debug)]
#[command(name = crate::app::BIN, version, about = "clip: screen capture, OCR and clipboard daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the background daemon that owns the global shortcuts.
    Daemon,
    /// Ask the running daemon for its status.
    Status,
    /// Capture the screen (defaults to dragging out a region).
    Shot {
        #[command(subcommand)]
        target: Option<ShotTarget>,
        #[command(flatten)]
        options: ShotOptions,
    },
    /// Print the text in an image.
    Ocr { image: PathBuf },
    /// Start the daemon at login.
    #[command(subcommand)]
    Service(ServiceCommand),
    /// Show or create the configuration file.
    #[command(subcommand)]
    Config(ConfigCommand),
    #[cfg(target_os = "linux")]
    #[command(name = SERVE_CLIPBOARD, hide = true)]
    ServeClipboard,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ShotTarget {
    /// Drag out a region.
    Region,
    /// Drag out a region and pick the text to copy.
    Text,
    /// Click a display.
    Screen,
    /// Every display, without selecting.
    Full,
    /// One display by name, without selecting.
    Output { name: String },
}

#[derive(Args, Debug, Clone, Default)]
pub struct ShotOptions {
    /// Write the capture here. Supports %Y %m %d %H %M %S.
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,
    /// Do not copy the capture to the clipboard.
    #[arg(long, global = true)]
    pub no_copy: bool,
    /// Do not save the capture to a file.
    #[arg(long, global = true)]
    pub no_save: bool,
    /// Seconds to wait before capturing.
    #[arg(short, long, global = true, default_value_t = 0.0)]
    pub delay: f64,
    /// Include the mouse cursor. Toggle with `c` in the overlay.
    #[arg(long, global = true)]
    pub cursor: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ServiceCommand {
    /// Install the launcher entry and the login service.
    Install,
    /// Remove them.
    Uninstall,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommand {
    /// Write a commented template, unless the file already exists.
    Init,
    /// Print the configuration in effect.
    Show,
}
