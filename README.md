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

## License

MIT
