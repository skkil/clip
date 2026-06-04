# AGENTS.md

skkil/clip is a cross-platform clipboard manager with a Chrome extension, CLI
tool, server, web app, and mobile app. This document describes the architecture
and structure of the repository.

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
