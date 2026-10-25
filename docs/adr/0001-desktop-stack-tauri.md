# Desktop stack: Tauri 2 with a Rust core

## Context

The app is Windows-first, must stay memory-frugal and fast, will later ship on macOS and Linux, and edits configuration files owned by other tools. Electron was rejected for memory cost. A Slint (Rust-native UI) branch was explored and dropped.

## Decision

Build on Tauri 2: a Rust `core` crate owns all filesystem access, parsing, watching, precedence resolution and mutation; a React + TypeScript + Vite frontend renders in the system WebView2 on Windows and the native webview elsewhere. The frontend talks to Rust through typed Tauri commands only.

## Considered Options

- **Electron**: fastest delivery and the richest Node parsing libraries, but a bundled Chromium and a large installer, which conflicts with the memory requirement.
- **Slint**: no webview and a small memory footprint, but it introduces the Slint Royalty-free licence's attribution obligation and a smaller UI ecosystem.
- **Tauri 2**: keeps the Rust core and a familiar web UI, uses the webview the OS already provides, and is MIT/Apache-2.0 licensed.

## Consequences

Windows depends on the WebView2 runtime, which ships with Windows 11 and current Windows 10 builds, and the installer must handle its absence. The IPC boundary is a real seam: every disk operation must be a command, never a frontend filesystem plugin. Web technology sets the UI stack for macOS and Linux as well.
