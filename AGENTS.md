# AGENTS.md

skkil/clip is a cross-platform clipboard manager with a Chrome extension, CLI
tool, server, web app, and mobile app. This document describes the architecture
and structure of the repository.

CLIP is end-to-end encrypted: the server is a **zero-knowledge relay** that
stores and routes ciphertext and can never read clip content. Beyond sync, it
turns any clip into a shareable URL (expiry, burn-after-reading, optional
password). Scope is strictly clipboard management and sync — no note-taking,
no team collaboration, no password-manager features.

## Domain model

- **Account** — owns devices and clips; anchors the encryption key, which is
  derived client-side and never sent to the server.
- **Device** — a registered client (extension, CLI, desktop, mobile) with a
  stable identity used for sync tracking and conflict resolution.
- **Clip** — encrypted content, format/content-type metadata, source device,
  a logical-clock timestamp, and tags/pin state.
- **Logical clock** — a Lamport or hybrid-logical-clock (HLC) value per clip,
  used for ordering and conflict resolution across devices.
- **SharedLink** — a shareable URL for a clip with expiry/burn-after-reading
  and optional password. The decryption key lives in the URL fragment (after
  `#`), which browsers never send to the server — so the operator only ever
  holds ciphertext.
- **Sync state** — each device's view of what it has synced, an offline queue
  for clips captured while disconnected, and gap recovery on reconnect.
- **Blob** — large payloads (images) stored by reference, content-addressed
  by SHA-256 hash, never inline.

## Sync & conflict resolution

Default approach: **per-item last-write-wins with hybrid-logical-clock
timestamps**, not a full CRDT. CRDTs solve data-structure conflicts, not
business-logic ones, and add compaction/snapshot burden over a long history —
LWW + HLC is the pragmatic fit for a clipboard's use case. Revisit only if a
documented case for principled convergence emerges (see `SharedLink`/offline
queue design).

## Real-time propagation

New clips fan out to a user's other online devices over WebSockets, backed by
a Redis/NATS-style pub/sub backplane. Presence tracks which devices are
online; offline devices get a queue that replays on reconnect.

## Cross-platform clipboard formats

The same "copy" produces non-1:1 format sets per OS — this is a lossy-mapping
problem, not just a serialization one:

- **macOS** — `NSPasteboard` holds multiple UTI-identified representations of
  one item (RTF, HTML, plain string), plus dynamic `dyn.*` UTIs.
- **Windows** — `CF_` format constants; `CF_HTML` is UTF-8 (unlike Windows'
  usual UTF-16) with a byte-offset header; bitmaps span `CF_BITMAP` /
  `CF_DIB` / `CF_DIBV5`. Windows also supports delayed rendering, which can
  hang the UI while a format is produced on demand.
- **Linux** — X11 has three non-standardized selections (`CLIPBOARD` /
  `PRIMARY` / `SECONDARY`); Wayland is a separate protocol entirely.

## Mobile platform constraints

- **Android IPC limit** — clipboard `ClipData` passes over Binder IPC with a
  ~1 MB transaction buffer shared per process (practical safe limit ~500 KB).
  Large images/HTML must be passed by reference/URI, not inline.
- **Android access limit** — apps can't read the clipboard on Android 10+
  unless they are the default IME or the focused app, so background polling
  is impossible — this forces an IME/foreground/share-sheet design. Android
  13+ also shows user-visible clipboard-access alerts.
- Response: content-addressed blob storage (SHA-256), chunking for large
  items, and deduplication (hash-match a re-copied item, bump its timestamp
  instead of re-storing).

## Security

- Zero-knowledge: server only ever holds ciphertext.
- Client-side AES-256-GCM; key derived via a strong KDF (PBKDF2 with high
  iteration counts, or Argon2); key never leaves the device.
- Respect OS sensitive/transient flags (e.g. `org.nspasteboard.ConcealedType`
  on macOS) so passwords/generated secrets are not logged or synced.
- Crash reports/analytics must never capture clip content — hard-block any
  log field that could contain content.
- SharedLink threat model: the decryption key lives in the URL fragment,
  never sent to the server, so the link itself is the secret — expiry, burn,
  and password exist to contain that.

## Repository structure

Multi-app monorepo without a workspace manager:

- `apps/chrome/` - Chrome extension (React + TypeScript + Vite)
- `apps/cli/` - Go CLI tool
- `apps/server/` - Go server (empty stub)
- `apps/web/` - Web application
- `apps/mobile/` - Mobile application

Each app is independent with its own package manager and build system.

## Chrome extension (`apps/chrome/`)

### Build system quirk

The extension has **three separate Vite configs** that must all run:

```bash
npm run build
```

Actually executes:

```bash
rm -rf dist && \
tsc -b && \
vite build --config ./vite/popup.config.ts && \
vite build --config ./vite/background.config.ts && \
vite build --config ./vite/content.config.ts
```

All three configs write to `dist/` with `emptyOutDir: false` - they accumulate
output instead of replacing it.

### Entrypoints

- **Popup**: `index.html` → `src/main.tsx` → `src/App.tsx`
- **Background script**: `src/background/index.tsx`
- **Content script**: `src/content/index.tsx`

### React Compiler enabled

The project uses React Compiler via `babel-plugin-react-compiler`. This affects
build performance and requires the babel plugin in the popup config.

### Dev workflow

```bash
cd apps/chrome
npm run dev      # Popup only (index.html)
npm run build    # All three bundles
npm run lint     # ESLint
```

No workspace-level commands. Each app runs independently.

## Go apps (`apps/cli/`, `apps/server/`)

### CLI app

Build:

```bash
cd apps/cli
make build    # runs go mod tidy, then builds to bin/clip
```

Entrypoint: `cmd/main.go` → `internal/cmd/`

Uses Cobra for CLI framework and Bubble Tea for TUI.

### Server app

Build:

```bash
cd apps/server
make build    # runs go mod tidy, then builds to bin/clip
```

Currently a stub with empty `main()`.

### Naming collision

Both Go apps output to `bin/clip` in their respective directories. This is
intentional - they are separate binaries.

## No root-level tooling

- No `pnpm-workspace.yaml` or root `package.json`
- No GitHub Actions workflows
- No shared ESLint, TypeScript, or prettier configs
- Each app is self-contained

## Chrome extension manifest

`apps/chrome/public/manifest.json` is copied to `dist/` during build. Content
script CSS must be at `assets/content.css` per manifest requirement.
