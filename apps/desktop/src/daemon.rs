use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;

use anyhow::{Context, Result};

use crate::action::Action;
use crate::ipc::{self, Request, Response, Status};
use crate::{app, platform};

#[derive(Default)]
struct Stats {
    shortcuts: AtomicBool,
    activations: AtomicU64,
}

pub fn run() -> Result<()> {
    let listener = ipc::bind()?;
    let stats = Arc::new(Stats::default());
    let (tx, rx) = mpsc::channel();

    let shortcut_stats = Arc::clone(&stats);
    thread::spawn(move || {
        shortcut_stats.shortcuts.store(true, Ordering::Relaxed);
        if let Err(e) = platform::current().and_then(|p| p.shortcuts(&Action::ALL, tx)) {
            eprintln!(
                "{}: global shortcuts unavailable: {e:#}\nbind `{} shot` in your compositor instead",
                app::NAME,
                app::BIN
            );
        }
        shortcut_stats.shortcuts.store(false, Ordering::Relaxed);
    });

    let action_stats = Arc::clone(&stats);
    thread::spawn(move || {
        for action in rx {
            action_stats.activations.fetch_add(1, Ordering::Relaxed);
            if let Err(e) = spawn(action) {
                eprintln!("{}: {} failed: {e:#}", app::NAME, action.id());
            }
        }
    });

    #[cfg(unix)]
    exit_on_signal()?;

    eprintln!("{} daemon ready (pid {})", app::NAME, std::process::id());
    ipc::serve(listener, |request| match request {
        Request::Ping => Response::Pong,
        Request::Status => Response::Status(Status {
            pid: std::process::id(),
            version: env!("CARGO_PKG_VERSION").into(),
            shortcuts: stats.shortcuts.load(Ordering::Relaxed),
            activations: stats.activations.load(Ordering::Relaxed),
        }),
    });
    Ok(())
}

pub fn status() -> Result<()> {
    match ipc::send(&Request::Status)? {
        Response::Status(s) => {
            println!("{} daemon {} (pid {})", app::NAME, s.version, s.pid);
            println!(
                "shortcuts:   {}",
                if s.shortcuts {
                    "registered"
                } else {
                    "unavailable"
                }
            );
            println!("activations: {}", s.activations);
            Ok(())
        }
        other => anyhow::bail!("unexpected reply: {other:?}"),
    }
}

fn spawn(action: Action) -> Result<()> {
    let exe = std::env::current_exe().context("locate own executable")?;
    let mut child = Command::new(exe)
        .args(action.args())
        .spawn()
        .context("spawn capture")?;
    thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(unix)]
fn exit_on_signal() -> Result<()> {
    use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};

    let mut signals = signal_hook::iterator::Signals::new([SIGTERM, SIGINT, SIGHUP])?;
    thread::spawn(move || {
        if signals.forever().next().is_some() {
            if let Ok(path) = ipc::socket_path() {
                std::fs::remove_file(path).ok();
            }
            std::process::exit(0);
        }
    });
    Ok(())
}
