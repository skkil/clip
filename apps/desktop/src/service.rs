use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::app;

#[cfg(target_os = "linux")]
const ICON: &[u8] = include_bytes!("../packaging/org.skkil.clip.png");

pub fn install() -> Result<()> {
    let exe = std::env::current_exe().context("locate own executable")?;
    for (path, contents) in files(&exe)? {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        std::fs::write(&path, contents).with_context(|| format!("write {}", path.display()))?;
        println!("wrote {}", path.display());
    }
    println!(
        "\nenable it with:\n  systemctl --user daemon-reload\n  systemctl --user enable --now app-{}.service",
        app::APP_ID
    );
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let exe = std::env::current_exe().context("locate own executable")?;
    for (path, _) in files(&exe)? {
        match std::fs::remove_file(&path) {
            Ok(()) => println!("removed {}", path.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).with_context(|| format!("remove {}", path.display())),
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn files(exe: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| app::home().join(".local/share"));
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| app::home().join(".config"));
    let (name, id, exe) = (app::NAME, app::APP_ID, exe.display());

    let desktop = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={name}\n\
         Comment=Screen capture and OCR\n\
         Exec={exe} shot\n\
         Icon={id}\n\
         Terminal=false\n\
         Categories=Utility;\n\
         Keywords=clipboard;screenshot;ocr;\n"
    );
    let service = format!(
        "[Unit]\n\
         Description={name} desktop daemon\n\
         PartOf=graphical-session.target\n\
         After=graphical-session.target\n\n\
         [Service]\n\
         Type=simple\n\
         Slice=app.slice\n\
         ExecStart={exe} daemon\n\
         Restart=always\n\
         RestartSec=2\n\n\
         [Install]\n\
         WantedBy=graphical-session.target\n"
    );

    Ok(vec![
        (
            data.join(format!("applications/{id}.desktop")),
            desktop.into_bytes(),
        ),
        (
            data.join(format!("icons/hicolor/256x256/apps/{id}.png")),
            ICON.to_vec(),
        ),
        (
            config.join(format!("systemd/user/app-{id}.service")),
            service.into_bytes(),
        ),
    ])
}

#[cfg(not(target_os = "linux"))]
fn files(_exe: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    anyhow::bail!(
        "login service is not implemented on {}",
        std::env::consts::OS
    )
}
