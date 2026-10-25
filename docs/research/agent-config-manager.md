# Agent config surface

## Scope and method

Consolidates four field reports on what a Windows desktop app must read/write to list, enable/disable and remove (V1), then create (V2), **subagents, skills and MCP servers** for Claude Code, OpenAI Codex, OpenCode and Command Code, at **project** and **global** scope, with notes on hooks, commands, plugins and memory files. Each report combined the vendor's official documentation with direct inspection of an installed build on one Windows 11 machine (home `C:\Users\raosu`): Claude Code `2.1.215` (`C:\Users\raosu\.local\bin\claude.exe`), Codex `codex-cli 0.164.0-alpha.11.1` (`C:\Users\raosu\AppData\Local\OpenAI\Codex\bin\2993767bd1dfb009\codex.exe`, not on `PATH`), OpenCode `2.1.4` (desktop bundle `C:\Users\raosu\AppData\Local\Programs\@opencodedesktop\resources\opencode-cli.exe`; an older `1.1.65` install is also present) and `command-code@1.88.0` (`C:\nvm4w\nodejs\node_modules\command-code`). Summarised 2026-10-26. Bracketed numbers are the numbered source list at the end; everything else is cited inline by URL or absolute path. Claims that a report itself marked as unverified are collected under "Open questions / unverified", not silently promoted.

`~` below = `%USERPROFILE%` = `C:\Users\raosu` on Windows. Directory overrides matter: Claude Code honours `CLAUDE_CONFIG_DIR` [4], Codex honours `CODEX_HOME` (which must already exist if set) [12], OpenCode honours `XDG_CONFIG_HOME` (verified working on Windows) and `OPENCODE_CONFIG`/`OPENCODE_CONFIG_DIR`/`OPENCODE_CONFIG_CONTENT` [26][33], and **Command Code has no env var that relocates `~/.commandcode`** [42].

## Cross-ecosystem matrix

### Subagents

