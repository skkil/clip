use std::sync::mpsc::Sender;

use anyhow::{Context, Result};
use ashpd::desktop::CreateSessionOptions;
use ashpd::desktop::global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut};
use futures_util::StreamExt;

use crate::action::Action;
use crate::app;

pub fn run(actions: &[Action], sink: Sender<Action>) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build portal runtime")?
        .block_on(serve(actions, sink))
}

async fn serve(actions: &[Action], sink: Sender<Action>) -> Result<()> {
    let portal = GlobalShortcuts::new()
        .await
        .context("no GlobalShortcuts portal on the session bus")?;
    let session = portal
        .create_session(CreateSessionOptions::default())
        .await
        .with_context(|| {
            format!(
                "the portal refused a session; it needs {id}.desktop installed and the daemon \
                 running as app-{id}.service (see `{bin} service install`)",
                id = app::APP_ID,
                bin = app::BIN
            )
        })?;

    let requested: Vec<NewShortcut> = actions
        .iter()
        .map(|a| NewShortcut::new(a.id(), a.description()).preferred_trigger(a.preferred_trigger()))
        .collect();
    let bound = portal
        .bind_shortcuts(&session, &requested, None, BindShortcutsOptions::default())
        .await
        .context("bind shortcuts")?
        .response()
        .context("bind shortcuts response")?;
    for shortcut in bound.shortcuts() {
        let trigger = match shortcut.trigger_description() {
            "" => "unbound; bind it in your compositor",
            t => t,
        };
        eprintln!(
            "{}: shortcut {} registered ({trigger})",
            app::NAME,
            shortcut.id()
        );
    }

    let mut activated = portal
        .receive_activated()
        .await
        .context("listen for shortcuts")?;
    while let Some(event) = activated.next().await {
        if let Some(action) = Action::from_id(event.shortcut_id())
            && sink.send(action).is_err()
        {
            break;
        }
    }
    Ok(())
}
