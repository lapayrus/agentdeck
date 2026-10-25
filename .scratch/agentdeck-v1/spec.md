Status: ready-for-agent

# AgentDeck V1 — see, disable and remove agent configuration across ecosystems

## Problem Statement

Developers now run several AI coding tools side by side — Claude Code, Codex, OpenCode and Command Code — each with its own directories, file formats, scope rules and precedence order. A developer cannot answer basic questions without reading documentation and hand-inspecting hidden folders: which agents, skills and MCP servers exist for each tool, which copy of a same-named artifact actually wins, which ones live only on this machine, and which ones are shared by several tools at once. Disabling something means knowing a different mechanism per tool: a permissions list here, an `enabled` flag there, and for two artifact types nothing but moving a file. Hand-editing those files is risky because several tools rewrite them, some carry comments that naive edits destroy, and at least one holds unrelated trust state that corrupts into a reset prompt.

## Solution

AgentDeck is a Windows desktop app that presents one unified view of every agent, skill and MCP server the four ecosystems load, for one selected project and for the machine as a whole. It resolves each ecosystem's precedence rules and shows the effective definition, marks the copies that lose, and shows which other ecosystems load the same path. From that view a developer can enable, disable or remove an artifact with one action, targeting either the project or the global scope, and every change is reversible: a snapshot is taken before each write, removed artifacts move to an app-managed trash, and artifacts whose ecosystem has no disable switch move to a quarantine area instead of being renamed in place.

The app is Claude-inspired in appearance, offline except for the update check, open source, and deliberately small and fast: a Rust core owns all disk access and the UI is a thin typed layer over it.

## User Stories

1. As a developer using several AI coding tools, I want to open one project folder and see every agent, skill and MCP server all four ecosystems load for it, so that I stop guessing what is active.
2. As a developer, I want the app to resolve each ecosystem's precedence rules and show me the definition that actually wins, so that a same-named artifact is not ambiguous.
3. As a developer, I want shadowed definitions marked and collapsed under the effective one, so that I can still see the file that is being ignored without it cluttering the list.
4. As a developer, I want to open a raw inspector for any artifact and see every file path and its contents, so that I can verify what the app read.
5. As a developer, I want the same directory that three ecosystems load to appear once with ecosystem badges, so that I do not mistake it for three separate skills.
6. As a developer, I want a disable confirmation that names every ecosystem affected by a shared artifact, so that removing one skill cannot silently break two other tools.
7. As a developer, I want to switch between Global, Project and Project-local scopes, so that I can see what is shared with my team versus private to me.
8. As a developer, I want artifacts from an ecosystem that is not installed to appear collapsed with the paths that were checked, so that I understand why the app cannot see a tool I believe is installed.
9. As a developer, I want the app to remember recently opened projects, so that reopening my usual repos takes one click.
10. As a developer, I want a folder that is not a git repository to still open as a project, with a badge saying so, so that I can manage configuration in scratch directories.
11. As a developer, I want the app to resolve the project root to the enclosing git root, so that project-scope artifacts are found the same way the ecosystems find them.
12. As a developer, I want an Agents view listing every defined agent per ecosystem, so that I can audit what delegates work on my behalf.
13. As a developer, I want a Skills view listing every skill with its name, description and root, so that I can see which instructions are available.
14. As a developer, I want an MCP view listing every server with transport, command or URL, and enabled state, so that I know what the tools can reach.
15. As a developer, I want an Other view listing hooks, slash commands, plugins and marketplaces, memory and instruction files, and settings files read-only, so that I still get a complete picture of the machine.
16. As a developer, I want to disable an agent using the ecosystem's native mechanism when one exists, so that the change is the same one the ecosystem itself would make.
17. As a developer, I want to disable an agent by quarantine when the ecosystem has no native switch, so that the definition stops loading without being destroyed.
18. As a developer, I want to disable a skill through the ecosystem's native settings where available, so that the skill remains discoverable and easy to re-enable.
19. As a developer, I want to disable an MCP server through its `enabled`/`disabled` flag where available, so that I keep its configuration for later.
20. As a developer, I want removal to move an artifact into an app-managed trash rather than delete it, so that a mistaken click is recoverable.
21. As a developer, I want to restore a trashed artifact to its original location, so that undo is a first-class action and not a file-system chore.
22. As a developer, I want to see a version history of every configuration file the app has changed, so that I can roll back a change the app made earlier.
23. As a developer, I want the app to take a timestamped snapshot before every write, so that no write is unrecoverable.
24. As a developer, I want the app to preserve comments and formatting when it edits a TOML or JSONC config, so that my hand-written notes are not wiped.
25. As a developer, I want the app to abort and ask me to rescan when a file changed since it was scanned, so that it never overwrites an edit made elsewhere.
26. As a developer, I want a plain-language explanation when an action needs a restart or reconnect to take effect, so that I do not assume a toggle failed.
27. As a developer, I want the app to notice files changing while it is open and refresh after a short debounce, so that it stays truthful after I edit something in another editor.
28. As a developer, I want a manual refresh that rescans everything, so that I have an escape hatch when a watcher misses something.
29. As a developer, I want read-only artifacts to offer open and reveal-in-Explorer actions, so that inspecting them is quick without making them editable.
30. As a developer, I want a light, dark and system theme in a Claude-inspired palette, so that the app is comfortable to use for long sessions.
31. As a developer, I want the window to remember its size and position, so that it opens where I left it.
32. As a developer, I want to filter the current list by name, so that finding one artifact in a large installation does not require scrolling.
33. As a developer, I want to see which mode the app is in when an ecosystem version is too old for an action, so that I understand why a control is disabled.
34. As a developer, I want to set an explicit binary path for an ecosystem that is installed outside `PATH`, so that version detection still works.
35. As a developer, I want the app to launch quickly and stay light in memory, so that a configuration viewer does not become the heaviest thing I run.
36. As a developer, I want the app to make no network request other than the update check, so that I can trust it with my configuration files.
37. As a developer, I want to be told when a new version is published and update in one click, so that staying current is not a manual download.
38. As a developer, I want to disable the update check, so that the app can run fully offline.
39. As a developer, I want the update to be signature-verified, so that a compromised download cannot be installed silently.
40. As an open-source contributor, I want the repository to build with one documented command and run its tests in CI, so that I can contribute without reverse-engineering the setup.
41. As an open-source contributor, I want the ecosystem-specific paths and formats documented in one place, so that adding a fifth ecosystem is a data change rather than a rewrite.
42. As a maintainer, I want typed contracts between the Rust core and the UI, so that the two sides cannot drift.
43. As a maintainer, I want every disk mutation to go through one audited surface, so that there is exactly one place to review for safety.
44. As a maintainer, I want scan and action behaviour tested against synthetic ecosystem trees, so that tests never touch a real user's configuration.
45. As a maintainer, I want a performance benchmark in CI, so that the scan budget is enforced rather than aspirational.
46. As a developer, I want the app to show an empty state that explains what it looked for when nothing is found, so that a blank list is not mistaken for a bug.
47. As a developer, I want the app to show errors from unreadable or malformed files without failing the whole scan, so that one broken artifact does not hide the rest.
48. As a developer, I want each listed artifact to show which scope and which concrete file it came from, so that I can find it myself.
49. As a developer, I want actions to be scoped to the selected scope, so that disabling a project artifact cannot accidentally change the global one.
50. As a developer, I want a confirmation before any removal, so that a destructive-looking action is deliberate.