| Ecosystem | Project path | Global Windows path | Format | Required fields | Native disable | Remove mechanism | Hot reload | Src |
|---|---|---|---|---|---|---|---|---|
| Claude Code | `<repo>\.claude\agents\**\*.md` (recursive) | `C:\Users\raosu\.claude\agents\*.md`; managed `C:\Program Files\ClaudeCode\.claude\agents\` | Markdown + YAML frontmatter, body = system prompt | `name` (≤256 chars, no `:`, must not start `-`), `description` | `permissions.deny: ["Agent(<name>)"]` in any settings file, or `claude --disallowedTools "Agent(Explore)"` | delete the `.md` (live session drops it) | watched `~/.claude/agents/` and `.claude/agents/`, few seconds; 3 restart exceptions | [1][6][39] |
| Codex | `<repo>/.codex/agents/<name>.toml` | `%USERPROFILE%\.codex\agents\<name>.toml` | TOML, 1 file = 1 agent; identity is the `name` field, not the filename | `name`, `description`, `developer_instructions` | none per file; global only: `agents.enabled = false` or `features.multi_agent = false` | delete/rename the `.toml` | none documented → restart | [13][9][40] |
| OpenCode | `.opencode/agents/<name>.md` (V1 also `agent/`, `mode/`, `modes/`); inline `agents.<name>` (V2) or `agent.<name>` (V1) | `C:\Users\raosu\.config\opencode\agents\<name>.md` | Markdown + frontmatter, or JSON object; ID from path (`team/reviewer.md` → `team/reviewer`) | `description`; `mode: primary/subagent/all` (V2 default for new custom agent = `primary`) | `disabled: true` (V2) / `disable: true` (V1) — documented for built-in and JSON agents, unverified for file-defined ones | delete the `.md` or the JSON key | `opencode reload`; `opencode service restart` | [22][28][33][41] |
| Command Code | `<root>\.commandcode\agents\<name>.md`; scan covers both `${cwd}/.commandcode/agents` and `<gitRoot>/.commandcode/agents` (cwd first) | `C:\Users\raosu\.commandcode\agents\<name>.md` | Markdown + YAML frontmatter | `name` (`[a-zA-Z0-9_-]`, not `explore`/`plan`/`review`/`general`); optional `description`, `tools`, `disallowedTools`, `model`, `reasoningEffort`, `maxTurns`, `permissionMode`, `background`, `showOutput` | no flag exists (`disabledAgents` has 0 hits in `dist/cli.mjs`); narrow `tools:` or move the file | delete the `.md` (`deleteAgentDefinition`; bundled agents throw `Cannot delete bundled agent`) | **next turn** (`createAgentRegistryRefreshMod` → `onTurnStart` → `registry.refresh()`) | [35][42][44] |

### Skills

| Ecosystem | Project path | Global Windows path | Precedence roots | Format / required fields | Native disable | Remove mechanism | Hot reload | Src |
|---|---|---|---|---|---|---|---|---|
| Claude Code | `<repo>\.claude\skills\<name>\SKILL.md` | `C:\Users\raosu\.claude\skills\<name>\SKILL.md` (44 entries on this machine: 43 authored + reserved `synced`) | enterprise > personal (`~/.claude/skills`) > project (`.claude/skills`); plugin skills load as `/plugin-name:skill-name`; a skill beats a same-named `.claude/commands/*.md` | Markdown; all 20 frontmatter fields optional, `description` recommended (`name`, `description`, `when_to_use`, `argument-hint`, `arguments`, `disable-model-invocation`, `user-invocable`, `allowed-tools`, `disallowed-tools`, `model`, `effort`, `context`, `agent`, `background`, `hooks`, `paths`, `shell`, `metadata`, `license`, `compatibility`) | `skillOverrides: {"<name>": "on"/"name-only"/"user-invocable-only"/"off"}`, `disable-model-invocation: true`, or `Skill(<name>)` deny | delete the skill directory | watched (`SKILL.md` text only); a **new top-level skills dir** needs `/reload-skills` | [2][6][39] |
| Codex | `.agents/skills/<skill>/SKILL.md` in CWD, every ancestor to repo root, and `$REPO_ROOT/.agents/skills` | `%USERPROFILE%\.agents\skills\<skill>\SKILL.md` — **`.agents`, not `.codex`**; admin `/etc/codex/skills`; bundled `C:\Users\raosu\.codex\skills\.system\` | no documented winner; duplicate `name`s are **not merged**, both appear in the picker | Markdown + YAML frontmatter; `name`, `description` required; optional `scripts/`, `references/`, `assets/`, `agents/openai.yaml` (`interface.display_name`, `policy.allow_implicit_invocation`, `dependencies.tools[]`) | `[[skills.config]] path = ".../SKILL.md"` + `enabled = false` in `config.toml` | delete the skill directory | "detects skill changes automatically"; restart as fallback | [15][9][40] |
| OpenCode | `.opencode/skills/<id>/SKILL.md` (V2 also `.opencode/skill/`); compat `.claude/skills`, `.agents/skills` | `C:\Users\raosu\.config\opencode\skills\<id>\SKILL.md`; compat `~/.claude/skills`, `~/.agents/skills` | built-ins → `.claude/skills` → `.agents/skills` → `~/.config/opencode/skills` → project `.opencode/skills` → explicit `skills: []` entries (later wins) | Markdown; V2 `name` is only a label (ID = directory, case-sensitive), `description` required or the skill is not advertised; V1 `name` 1–64 chars matching `^[a-z0-9]+(-[a-z0-9]+)*$` and the directory, `description` 1–1024 | `permissions` action `skill` with `effect: "deny"`; hide-only via `metadata.opencode/autoinvoke: false` or `disable-model-invocation: true` | delete the directory (`SKILL.md` must be **all-caps**) | `opencode reload` | [23][28][33][41] |
| Command Code | `<root>\.commandcode\skills\<name>\SKILL.md` **and** `<root>\.agents\skills\<name>\SKILL.md` | `C:\Users\raosu\.commandcode\skills\<name>\SKILL.md` (44 junctions) and `C:\Users\raosu\.agents\skills\<name>\SKILL.md` (62 skills); bundled `<prefix>\node_modules\command-code\dist\bundled\<skill>` | `<cwd>/.commandcode/skills` → `<cwd>/.agents/skills` → `<gitRoot>` pair → `~/.commandcode/skills` → `~/.agents/skills` → extras (`--skill`, settings `skills[]`) → bundled; first occurrence wins | Markdown; `name` required ≤64 chars `[a-z0-9-]` equal to dir name, `description` required ≤1024; parser also reads `license`, `compatibility`, `allowed-tools`, `metadata`, `model`, `argument-hint`, `disable-model-invocation`, `when_to_use`, `user-invocable`, `disallowed-tools`, `arguments`, `effort` | `disabledSkills: ["name", …]` in `settings.json` — unioned across **user + projectShared + projectLocal**, drops the name from project *and* global lists | `cmdc skills remove <name> [--global/-g] [--yes/-y]` — resolves **only** `.commandcode/skills`, never `~/.agents/skills` | next turn, no restart | [36][42][44] |

### MCP servers

| Ecosystem | Config file and key path | Project | Global | Scope model / merge rule | CLI add / remove | Disable flag | Hot reload | Src |
|---|---|---|---|---|---|---|---|---|
| Claude Code | `<repo>\.mcp.json`, `C:\Users\raosu\.claude.json`, `C:\Program Files\ClaudeCode\managed-mcp.json`; key `mcpServers.<name>` | `<repo>\.mcp.json` (`{"mcpServers":{}}`) | `~/.claude.json` → top-level `mcpServers` (user) or `projects["<path>"].mcpServers` (local) | `local` (default, private) / `project` (committed) / `user`; plus plugin and claude.ai connectors. Precedence local > project > user > plugin > connectors, `managedMcpServers` above all (v2.1.259+); **whole entry wins, no field merging** | `claude mcp add [-s local/user/project] [-t stdio/sse/http] [-e/--env] [-H/--header] [--callback-port] [--client-id] [--client-secret] -- <cmd>`; `claude mcp add-json <name> <json>` (**only way to add `ws`**); `claude mcp remove <name> [-s <scope>]`; also `list`, `get`, `reset-project-choices`, `login`, `logout` | `disabledMcpServers`/`enabledMcpServers` in `~/.claude.json` per project; `disabledMcpjsonServers`/`enabledMcpjsonServers`/`enableAllProjectMcpServers` in settings for `.mcp.json` entries | none documented → new session or reconnect from `/mcp` | [3][4][39] |
| Codex | `config.toml` → `[mcp_servers.<id>]`; global `%USERPROFILE%\.codex\config.toml`, project `<repo>/.codex/config.toml` | `<repo>/.codex/config.toml` (trusted projects only) | `%USERPROFILE%\.codex\config.toml` | layer order (CLI → project layers → profile → user → cloud → system); server identity is the table key; admin `requirements.toml` allowlist can disable non-matching servers | `codex mcp list [--json]`, `get <NAME>`, `add <NAME> (--url <URL> or -- <COMMAND>...) [--env K=V] [--bearer-token-env-var E] [--oauth-client-id ID] [--oauth-client-secret S] [--oauth-resource R]`, `remove <NAME>`, `login`, `logout`. **No `--scope` flag** — CLI writes global `~/.codex/config.toml` only | `enabled = false` (documented "without removing its configuration"); per-tool narrowing via `enabled_tools`/`disabled_tools` | restart ("Save the server, then select Restart") | [16][9][18][40] |
| OpenCode | `opencode.json(c)` → `mcp.servers.<name>` (V2) or `mcp.<name>` (V1); global `C:\Users\raosu\.config\opencode\opencode.json(c)`, project `<repo>/opencode.json(c)` or `<repo>/.opencode/opencode.json(c)`, system `%ProgramData%\opencode\opencode.json(c)` | any project config document | `%USERPROFILE%\.config\opencode\opencode.json` | server lives in whichever document defines it; a higher-precedence document **replaces the whole server object**; managed dir cannot be overridden by settings (needs `policies`) | `opencode mcp list`, `add <name> [-- <command…>]` with `--url`, `--header key=value`, `--env key=value`, `--global`; `auth [name]`, `logout [name]`. **No `remove`/`disable` subcommand** | `"disabled": true` (V2, default `false`); V1 `"enabled": false` | `opencode reload` / `opencode service restart` | [24][25][28][29][41] |
| Command Code | `{"mcpServers":{…}}` in `C:\Users\raosu\.commandcode\mcp.json`, legacy `C:\Users\raosu\.commandcode\.mcp.json`, `<root>\.mcp.json`, `C:\Users\raosu\.commandcode\projects\<slug>\mcp.json`, plus `settings.json` → `mcp.servers` array | `<root>\.mcp.json` (committed); private default `~/.commandcode/projects/<slug>/mcp.json` | `~/.commandcode/mcp.json` | `local > project > user` (later wins); `~/.agents/mcp.json` and `<root>/.commandcode/mcp.json` are **never read** | `cmdc mcp add`, `add-json <name> '{…}'`, `list`, `get`, `remove <name>`, `auth [--status/--list/--clear]`; flags `--scope user/project/local`, `--transport`, `--env`, `--header`; writes `mode 0600`, strips OAuth secrets to `mcp-tokens.json` | `"enabled": false` **in the same file the server is defined in** | new session (clients built once per session); `/mcp` toggles affect the live session | [37][38][42][44] |

### Other artifacts

| Artifact | Claude Code | Codex | OpenCode | Command Code | Src |
|---|---|---|---|---|---|
| Memory / instructions | `CLAUDE.md` hierarchy: `~/.claude/CLAUDE.md`, `<repo>/CLAUDE.md`, `CLAUDE.local.md`, subdir `CLAUDE.md`, `AGENTS.md` may substitute | `AGENTS.md` + `AGENTS.override.md` root→CWD, global `~/.codex/AGENTS.md`, `project_doc_max_bytes` 32 KiB, `project_doc_fallback_filenames` | `AGENTS.md` only (V2); `instructions: []` globs/URLs; V1 fell back to `CLAUDE.md` | `AGENTS.md` only — **`CLAUDE.md` is not read** (`/import` rewrites the name) | [6][14][30][33][44] |
| Slash commands | `.claude\commands\*.md`, `~/.claude\commands\*.md` (merged into skills; same frontmatter minus `name`/`paths`) | **removed** — `~/.codex/prompts/*.md` dead since 0.117.0 | `.opencode/commands/<name>.md`, JSON `commands{}` (`template`, `description`, `agent`, `model`, `subagent`) | `<root>\.commandcode\commands\**\*.md`, `~\.commandcode\commands\`; name = filename, `$1`/`${@:2}` args | [2][6][21][28][44] |
| Hooks | `settings.json` → `hooks`; plugin `hooks/hooks.json`; skill/subagent frontmatter; handlers `command/http/mcp_tool/prompt/agent` | `~/.codex/hooks.json`, `<repo>/.codex/hooks.json`, or inline `[hooks]`; Windows override `commandWindows`/`command_windows` | plugin hooks only | docs `hooks.md` in the shipped bundle | [7][11][44] |
| Plugins / marketplaces | `~/.claude/plugins/` (`installed_plugins.json`, `known_marketplaces.json`, `cache/`, `marketplaces/`), `settings.json` → `enabledPlugins`, `extraKnownMarketplaces`; `claude plugin …` | `[plugins."<name>@<marketplace>"] enabled`, `[marketplaces.<name>] source_type/source/ref/sparse_paths`; `codex plugin add/remove/list --json` | `.opencode/plugins/*.ts` (local dirs, must be directories), `plugins: []` (npm, `-id` disables), load order global cfg → project cfg → global dir → project dir | mods: `~\.commandcode\mods\*.ts`, `<root>\.commandcode\mods\*.ts`; `cmdc mods add/list/remove/update/enable/disable` | [8][9][28][31][44] |
| Settings files | `.claude/settings.json`, `.claude/settings.local.json`, `~/.claude/settings.json`, `C:\Program Files\ClaudeCode\managed-settings.json` (+ `managed-settings.d\*.json`), HKLM/HKCU `SOFTWARE\Policies\ClaudeCode` value `Settings` | `.codex/config.toml`, `~/.codex/config.toml`, `~/.codex/<profile>.config.toml` | see the MCP servers matrix; V1 also probed `config.json` | `~\.commandcode\settings.json`, `<root>\.commandcode\settings.json`, `<root>\.commandcode\settings.local.json` (locks keys), `~\.commandcode\config.json` | [4][5][10][25][42] |
| Agent-scoped extras | `~/.claude/agent-memory/<agent>/MEMORY.md`, `.claude/output-styles/*.md`, `.claude/workflows/*.js`, `.claude/rules/*.md` | — | `cli.json` (`theme`, `keybinds`, `leader.timeout`); `tui.json` in V1 | `~\.commandcode\taste\` (`taste.md` + `<category>/taste.md`), `~\.commandcode\plans\*.md` | [6][27][33][42][44] |
| Permission modes | `permissionMode`: `default/acceptEdits/auto/dontAsk/bypassPermissions/plan` | `approval_policy`, `sandbox_mode`, `[windows] sandbox = "mxc"/"elevated"/"unelevated"` | `permissions: [{action,resource,effect}]` | `default/accept-edits/plan/dont-ask/yolo` | [1][9][22][44] |

## Per-ecosystem notes

### Claude Code

- Settings precedence (high→low): managed → `--settings`/CLI → `.claude/settings.local.json` → `.claude/settings.json` → `~/.claude/settings.json`. List keys (`permissions.allow`, `hooks`) **merge**; scalars take the highest level [4].
- Subagent same-name winner: managed > `--agents` > `.claude/agents/` > `~/.claude/agents/` > plugin [1].
- Skill same-name winner: enterprise > personal > project; project skill replaces a same-named bundled skill but not its aliases [2].
- Discovery asymmetry the UI must respect: subagents and skills are found by **walking up** from the working directory; settings files come from the **primary working directory only** [1][2][4]. `--add-dir` dirs also load their `.claude/agents|skills|commands`; `permissions.additionalDirectories` does not [2].
- Silent skips: no `name`; opening `---` not on line 1; invalid `name`; `name` without `description`; unparsable YAML (debug log only). Two files in one `agents/` tree with the same `name`: one wins by filesystem read order — undocumented, reported by `/doctor` [1].
- Reserved skill dirs: `synced` (skipped if authored), `anthropic-skills` (never loads outside a plugin). Names containing zero-width/control chars, private-use glyphs, RTL marks, non-single spaces or `<`/`>` break loading [2].
- A skill folder containing `.claude-plugin/plugin.json` loads as plugin `<name>@skills-dir` and needs workspace trust in a project [2].
- MCP: missing `type` means `stdio`, so a `url` entry without an explicit type is a config error and is skipped; `streamable-http` is an alias of `http`; `"type": "sdk"` entries are skipped; server names must be letters/digits/`-`/`_` and built-ins (`workspace`, `claude-in-chrome`, `computer-use`, `Claude Preview`, `Claude Browser`) are rejected [3].
- Three **independent** MCP off-switches: per-project `disabledMcpServers`/`enabledMcpServers` in `~/.claude.json`; `disabledMcpjsonServers`/`enabledMcpjsonServers`/`enableAllProjectMcpServers` for `.mcp.json`; deletion [3].
- Windows: the "put `settings.local.json` at the git root" rule is explicitly skipped, so the file stays beside `.claude/settings.json`; `C:\ProgramData\ClaudeCode\managed-settings.json` is legacy and **not** read [4][5]. Hooks default to `bash` or **PowerShell when Git Bash is absent**; set `"shell": "powershell"` and matcher `Bash|PowerShell`; a failing `PreToolUse`/`PermissionRequest` hook can block the tool call [7].
- Trust is per-folder (`hasTrustDialogAccepted` in `~/.claude.json`), gates `.mcp.json` approvals, project subagent frontmatter hooks, inline MCP servers in project agent files and skills-dir plugins; deny rules apply without trust, allow rules wait for it [1][3][4].

### Codex

- Precedence (high→low): CLI flags / `-c key=value` → project `.codex/config.toml` layers (root→CWD, closest wins) → `~/.codex/<profile>.config.toml` → `~/.codex/config.toml` → cloud-managed → `/etc/codex/config.toml` → built-ins [10].
- **The whole project layer is gated on trust**: `[projects.'<path>'] trust_level = "trusted"` in the *user* config; untrusted projects skip config, hooks and rules. Project keys are stored lowercased on disk even when the real path is mixed-case [9][11][40].
- Keys a project config may not set (ignored with a startup warning): `openai_base_url`, `chatgpt_base_url`, `apps_mcp_product_sku`, `model_provider`, `model_providers`, `notify`, `profile`, `profiles`, `experimental_realtime_ws_base_url`, `otel` [11][9].
- Merge is key-by-key; `shell_environment_policy.filters` keys merge case-insensitively but `filter`/`exclude`/`include_only` forms must not be mixed in one layer. Relative paths in a project config resolve against the `.codex/` folder holding it [11].
- Built-in agents: `default`, `worker`, `explorer`. A custom agent whose `name` matches a built-in **takes precedence**. Scalar `[agents]` setting names are reserved and cannot be used as role names [13][9].
- Ignored inside an agent TOML (inherited from the parent instead): `sandbox_mode`, `approval_policy`, `model_provider`, `mcp_servers`, service tier [13].
- Skills: duplicate `name`s are not merged; `skills.max_context_tokens` bounds the catalog (default 2% of context, capped at 10000); implicit invocation can be suppressed per skill via `agents/openai.yaml` → `policy.allow_implicit_invocation: false` [15].
- MCP: `env_vars` (an allowlist of names to forward) and `[mcp_servers.<id>.env]` (explicit values) are independent and used together in one entry. Full field set: `command`, `args`, `env`, `env_vars`, `cwd`, `url`, `bearer_token_env_var`, `http_headers`, `env_http_headers`, `http_headers_helper`, `auth`, `oauth.*`, `startup_timeout_sec`/`_ms`, `tool_timeout_sec`, `enabled`, `required`, `enabled_tools`, `disabled_tools`, `default_tools_approval_mode`, `tools.<tool>.approval_mode`, `tools.<tool>.output_token_limit`, `experimental_environment` [9].
- `codex mcp list` on the inspected machine shows four servers (`codex_app` disabled, `cua_repl`, `node_repl`, `rea` enabled) — the **merged effective view**, not a dump of `config.toml` [40].
- Unknown keys are silently tolerated by default; `--strict-config` is the opt-in that errors on unrecognised fields, so a mistyped `enabled` key silently leaves a server enabled [18].
- Windows: TOML backslash escaping is a live trap — both `command = 'C:\Users\…\node_repl.exe'` (literal string) and `notify = [ "C:\\Users\\…", "turn-ended" ]` appear in one file; a naive `"C:\Users\..."` basic string breaks on `\U`. The Windows app and native CLI share `%USERPROFILE%\.codex`; a CLI inside WSL does **not** share config/auth/sessions unless `CODEX_HOME=/mnt/c/Users/<user>/.codex` is exported [19][20][40].

### OpenCode

- Precedence V2 (low→high): `~/.config/opencode/opencode.json(c)` → ancestor project `opencode.json(c)` farthest→nearest → `.opencode/opencode.json(c)` (every discovered `.opencode` overrides every direct config) → `/etc/opencode` / `%ProgramData%\opencode` → macOS MDM `ai.opencode.managed` [25]. V1 has 8 tiers including remote `.well-known/opencode` and `OPENCODE_CONFIG_CONTENT` [33].
- Merge rules: objects deep-merge; `mcp.servers.<name>` is **replaced whole**; `plugins[]`, `skills[]` and `instructions[]` are **combined**; agent permission rules append while later scalars replace earlier ones [23][24][25][28].
- V1/V2 may mix at top level (V2 value wins) but **not nested**: inside one `agents`/`providers`/`commands`/`models` entry keep a single format. A Markdown agent file using any of `permissions`, `request`, `system`, `disabled` is parsed as V2 and its V1 fields (`name`, `tools`, `permission`, `temperature`) are ignored with a warning [28].
- Aliases and removed keys: `agents`/`agent`, `system`/`prompt`, `disabled`/`disable`, `permissions`/`permission`+`tools`, `commands`/`command`, `providers`/`provider`, `plugins`/`plugin`, `references`/`reference`. Fields ignored in V2: `logLevel`, `server`, top-level `subagent_depth`, agent `name`, an MCP entry with no `type`, `lsp` execution, V1 provider `id`/`whitelist`/`blacklist` [28][25].
- `subagent_depth` defaults to 1 (subagents cannot spawn subagents); in V2 it lives under `experimental.subagent_depth` [33][25]. Built-ins on 2.1.4: `build`, `plan` (primary), `general`, `explore` (subagent), plus hidden `compaction`, `summary`, `title` [41].
- V1 probed three filenames in the global dir in order — `config.json`, `opencode.json`, `opencode.jsonc` [41]. Skills: project sources are searched CWD→project root including every level, scanning stops six directories below a source root, hidden dirs and `node_modules` skipped [23].
- Managed settings cannot be beaten by settings; enforcing a restriction needs `policies` (`provider.use`, `tool.use`, `integration.use`) — the app must render "locked" state rather than offer an edit it cannot win [25][28].

### Command Code

- Root resolution: git root, else walk up ≤10 levels for `.commandcode`, else cwd (`resolveProjectDir`, `walkUpToFindDir` with `_a=10`); home = `HOME ?? USERPROFILE` [42].
- Layered settings: `settings.json` (user) → `<root>\.commandcode\settings.json` (project shared) → `<root>\.commandcode\settings.local.json` (project local, **locks** keys — `cmdc config set` refuses with "… is locked by settings.local.json — edit that file to change it.") [34][42][44].
- **Precedence asymmetry:** skills resolve project > personal and `.commandcode` > `.agents` (first occurrence wins, shadowed copies reported as "Duplicate names" in `/skills` and `cmdc skills list --debug`); agents resolve bundled → personal → project with **first definition winning, i.e. personal shadows project** — the reverse of skills [36][35].
- Reserved agent names `["explore","plan","review","general"]`; a custom file using one is ignored. Project skill roots are scanned **only when the workspace is trusted** [42][36].
- Disabling a skill name disables **every copy** of that name (`filterDisabledSkills` drops it from both project and global lists); `setEnabled` keeps the array sorted and deletes the key when empty [42].
- `cmdc skills remove` only resolves `~\.commandcode\skills` or `<project>\.commandcode\skills` and `fs.rm`s the directory — on this machine every entry in `~\.commandcode\skills` is a **junction** to `C:\Users\raosu\.agents\skills\<name>`, so removing leaves the `.agents` original live and the skill reappears with an `[.agents]` badge [42].
- `~\.agents\.skill-lock.json` is **not** a Command Code file (0 hits for `.skill-lock`/`skillFolderHash`); its `lastSelectedAgents` naming 16 agents identifies it as an external multi-agent installer's state. `~\.agents\agents` and `~\.agents\mcp.json` are likewise never read [42].
- MCP loader order (later wins) is `settings.json` → `mcp.servers` (`includeProjectServers === false` only) → `mcp.json` → `.mcp.json` → `projects/<slug>/mcp.json`. `transport` is the field name, with `type` accepted as an alias (`inferTransport`); validation requires http/sse `url` or stdio `command`, boolean `enabled`, `Record<string,string>` `headers`/`env`; `${VAR}` / `${VAR:-default}` resolve at runtime, `$${VAR}` escapes [37][42].
- Tool naming `mcp__<server>__<tool>`. `readRawMcpConfigForWrite` refuses to touch a file that is not valid JSON or lacks the `{"mcpServers":{…}}` shape ("… so it was left untouched") [37][42].
- Windows: **`cmd` collides with the shell** — docs say read `cmd` as `cmdc`; shell out to `cmdc`, `command-code`, or `node <prefix>\node_modules\command-code\dist\index.mjs`. Auto-update is on by default, so `dist\bundled` can change under the app between releases [38][42].

## Conflicts

Where the four reports found the sources disagreeing, or the docs disagreeing with the shipped code. None of these were resolved by testing; the stance below is the one the reports took (or "unresolved" where they did not).

| # | Conflict | Where | Stance | Src |
|---|---|---|---|---|
| 1 | Codex user-scope skills: official discovery table says `$HOME/.agents/skills`, a GitHub commenter claims Codex also reads `.codex/skills` | Codex | Official docs treated as authoritative; `.agents\skills` is the only documented root | [15][21] |
| 2 | OpenCode MCP `timeout` default: V2 docs say 30 s startup / 30 s catalog / 12 h execution, the published schema's `McpLocalConfig` says "Defaults to 5000" | OpenCode | Unresolved; do not hard-code a default in the UI | [24][29] |
| 3 | Command Code reads `.agents/skills` but **not** `.agents/mcp.json` — the `.agents` compat surface is asymmetric between artifact types | Command Code | Code behaviour trusted: user MCP lives in `~\.commandcode\mcp.json` | [37][42] |
| 4 | Command Code skill precedence (project > personal, `.commandcode` > `.agents`) is the reverse of agent precedence (personal > project); docs also place `--skill`/`skills[]` extras at step 5 while the code appends them after the user roots | Command Code | Unresolved; surface the ambiguity rather than silently reordering | [35][36][42] |
| 5 | Codex documents two ways to declare an agent — standalone `agents/*.toml` and inline `[agents.<name>]` + `config_file` — with no stated precedence when both define one `name` | Codex | Unresolved; the report recommends supporting standalone files and showing inline tables read-only | [13][9] |
| 6 | Codex says skills are "detected automatically" but also "Restart Codex after changing `~/.codex/config.toml`" — and `[[skills.config]]` disables live in that file | Codex | Restart-required for skill *disable/enable*; automatic for `SKILL.md` edits | [15] |
| 7 | Claude Code docs say `~/.claude.json` is internal ("you don't need to edit it"), yet it is the only home for `local` and `user` scope MCP servers | Claude Code | The app must write it for local/user scope, atomically and defensively | [3][4] |
| 8 | Cross-ecosystem (not a source conflict): skill precedence runs **personal/enterprise over project** in Claude Code, **project over global** in OpenCode and Command Code | all | The UI cannot show one "effective skills" model; resolve per ecosystem | [2][23][36] |

## Disable vs remove

Prefer the toggle. Deletion from a manager UI is irreversible from the user's side and destroys authored content; every ecosystem except Codex subagents and Command Code subagents offers a documented non-destructive off-switch.

| Ecosystem | Artifact | Non-destructive mechanism | Where the app writes | Reversible | Src |
|---|---|---|---|---|---|
| Claude Code | Subagent | `permissions.deny: ["Agent(<name>)"]`; per-invocation `--disallowedTools "Agent(<name>)"` | any `settings*.json` (project preferred) | yes — remove the deny rule | [1] |
| Claude Code | Skill | `skillOverrides: {"<name>": "off"}` (`on`/`name-only`/`user-invocable-only`/`off`); hide-only `disable-model-invocation: true` | `settings.json`; `/skills` writes to `.claude/settings.local.json` | yes | [2] |
| Claude Code | MCP | per-project `disabledMcpServers` / `enabledMcpServers` (disjoint lists); `disabledMcpjsonServers` for `.mcp.json` entries | `~/.claude.json` → `projects["<path>"]`; or `settings*.json` | yes | [3] |
| Claude Code | Hooks | `disableAllHooks` | `settings*.json` | yes | [6][7] |
| Codex | Skill | `[[skills.config]]` `path` + `enabled = false` | `~/.codex/config.toml` | yes | [15][9] |
| Codex | MCP | `enabled = false`; narrower `enabled_tools`/`disabled_tools` | the `[mcp_servers.<id>]` table, same file it lives in | yes | [16][9] |
| Codex | Plugin | `[plugins."<name>@<marketplace>"] enabled = false` | `config.toml` | yes | [9] |
| Codex | All subagents | `agents.enabled = false` or `features.multi_agent = false` | `config.toml` | yes | [13][9] |
| Codex | Single subagent | **none — needs file move** out of `agents/` (or delete) | — | no (unless moved back) | [13] |
| OpenCode | Agent | `disabled: true` (V2) / `disable: true` (V1) on the JSON entry or built-in override | `opencode.json(c)`; unverified for file-defined agents — verify with `opencode debug agents` | yes | [22][33] |
| OpenCode | Skill | permission action `skill` + `effect: "deny"`; hide-only `metadata.opencode/autoinvoke: false` | `permissions` array (config doc or agent file) | yes | [23] |
| OpenCode | MCP | `"disabled": true` (V2) / `"enabled": false` (V1) | the document that defines the server | yes | [24][33] |
| Command Code | Skill | `disabledSkills: ["name"]` (union of three settings files) | `settings.json` (user for global, project for project) | yes | [36][42] |
| Command Code | Agent | **none — needs file move** (`disabledAgents` absent); narrowing `tools:` also possible | — | no (unless moved back) | [42] |
| Command Code | MCP | `"enabled": false` in the same file the server came from | that scope file (`setMcpServerEnabled` rewrites it) | yes | [37][42] |

## Write safety

- **Codex TOML comment loss is observed, not theoretical.** The live `C:\Users\raosu\.codex\config.toml` has no comments at all, long runs of blank lines where commented keys used to be, and a sibling `config.toml.rea.backup`; the CLI persists settings into the file (`codex features enable|disable`, `/keymap`, `/statusline`, `/title`, `/theme`, `/experimental`) [40][18]. A naive TOML round-trip silently strips user annotations. Serialize with a real encoder, merge into the existing document, never string-concatenate or replace the file [9][40].
- **OpenCode JSONC comment loss.** The published schema sets `allowComments: true` and `allowTrailingCommas: true` and V2 reads `.jsonc`; `JSON.parse` → `JSON.stringify` destroys comments and formatting. Use a comment-preserving parser, or refuse to write when comments are detected and show a diff. The root schema is `additionalProperties: false`, so one misspelled top-level key invalidates the document — validate against `https://opencode.ai/config.json` (draft 2020-12, `$ref`s `https://models.dev/model-schema.json` for model fields) [29][28][25].
- **Claude Code `~/.claude.json` is the dangerous file.** It is app state (per-project MCP, trust, history) that Claude Code itself writes and docs say you need not edit [4]. Unparsable content produces a Configuration error; Claude Code copies it to `~/.claude/backups/.claude.json.corrupted.<ts>` and offers a reset, and keeps the **five** most recent `.claude.json.backup.<ts>` — exactly 5 exist on this machine [4][6][39]. Settings files are strict JSON: a `//` comment or trailing comma is a Settings Error; a schema-rejected single value is a Settings Warning and is skipped [4].
- **Command Code `settings.json` fails silently.** Invalid JSON is caught and read as `{}`, so every user setting is ignored until fixed, while `cmdc config set` refuses to write it. A stray key in `settings.local.json` locks that key. `mcp.json` with bad JSON/shape makes `cmdc mcp add/enable` refuse to write at all [42][44].
- **Windows EPERM/EBUSY on rename.** Community issues report concurrent Claude Code sessions hitting `EBUSY`/`EPERM` writing `~/.claude.json` because `renameSync` fails while another process holds the target; not addressed in official docs [43]. Command Code already uses `write(temp) → rename(dest)` with 0600 (`tempPathFor`, `writeCommandAuthFileObject` `mode:384`), treats `EPERM` as "process alive"/"permission denied", and has **no retry loop** — a locked file can surface a raw rename error [42]. Practical contract for the app: write temp + `File.Move(replace)`, back off on `EPERM`/`EBUSY`, never truncate-then-write, never hold a handle on `~/.commandcode\*.json` or `~/.claude.json` while the tool starts [42][43].
- **Do not write what another process owns.** OpenCode's `service.json` (contains `password`), `opencode.db` (+ `-wal`, 105 MB here), `log/`, `snapshot/`, `storage/`, `tool-output/` are read-only at most [41]. Claude Code's `~/.claude.json` and Command Code's `sessions/`, `history.jsonl`, `file-history/`, `auth.json`, `telemetry-install-id` are tool-owned [6][42].
- **Prefer the vendor CLI** where it exists and validates: `claude mcp add|add-json|remove`, `codex mcp add|remove`, `opencode mcp add`, `cmdc mcp add|add-json|remove`, `cmdc skills remove`, `codex plugin add|remove`. Each normalises fields the app would otherwise have to guess (`type` → `transport` in Command Code, `enabled` defaulting, OAuth secret stripping, `0600`), and each keeps the tool's own file shape [3][18][41][42]. The one case with no CLI is Codex project-scope MCP, which must be hand-written into `<repo>/.codex/config.toml` [16].
- **Preserve unknown content.** Command Code's own writer preserves unknown top-level keys and rewrites with 2-space indent [42]; OpenCode V2 never rewrites the source file, so a V1-shaped file stays V1-shaped on disk while the app "sees" V2 values — write back the shape you read [25][28].
- **Encoding.** Read/write `.md`/`.ts`/`.json` as UTF-8; PowerShell 5.1 `Get-Content -Raw` defaults to ANSI and mangles non-ASCII [41].

## Hot reload / restart matrix

| Ecosystem | Artifact | Behaviour | Src |
|---|---|---|---|
| Claude Code | `settings.json` / `settings.local.json` / managed file | watched; most edits apply live including `permissions` and `hooks`; `model`, `effortLevel`, `modelSettings` are read once at start | [4] |
| Claude Code | `~/.claude/agents/`, `.claude/agents/` | watched, few seconds. Restart required when: the first file lands in a **new** `agents/` dir, the dir is in an `--add-dir` path, or the session used `--disable-slash-commands` | [1] |
| Claude Code | Skills dirs | watched for `SKILL.md` text; a new top-level skills dir needs `/reload-skills` | [2] |
| Claude Code | Plugins | `/reload-plugins` (`--force` when MCP tools change); plugin `hooks/`, `.mcp.json`, `agents/`, `output-styles/` need it | [2][8] |
| Claude Code | `.mcp.json` / `~/.claude.json` mcpServers | no documented watcher → new session or reconnect from `/mcp` | [3] |
| Codex | `config.toml` (incl. `[mcp_servers]`, `[[skills.config]]`, agents) | restart ("Restart Codex after changing `~/.codex/config.toml`"; app: "Save the server, then select Restart") | [15][16] |
| Codex | `AGENTS.md` chain | rebuilt every run; "there is no cache to clear" | [14] |
| Codex | Skills | detected automatically, restart as fallback | [15] |
| Codex | `agents/*.toml` | loaded with session config at start; no watcher documented | [13] |
| OpenCode | config / agents / skills / MCP | `opencode reload` ("Reload configuration"; `--standalone`/`--server`) | [41] |
| OpenCode | plugin dirs / watched config dirs | reload automatically; otherwise `opencode service restart` | [31] |
| OpenCode | `cli.json` | "Valid edits reload while the TUI is running" | [27] |
| OpenCode | `AGENTS.md` (global and upward) | re-read before the next model request; nested ones are not | [30] |
| Command Code | agent file add/edit/delete | **next turn** (`onTurnStart` registry refresh) | [42][35] |
| Command Code | skill add/edit/delete | next turn, no restart; catalog rebuilt per turn | [36] |
| Command Code | `disabledSkills` write | guaranteed next session; a running session re-reads on `/skills` redraw | [36] |
| Command Code | `AGENTS.md` | next request (system prompt rebuilt each request) | [44] |
| Command Code | MCP on disk | **new session** (clients built once per session); `/mcp` toggles act on the live session | [37][42] |
| Command Code | mods | process lifetime; `/reload` re-imports | [44] |

## V2 creation

| Ecosystem / artifact | Canonical layout | Minimum fields | CLI | Where the app writes | Validation | Src |
|---|---|---|---|---|---|---|
| Claude Code subagent | `<scope>\agents\<name>.md`, `<scope>` = `<repo>\.claude` or `C:\Users\raosu\.claude` | `name`, `description`; optional `tools` (or `mcp__<server>__*`), `model`, `permissionMode`, `maxTurns`, `skills`, `mcpServers`, `hooks`, `memory`, `background`, `effort`, `isolation`, `color`, `initialPrompt` | none (`/agents` only prints a reminder ≥2.1.198) | `agents\` dir; create it first for global scope (it does not exist here) | `claude plugin validate .claude\agents` (component-dir mode needs v2.1.233+), or `claude --debug` → `~/.claude/debug/<session-id>.txt` grep `does not recognize or apply` (v2.1.296+) | [1][8][39] |
| Claude Code skill | `<scope>\skills\<name>\SKILL.md` + optional `reference.md`, `scripts/` | `name`, `description` (all frontmatter optional; keep `SKILL.md` under 500 lines) | none | `skills\<name>\` | `/skills` list, `claude --debug` | [2][39] |
| Claude Code MCP | `<repo>\.mcp.json` / `~/.claude.json` | stdio: `command`; http/sse/ws: `type` + `url`; optional `args`, `env`, `headers`, `headersHelper`, `oauth`, `timeout`, `alwaysLoad` | `claude mcp add` / `add-json`; `claude mcp get <name>` shows ✔ Connected / ! Needs authentication / ✘ Failed to connect / ⏸ Pending approval | scope per intent: `--scope project` = `.mcp.json`, `local` = `projects["<path>"]`, `user` = top-level | `claude mcp get`, `claude mcp list` | [3] |
| Codex subagent | `<repo>/.codex/agents/<name>.toml` or `%USERPROFILE%\.codex\agents\<name>.toml` | `name`, `description`, `developer_instructions`; optional `model`, `model_reasoning_effort`, `model_reasoning_summary`, `model_verbosity`, `features.*`, `skills.config`, `skills.bundled.enabled`, `skills.include_instructions` | none (`codex agents` only browses sessions) | `agents\`; filename is cosmetic, `name` is identity | none documented beyond loading; check the picker | [13][40] |
| Codex skill | `%USERPROFILE%\.agents\skills\<skill>\SKILL.md` or `<repo>/.agents/skills/<skill>/SKILL.md` | `name`, `description`; optional `scripts/`, `references/`, `assets/`, `agents/openai.yaml` | none documented — there is no Codex CLI command for authoring skills | **`.agents\skills`, never `.codex\skills`** | import + non-empty `name`/`description`; duplicate names are allowed | [15] |
| Codex MCP | `[mcp_servers.<id>]` in `config.toml` | `command` (stdio) or `url` (HTTP); rest optional, incl. `startup_timeout_sec`, `enabled` | `codex mcp add …` for global; **no `--scope`**, so project scope is hand-written | global via CLI; `<repo>/.codex/config.toml` for project (repo must be trusted) | `codex mcp list --json`, `codex mcp get`, `codex --strict-config`, `codex doctor`, `/debug-config` | [16][18] |
| OpenCode subagent | `.opencode/agents/<name>.md` or `%USERPROFILE%\.config\opencode\agents\<name>.md`; or `agents.<id>` JSON | `description`, `mode: subagent` (+ body). Optional `model` (`provider/model` + `#variant`), `system`, `permissions`, `steps`, `hidden`, `color`, `disabled`, `request` | none | `agents\` | `opencode debug agents` (JSON array) | [22][28][41] |
| OpenCode skill | `<root>\<name>\SKILL.md` where `<root>` = `.opencode\skills` or `%USERPROFILE%\.config\opencode\skills` | `name`, `description`; optional `metadata.opencode/autoinvoke`, `license`, `compatibility` | none | `skills\<id>\`; register out-of-tree roots via `skills: []` | `opencode debug config`; plugin `ctx.skill.list()` for live state | [23][32][41] |
| OpenCode MCP | `mcp.servers.<name>` in `opencode.json(c)` | V2 local: `type:"local"` + `command` array; V2 remote: `type:"remote"` + `url`; optional `environment`, `cwd`, `headers`, `disabled`, `oauth`, `timeout`, `codemode`, `protocol` | `opencode mcp add …` (writes project config, `--global` for user) | the document matching the requested scope | `opencode mcp list`, `opencode debug config` | [24][41] |
| Command Code subagent | `<scope-root>\agents\<name>.md`; honour `<gitRoot>\.commandcode\agents` too | `name` (required), `description`, `tools`, `disallowedTools`, `model`, `reasoningEffort`, `maxTurns`, `permissionMode`, `background`, `showOutput` | none | `~\.commandcode\agents` (global) or `<root>\.commandcode\agents` (project) | loads next turn; invalid frontmatter warns on the feed (1.38.2+) | [35][42] |
| Command Code skill | `<scope-root>\skills\<name>\SKILL.md` + `scripts\`, `references\`, `assets\` | `name`, `description` (name = dir, `[a-z0-9-]` ≤64; description ≤1024) | `cmdc skills add <owner/repo> [-g] [-f] [-s <name>]`; authoring via bundled `skill-builder` | **one root only** — `.commandcode\skills` (or `.agents\skills`), never mirrored | `cmdc skills list --debug`, `/skills` issues view | [36][42] |
| Command Code MCP | `~\.commandcode\mcp.json` (global), `<root>\.mcp.json` (project), `~\.commandcode\projects\<slug>\mcp.json` (local) | `transport` (`stdio`/`http`/`sse`, `type` accepted), `command`+`args` or `url`, optional `enabled`, `headers`, `env`, `oauth` | `cmdc mcp add [--scope …]`, `cmdc mcp add-json <name> '{…}'` | the scope file the CLI would write | `readRawMcpConfigForWrite` shape check; `/mcp`, `cmdc mcp get` | [37][38][42] |

**Trap cases found**

- **Codex user skills live in `.agents\skills`, not `.codex\skills`.** `%USERPROFILE%\.codex\skills` on this machine holds only bundled `.system` skills (`imagegen`, `openai-docs`, `review-agent`, `skill-creator`, `skill-installer`); writing user skills there is off-spec [15][40].
- **Command Code must not be written to both skill roots.** It reads `.commandcode\skills` and `.agents\skills` and `.commandcode` wins, so a mirror just creates a shadowed copy and two files to keep in sync; the 44 junctions on this machine are an external installer's artifact, not a requirement [42].
- **OpenCode has no `mcp remove`.** Its subcommands are exactly `list`, `add`, `auth`, `logout`; deletion means editing the `mcp.servers.<name>` entry out of the right document [41].
- **Claude Code `ws` transport cannot be added with `--transport`.** Use `claude mcp add-json`; WebSocket has no OAuth and is header-only [3].
- **Codex has no `--scope` flag** (at 0.164.0-alpha.11.1), so project-scope MCP creation must be a hand-written `[mcp_servers.<id>]` table in `<repo>/.codex/config.toml`, in a trusted repo, without the forbidden project keys [16][11].
- **OpenCode agent files must be one format.** Mixing V1 and V2 keys inside a single agent file silently drops the V1 subset [28].

## Version gating

| Feature | Ecosystem | Minimum / boundary | Src |
|---|---|---|---|
| `claude plugin validate <component-dir>` for a bare `.claude\agents` | Claude Code | v2.1.233+ (installed build is 2.1.215) | [1][8] |
| `omitClaudeMd` subagent field | Claude Code | v2.1.271+ | [1] |
| `autoCompactWindow` subagent field (100000–1000000) | Claude Code | v2.1.296+ | [1] |
| `managedMcpServers` ranked above all MCP scopes | Claude Code | v2.1.259+ | [3] |
| Auto-fallback from HTTP to SSE | Claude Code | v2.1.265+ | [3] |
| MCP tasks (public beta) | Claude Code | v2.1.304+ | [3] |
| `/agents` interactive wizard | Claude Code | v2.1.197 and earlier | [1] |
| Custom prompts `~/.codex/prompts/*.md` | Codex | **removed in 0.117.0** — skills are the replacement | [21][17] |
| `[profiles.<name>]` table / top-level `profile =` no longer read | Codex | since 0.134.0 | [11] |
| Managed permission-profile allowlists | Codex | 0.138.0+; ≤0.137.0 ignore them | [9] |
| WSL1 support / bubblewrap Linux sandbox | Codex | supported through 0.114 / introduced 0.115 | [20] |
| V2 config shape (`agents`, `mcp.servers`, `disabled`) | OpenCode | 2.x; V1 shapes still read on 2.1.4 | [25][28][41] |
| `.agents/skills` compatibility | Command Code | 0.18.9+ | [42] |
| `/agents` overhaul, per-agent models, bundled `skill-builder`/`mod-builder`/`command-code-knowledge` | Command Code | 1.0.0 | [42] |
| Agent frontmatter warnings on the feed | Command Code | 1.38.2+ | [42] |
| Builds inspected | all | Claude Code 2.1.215; Codex 0.164.0-alpha.11.1; OpenCode 2.1.4 (older 1.1.65 on disk); command-code 1.88.0 | [39][40][41][42] |

## Open questions / unverified

| # | Ecosystem | Question | Why it matters to the app |
|---|---|---|---|
| 1 | Claude Code | `.mcp.json` has no documented watcher; whether MCP config changes hot-reload at all | Advertise "restart/reconnect", not live refresh |
| 2 | Claude Code | `.mcp.json` ancestor walk from a nested cwd is unstated | Project-scope discovery in a monorepo |
| 3 | Claude Code | `EBUSY`/`EPERM` on `~/.claude.json` is community-reported only (issues 25768, 46482) | Retry/backoff policy needs empirical confirmation |
| 4 | Claude Code | Component-dir `plugin validate` and `/reload-skills` untested on 2.1.215 | Validation UX may be unavailable on the installed build |
| 5 | Claude Code | Subagent "new directory ⇒ restart" not observed (no `~/.claude/agents/` existed) | First global agent creation may need a restart |
| 6 | Claude Code | Desktop-app parity of agent/skill/MCP discovery is unobserved | Cross-check before claiming parity |
| 7 | Codex | `[agents.<name>]` inline tables vs standalone `agents/*.toml`: precedence and whether `config_file` may point at a standalone file | Two documented ways to declare one agent |
| 8 | Codex | Duplicate agent `name` across global and project scope — no documented winner (skills explicitly do not dedupe) | Which row to show as active |
| 9 | Codex | Windows equivalents of `/etc/codex/config.toml`, `/etc/codex/skills` and the system `requirements.toml` path | Admin-scope surface cannot be implemented yet |
| 10 | Codex | Hot reload of `config.toml`, `agents/*.toml`, `[mcp_servers]` untested by mutation | Same as Claude Code #1 |
| 11 | Codex | `[projects.*]` key normalisation (lowercasing, `\\?\` prefixes) observed but undocumented | Trust lookups keyed on raw casing will miss |
| 12 | Codex | Whether `codex mcp add` rewrites or appends, and whether it preserves comments | Determines whether the app may use the CLI on an annotated file |
| 13 | Codex | Whether `codex mcp remove` also deletes stored OAuth credentials | Secret residue after "remove" |
| 14 | Codex | Whether a per-agent `enabled = false` is accepted (assumed unsupported) | Would enable per-subagent toggling |
| 15 | Codex | `.codex/skills` vs `.agents/skills` for user scope: a GitHub commenter claims `.codex/skills` is read, conflicting with the official table | Resolved by treating official docs as authoritative, but should be re-tested |
| 16 | Codex | Exact filename/locations of `requirements.toml` | Managed-policy display |
| 17 | OpenCode | Whether `agents.<id>.disabled: true` disables a **file-defined** agent of the same ID | Safe V1 action is currently file move — see "Disable vs remove" |
| 18 | OpenCode | Rename semantics for agents and skills (ID derived from path) | Rename = delete + create; references (`permissions[].resource`, `default_agent`) must be rewritten |
| 19 | OpenCode | MCP `timeout` default: 30 s startup / 30 s catalog / 12 h execution (docs) vs "Defaults to 5000" in the schema's V1 section | Form defaults |
| 20 | OpenCode | V2 location of MCP OAuth credentials (V1 named `~/.local/share/opencode/mcp-auth.json`) | Logout/credential cleanup |
| 21 | OpenCode | Whether `opencode.json(c)` reloads automatically (documented only for plugin dirs, `cli.json`, `AGENTS.md`) | Whether to call `opencode reload` |
| 22 | OpenCode | `service.json` portability outside Windows; whether `mcp auth`/`logout` state is per-scope or per-user | Secret handling |
| 23 | OpenCode | `opencode debug skill` existed in V1 docs but 2.1.4 exposes only `debug agents/config/paths/heap-dump` | Skill enumeration must read the filesystem or use `ctx.skill.list()` |
| 24 | OpenCode | On-disk formatting produced by `opencode mcp add` (rewrite, key sorting, comment preservation) | Same class as Codex #12 |
| 25 | OpenCode | `%ProgramData%\opencode` behaviour — documented, absent on this machine | Managed/locked-state UI |
| 26 | Command Code | Authorship of `~\.agents\mcp.json` (not read as MCP, but `.agents\skills` is) | Which file the app should treat as user MCP |
| 27 | Command Code | Write-race semantics for `settings.json` with a live session (no lock file found; only the cron scheduler takes a lease) | Concurrent-write safety |
| 28 | Command Code | Whether a disabled-but-shadowed skill still lists as disabled (`filterDisabledSkills` runs before shadowing resolution) | Status accuracy in the UI |
| 29 | Command Code | Global `~\.commandcode\taste\` layout unobserved | Taste management is V2-scope only |
| 30 | Command Code | `--skill` / settings `skills[]` precedence against `.commandcode/skills` (code pushes extras after user roots, docs list them at step 5) | Ordering of extra roots |
| 31 | Command Code | `/docs/taste`, `/docs/settings` cited from in-package links but not fetched; `vsix/` payload uninspected | Doc gaps for two surfaces |

## Sources

**Official docs — Claude Code**

1. https://code.claude.com/docs/en/sub-agents — scope, frontmatter, skip rules, hot reload, disable, `plugin validate`
2. https://code.claude.com/docs/en/skills — locations, frontmatter, precedence, `skillOverrides`, live change detection
3. https://code.claude.com/docs/en/mcp — schema, scopes, CLI, precedence, disable, status, trust, stdio shutdown
4. https://code.claude.com/docs/en/settings — four settings files, precedence, strict JSON, backups
5. https://code.claude.com/docs/en/managed-settings — per-OS paths, registry keys, legacy path
6. https://code.claude.com/docs/en/claude-directory — full file inventory, app data
7. https://code.claude.com/docs/en/hooks — locations, handler fields, PowerShell failures
8. https://code.claude.com/docs/en/plugins/cli-reference — `claude plugin validate|disable|enable|list`, `/reload-plugins`

**Official docs — Codex** (these 302-redirect to `learn.chatgpt.com/docs/*`; appending `.md` returns markdown)

9. https://developers.openai.com/codex/config-file/config-reference — incl. `config-schema.json`
10. https://developers.openai.com/codex/config-file/config-basic
11. https://developers.openai.com/codex/config-file/config-advanced
12. https://developers.openai.com/codex/config-file/environment-variables
13. https://developers.openai.com/codex/agent-configuration/subagents
14. https://developers.openai.com/codex/agent-configuration/agents-md
15. https://developers.openai.com/codex/build-skills
16. https://developers.openai.com/codex/extend/mcp
17. https://developers.openai.com/codex/customization/overview
18. https://developers.openai.com/codex/developer-commands
19. https://developers.openai.com/codex/windows/windows-app
20. https://developers.openai.com/codex/windows/wsl
21. https://github.com/openai/codex/issues/15941 — custom prompts removed in 0.117.0

**Official docs — OpenCode** (V1 pages carry a "New OpenCode v2 is available" banner; V2 is current)

22. https://opencode.ai/v2/docs/agents
23. https://opencode.ai/v2/docs/skills
24. https://opencode.ai/v2/docs/mcp-servers
25. https://opencode.ai/v2/docs/config
26. https://opencode.ai/v2/docs/cli
27. https://opencode.ai/v2/docs/cli/config
28. https://opencode.ai/v2/docs/migrate-v1
29. https://opencode.ai/config.json — published schema, draft 2020-12, V1 keys `deprecated: true`
30. https://opencode.ai/v2/docs/instructions
31. https://opencode.ai/v2/docs/plugins
32. https://opencode.ai/v2/docs/build/plugins
33. V1 pages: https://opencode.ai/docs/config/, /docs/agents/, /docs/skills/, /docs/mcp-servers/, /docs/cli/, /docs/plugins/, /docs/rules/

**Official docs — Command Code**

34. https://commandcode.ai/docs
35. https://commandcode.ai/docs/agents (also served for `/docs/core-concepts/custom-agents`)
36. https://commandcode.ai/docs/skills
37. https://commandcode.ai/docs/mcp
38. https://commandcode.ai/docs/reference/cli — `cmdc` on Windows, subcommands, `--skill`, `--no-skills`, `--mod`, `--safe-mode`

**Local installs inspected**

39. Claude Code `2.1.215` — `C:\Users\raosu\.local\bin\claude.exe` (`--version`, `mcp add --help`, `mcp remove --help`, `plugin validate --help`); `C:\Users\raosu\.claude\{settings.json,CLAUDE.md,skills\,commands\,backups\}`; `C:\Users\raosu\.claude.json`; `C:\Users\raosu\.claude\agents\` absent
40. Codex `codex-cli 0.164.0-alpha.11.1` — `C:\Users\raosu\AppData\Local\OpenAI\Codex\bin\2993767bd1dfb009\codex.exe`; `C:\Users\raosu\.codex\{config.toml,config.toml.rea.backup,AGENTS.md,agents\code-reviewer.toml,agents\red-team.toml,skills\.system\*\SKILL.md}`
41. OpenCode `2.1.4` — `C:\Users\raosu\AppData\Local\Programs\@opencodedesktop\resources\{opencode-cli.exe,opencode-cli.version}`; `C:\Users\raosu\.config\opencode\{opencode.json,service.json,plugins\rtk.ts}`; `C:\Users\raosu\.local\share\opencode\`; older `1.1.65` in logs
42. `command-code@1.88.0` — `C:\nvm4w\nodejs\node_modules\command-code\{package.json,CHANGELOG.md,dist\cli.mjs,dist\bundled\}`; `C:\Users\raosu\.commandcode\{config.json,mcp.json,auth.json,updates.json,history.jsonl,skills\,projects\,cache\,file-history\}`; `C:\Users\raosu\.agents\{mcp.json,.skill-lock.json,skills\}`
43. Community, non-official: https://github.com/anthropics/claude-code/issues/25768 and /issues/46482 — `EBUSY`/`EPERM` on `~/.claude.json`
44. In-package Command Code docs: `C:\nvm4w\nodejs\node_modules\command-code\dist\bundled\command-code-knowledge\reference\{skills.md,mcp.md,custom-agents.md,custom-slash-commands.md,memory.md,hooks.md,permissions.md,plans.md}.md` and `…\dist\bundled\config\references\settings.md`

## Raw reports

- [./parts/claude-code.md](./parts/claude-code.md)
- [./parts/codex.md](./parts/codex.md)
- [./parts/opencode.md](./parts/opencode.md)
- [./parts/commandcode.md](./parts/commandcode.md)
