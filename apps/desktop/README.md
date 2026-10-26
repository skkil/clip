# clip-desktop

The CLIP desktop app: freeze-the-screen capture, OCR text copy and a
background daemon for global shortcuts, in one binary. The executable is
`clip-desktop` so it does not collide with the `clip` CLI; everything a user
sees says `clip`.

Only Linux/Wayland is implemented (wlroots `screencopy` + `layer-shell`;
window snapping uses `hyprctl` on Hyprland). Other platforms plug in behind
the `Platform` trait in `src/platform/mod.rs`.

## Commands

```
clip-desktop shot [region]     drag out a region or click a window (default)
clip-desktop shot text         same, then pick the lines of text to copy (OCR)
clip-desktop shot screen       click a display
clip-desktop shot full         every display, no selection
clip-desktop shot output NAME  one display, no selection
clip-desktop ocr IMAGE         print the text in a PNG
clip-desktop daemon            register global shortcuts (shot, text)
clip-desktop status            ask the running daemon how it is doing
clip-desktop service install   launcher entry + systemd user unit
clip-desktop config init|show
```

Shot flags: `-o/--output PATH` (supports `%Y %m %d %H %M %S`), `--no-copy`,
`--no-save`, `-d/--delay SECS`, `--cursor`.

Every capture is copied to the clipboard and saved to
`~/Pictures/Screenshots` unless told otherwise, and a desktop notification
(`notify-send`) says what happened or why it failed. Only one overlay can be
open at a time; a second invocation exits silently.

## In the overlay

A toolbar at the top shows the current choices; click a button or press its
key.

| Button | Key | Effect |
| --- | --- | --- |
| Region / Screen | `Tab` | drag an area or click a window / click a display |
| Image / Text | `T` | copy a picture, or recognise the text and pick lines |
| Cursor | `C` | show / hide the mouse cursor |
| Edit | `E` | after selecting, black out or blur parts before copying |

Releasing the mouse copies straight away unless Edit is on (holding `Ctrl`
while selecting does the same). Hold `Z` for a magnifier; `Esc`, `q` or
right-click cancels.

- **Editor:** Black is picked; drag to cover. `1`/`2` switch Black/Mosaic,
  `U` or `Backspace` undoes, arrows grow an edge and `Shift`+arrows shrink
  it, `Enter` or the Copy button copies.
- **Text:** recognised lines are outlined. Click a line to pick it, drag
  across lines to pick them (`Shift` adds), `A` picks all. `Enter` copies the
  picked lines, or everything when none are picked.

## OCR

Uses [`apps/shared/ocr`](../shared/ocr), the same engine the Chrome extension
will run as WebAssembly. Install the models once:

```
apps/shared/ocr/scripts/fetch-models.py --out ~/.local/share/clip/ocr
```

## Configuration

`~/.config/clip/config.toml`, every key optional (`clip-desktop config init`
writes a commented template):

```toml
[storage]
screenshots = "~/Pictures/Screenshots"   # CLIP_SHOT_DIR overrides
filename = "clip_%Y-%m-%d_%H-%M-%S.png"
save = true

[ocr]
models = "~/.local/share/clip/ocr"
scripts = ["korean", "chinese", "latin"]
```

## Setup

```
cargo build --release
install -Dm755 target/release/clip-desktop ~/.local/bin/clip-desktop
clip-desktop service install
systemctl --user daemon-reload
systemctl --user enable --now app-org.skkil.clip.service
```

The GlobalShortcuts portal only accepts the daemon when it runs as
`app-org.skkil.clip.service` **and** `org.skkil.clip.desktop` is installed;
`service install` writes both. On Hyprland the portal still leaves the key
binding to you, so binding the binary directly is simpler and needs no
daemon:

```
bind = , Print,       exec, ~/.local/bin/clip-desktop shot
bind = SHIFT, Print,  exec, ~/.local/bin/clip-desktop shot text
```

Use absolute paths: the compositor does not inherit your shell's `PATH`.
Avoid binding capture behind `Ctrl`: a held `Ctrl` opens the editor.

## Layout

```
src/
  main.rs cli.rs        command line
  shot.rs               capture → select → render → copy/save
  daemon.rs ipc.rs      background daemon and its control socket
  service.rs            launcher entry + systemd unit
  capture.rs            multi-display capture model, cursor layer, edits
  select.rs menu.rs     overlay state machine and editor menu layout
  raster.rs text.rs     RGBA image ops and glyph rendering
  ocr.rs config.rs ...
  platform/
    mod.rs              Platform trait
    wayland/            capture, overlay, clipboard, windows, shortcuts
```