## Implementation Decisions

**Shell and stack.** Tauri 2 desktop app: a UI-free Rust core crate, a Tauri application crate, and a React + TypeScript + Vite frontend. The core owns every filesystem read, parse, watch and write; the frontend receives typed view models and sends typed commands. No filesystem plugin is exposed to the frontend. See ADR-0001.

**Domain model.** The core exposes the glossary vocabulary: Ecosystem, Artifact, Managed artifact, Read-only artifact, Scope (Global, Project, Project-local), Effective state, Shadowed, Disable, Remove, Quarantine, Restore. Artifact identity is ecosystem plus artifact type plus name plus origin path; a shared origin path yields one entry with multiple ecosystems and an `affected_ecosystems` set used by confirmation dialogs.

**Discovery.** A declarative discovery manifest, one entry per known path per ecosystem, declares scope, artifact type, recursion depth, whether the path is ancestor-walked, and which parser applies. There is no generic directory crawl and no discovery by brute force. Unknown directories are never opened. Recursion is bounded inside known roots, skipping `node_modules` and irrelevant hidden content. The manifest is derived from `docs/research/agent-config-manager.md`, including its exception cases: non-dot files at the project root, Codex skills under `.agents`, compatibility roots read by OpenCode and Command Code, and the derived per-project Command Code path.

**Injection seam.** The core is constructed with an explicit value holding the home directory, app-data directory, project root and optional per-ecosystem overrides. It does not read environment variables internally, so tests can point it at fixture trees. See "Testing Decisions".

**Parsing.** Parsers are format-preserving. TOML edits preserve comments and key order; JSONC edits preserve comments and whitespace; Markdown frontmatter is read without rewriting the document unless the frontmatter itself is being changed. Raw file content is retained for the raw inspector. Malformed files produce per-file warnings and do not abort a scan.

**Effective resolution.** Resolution applies each ecosystem's documented precedence separately per artifact type; there is no single global rule. Losers are returned as Shadowed entries attached to the winner. Shared paths are detected across ecosystems and deduplicated into one entry with badges. Where research recorded an unresolved precedence conflict, the UI reports the state as unknown rather than asserting a winner.

