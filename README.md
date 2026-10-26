# AgentDeck

One window for the agents, skills and MCP servers installed across Claude Code, Codex, OpenCode and Command Code.

AgentDeck reads each ecosystem's own configuration, resolves its precedence rules, and shows the definition that actually wins. Agents, skills and MCP servers can be disabled or removed from either the global or the project scope, with a snapshot taken before every write and removed artifacts moved to an app-managed trash rather than deleted.

## Status

V1 is in development and Windows-only. The architecture and release pipeline leave room for macOS and Linux.

## Requirements

- Windows 10/11 with the WebView2 runtime
- Node.js 22+
- Rust stable with the MSVC toolchain
- Visual Studio Build Tools with the C++ workload

## Development

```powershell
npm install --include=dev
npm run tauri dev
```

## Checks

```powershell
npm test
npm run build
cargo test -p agentdeck-core
cargo check -p agentdeck
```

## Documentation

- `GLOSSARY.md` — the domain vocabulary
- `docs/adr/` — architecture decisions
- `docs/research/agent-config-manager.md` — the configuration surface of each ecosystem
- `.scratch/agentdeck-v1/spec.md` — the V1 spec and tickets

## Releases and updates

Releases are built by `.github/workflows/release.yml` on a `v*` tag. The workflow produces an MSI, an NSIS installer and signed updater artifacts for a draft GitHub Release.

Two repository secrets are required:

- `TAURI_SIGNING_PRIVATE_KEY` — the contents of the minisign private key
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — its password, if one was set

The updater public key lives in `src-tauri/tauri.conf.json`. The private key must never be committed. The endpoint targets `https://github.com/lapayrus/agentdeck/releases/latest/download/latest.json`, so releases must be published from that repository.

To install the signing secret on a machine with the GitHub CLI:

```powershell
gh secret set TAURI_SIGNING_PRIVATE_KEY --repo lapayrus/agentdeck --body (Get-Content "$env:USERPROFILE\.agentdeck\updater.key" -Raw)
```

The app checks for updates at launch (throttled by the updater plugin) and on demand from Settings. The check is a signed request to GitHub Releases and is the only network call the app makes; it can be disabled.

## Generated types

The TypeScript view models are generated from the Rust core by `ts-rs`:

```powershell
npm run types
```

This writes `src/generated/`. CI regenerates them and fails if the committed files differ, so change the Rust type and rerun the command rather than editing `src/generated` or `src/types.ts`.

## License

MIT