**Actions.** One audited action surface handles enable, disable, remove and restore. Each action produces a plan describing the files and keys it will touch and the ecosystems affected, which the UI shows before confirmation. Every write takes a timestamped snapshot, re-reads and hash-checks the target, writes via temporary file plus atomic replace with retry on Windows sharing violations, then triggers a rescan. Configuration-key removals snapshot the whole file and delete only the key; file-based removals move the artifact into trash; disabling where no native switch exists moves it into quarantine.

**Recoverability.** Trash, quarantine and history are app-managed, each with a manifest recording origin path, scope, ecosystem, timestamp and the snapshot it corresponds to. V1 does not purge them automatically; pruning is a manual action in Settings. Restore detects a conflicting later edit and surfaces it rather than overwriting.

**Freshness.** Filesystem watchers cover the manifest's roots, debounced at 300 ms, with a manual refresh. Per-ecosystem reload hints describe whether the change applies on the next turn, the next session or after a restart.

**Detection and capability gating.** An ecosystem is detected by the presence of its config root. Its version is probed from its binary when available, with a user-supplied path override for binaries outside `PATH`. A capability matrix describes which actions a detected version supports, and unsupported controls are disabled with the reason shown.

**Persistence.** A single app-data root holds `state.json` (recent projects, window geometry, theme mode, binary overrides, update-check preference), `trash/`, `quarantine/` and `history/`, all written atomically. No database.

**UI.** One window: left rail of ecosystems, a scope switch above the artifact list, artifact-type tabs (Agents, Skills, MCP, Other), a list in the middle and a detail pane on the right containing metadata, actions and the raw inspector. Confirmation dialogs name affected ecosystems. Theme is Claude-inspired with light, dark and system modes. Project selection is a folder picker with a recents list; the settled project root is displayed and a non-git root is badged.

**Contracts and packaging.** Rust view models and command payloads are the source of truth; TypeScript types are generated from them and a CI check fails on drift. App name is AgentDeck; the repository is MIT licensed and open source. Distribution is an MSI (machine-wide) and an NSIS installer (per-user, updater-capable); no portable build in V1. Updates use Tauri's updater with minisign-signed artifacts published to GitHub Releases, checked at launch with a throttle plus a manual check, and disableable in Settings. The update check is the only network request the app ever makes.

**Performance.** Budgets: cold start under 1.5 s; a 10,000-file project scan under 2 s; idle working set under 150 MB across app and WebView2 processes; no filesystem work on the UI thread; 300 ms watcher debounce. A CI benchmark enforces the scan budget.

## Testing Decisions

A good test drives observable behaviour through a public surface and asserts on outputs and resulting filesystem state, not on internal structure. Tests must never read or write a real user configuration directory.

**Primary seam — the core manager.** All scanning, parsing, resolution, action planning, snapshotting, writing, quarantine, trash and restore is tested through one core entry point constructed with injected roots. Tests build synthetic ecosystem trees in temporary directories — shapes copied from the research document, including malformed frontmatter, JSONC with comments, TOML with comments and blank-line runs, same-name collisions across scopes, a path shared by three ecosystems, a non-git project root, and an ecosystem with no config root. Assertions cover the returned snapshot, the shadowed list, warnings, the action plan, the resulting file contents, and the manifest written to trash, quarantine or history. Round-trip properties: enable then disable returns the original effective state and byte-identical file content where the ecosystem allows it; remove then restore returns the file to its origin; every write stays inside an allowed root.

**Secondary seam — the frontend command client.** A thin typed interface injected into React components, faked in component tests. It exists only for three flows: rendering effective versus shadowed entries, the disable confirmation naming affected ecosystems, and remove/restore. No general UI testing and no end-to-end run of the packaged app in V1.

**Contracts.** Serde round-trip tests for every command payload and view model, plus the generated-TypeScript drift check.

**What is not tested through the UI seam.** Parsing, precedence, planning and mutation correctness are core tests; duplicating them through the UI would test the test harness.

## Out of Scope

- Creating or adding agents, skills or MCP servers (V2).
- Editing hooks, slash commands, plugins, memory files or settings beyond viewing them.
- Installing, upgrading or uninstalling the ecosystems themselves.
- macOS and Linux builds; the architecture and packaging pipeline must allow them, but V1 ships Windows only.
- Cloud sync, accounts, telemetry, crash reporting.
- A portable-build distribution and any update path for it.
- Multi-window, tray behaviour and localization.
- A command-line interface.

## Further Notes

Precedence and reload behaviour differ per ecosystem and per artifact type; several points remain unverified in the research document. Those are recorded as open questions there and must surface as "unknown" in the UI rather than as asserted facts.

The riskiest surfaces are `~/.claude.json`, which carries unrelated per-project trust state, and the config files that the ecosystems rewrite themselves. ADR-0002 governs the mitigation. The raw inspector and the version history exist partly so a user can verify and unwind any change without trusting the app's summary.

Reference material: `docs/research/agent-config-manager.md` for the path and format inventory, `GLOSSARY.md` for vocabulary, ADR-0001 for the stack and ADR-0002 for mutation safety.
