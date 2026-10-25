# Claude Code on-disk config surface

Scope: what a Windows desktop app must read/write to list, enable/disable, remove (V1) and create (V2) **subagents**, **skills**, and **MCP servers** at **project** and **global** scope, for Claude Code.

Machine under study: Windows 11, CLI `2.1.215` at `C:\Users\raosu\.local\bin\claude.exe` (verified: `claude --version`), home `C:\Users\raosu`.

---

## Path inventory

`~` = `%USERPROFILE%` on Windows (`[claude-directory](https://code.claude.com/docs/en/claude-directory)`), `$HOME` on macOS/Linux. Everything under `~/.claude` moves to `CLAUDE_CONFIG_DIR` when that env var is set (`[settings](https://code.claude.com/docs/en/settings)`, `[claude-directory](https://code.claude.com/docs/en/claude-directory)`).

| Artifact | Project | Global (Windows) | macOS / Linux | Format | Source |
|---|---|---|---|---|---|
| Subagents | `<repo>\.claude\agents\*.md` (recursive) | `C:\Users\raosu\.claude\agents\*.md` | `~/.claude/agents/` | MD + YAML frontmatter | `[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`, `[dir](https://code.claude.com/docs/en/claude-directory)` |
| Subagents (managed) | — | `C:\Program Files\ClaudeCode\.claude\agents\` | `/etc/claude-code/.claude/agents/`, `/Library/Application Support/ClaudeCode/.claude/agents/` | MD + frontmatter | `[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)` |
| Subagents (`--agents` JSON) | session only | — | — | JSON on CLI | `[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)` |
| Skills | `<repo>\.claude\skills\<name>\SKILL.md` | `C:\Users\raosu\.claude\skills\<name>\SKILL.md` | `~/.claude/skills/<name>/SKILL.md` | Dir + MD + frontmatter | `[skills:where](https://code.claude.com/docs/en/skills#where-skills-live)` |
| Skills (plugin) | — | `<plugin>\skills\<name>\SKILL.md` | same | Dir + MD | `[skills:where](https://code.claude.com/docs/en/skills#where-skills-live)` |
| Skills (synced) | — | `C:\Users\raosu\.claude\skills\synced\` | `~/.claude/skills/synced/` | Dir + MD, **reserved name** | `[skills:synced](https://code.claude.com/docs/en/skills#where-synced-skills-load)` |
| Slash commands | `<repo>\.claude\commands\*.md` | `C:\Users\raosu\.claude\commands\*.md` | `~/.claude/commands/` | MD + frontmatter | `[skills:where](https://code.claude.com/docs/en/skills#where-skills-live)` |
| MCP (project) | `<repo>\.mcp.json` | — | `<repo>/.mcp.json` | JSON `{mcpServers:{}}` | `[mcp:scopes](https://code.claude.com/docs/en/mcp#mcp-installation-scopes)` |
| MCP (local scope) | — | `C:\Users\raosu\.claude.json` → `projects["<path>"].mcpServers` | same | JSON | `[mcp:local](https://code.claude.com/docs/en/mcp#local-scope)` |
| MCP (user scope) | — | `C:\Users\raosu\.claude.json` → top-level `mcpServers` | same | JSON | `[mcp:user](https://code.claude.com/docs/en/mcp#user-scope)` |
| MCP (plugin) | — | `<plugin>\.mcp.json` or inline `plugin.json` | same | JSON | `[mcp:plugin](https://code.claude.com/docs/en/mcp#plugin-provided-mcp-servers)` |
| MCP (managed) | — | `C:\Program Files\ClaudeCode\managed-mcp.json` | `/etc/claude-code/managed-mcp.json` | JSON | `[managed](https://code.claude.com/docs/en/managed-settings)`, `[mcp](https://code.claude.com/docs/en/mcp)` |
| Settings (shared) | `<repo>\.claude\settings.json` | `C:\Users\raosu\.claude\settings.json` | `~/.claude/settings.json` | JSON | `[settings](https://code.claude.com/docs/en/settings#settings-files-and-who-they-affect)` |
| Settings (local) | `<repo>\.claude\settings.local.json` | — | same as project | JSON | `[settings](https://code.claude.com/docs/en/settings#settings-files-and-who-they-affect)` |
| Managed settings | — | `C:\Program Files\ClaudeCode\managed-settings.json` (+ `managed-settings.d\*.json`) | `/etc/claude-code/`, `/Library/Application Support/ClaudeCode/` | JSON | `[managed](https://code.claude.com/docs/en/managed-settings#where-each-mechanism-stores-the-policy)` |
| Managed settings (registry) | — | `HKLM\SOFTWARE\Policies\ClaudeCode` value `Settings`; HKCU same path as fallback | — | REG_SZ JSON | `[managed](https://code.claude.com/docs/en/managed-settings#where-each-mechanism-stores-the-policy)` |
| App state | — | `C:\Users\raosu\.claude.json` | `~/.claude.json` | JSON | `[settings](https://code.claude.com/docs/en/settings#find-or-create-your-settings-files)` |
| CLAUDE.md | `<repo>\CLAUDE.md` (+ `.claude\CLAUDE.md`, subdirs) | `C:\Users\raosu\.claude\CLAUDE.md` | `~/.claude/CLAUDE.md` | MD | `[dir](https://code.claude.com/docs/en/claude-directory)` |
| Rules | `<repo>\.claude\rules\*.md` | `~/.claude/rules/*.md` | same | MD + `paths` frontmatter | `[dir](https://code.claude.com/docs/en/claude-directory)` |
| Hooks | `<repo>\.claude\settings.json` → `hooks` | `~/.claude/settings.json` → `hooks` | same | JSON | `[hooks:locations](https://code.claude.com/docs/en/hooks#hook-locations)` |
| Plugins | `<repo>\.claude\settings.json` → `enabledPlugins`, `extraKnownMarketplaces` | `C:\Users\raosu\.claude\plugins\` (`installed_plugins.json`, `known_marketplaces.json`, `cache\`, `marketplaces\`) | `~/.claude/plugins/` | JSON + dirs | `[plugins-cli](https://code.claude.com/docs/en/plugins/cli-reference)`, `[dir](https://code.claude.com/docs/en/claude-directory)` |
| Subagent memory | `<repo>\.claude\agent-memory\<agent>\` / `agent-memory-local\` | `C:\Users\raosu\.claude\agent-memory\<agent>\` | same | MD (`MEMORY.md`) | `[sub:memory](https://code.claude.com/docs/en/sub-agents#enable-persistent-memory)` |
| Output styles | `<repo>\.claude\output-styles\*.md` | `~/.claude/output-styles/*.md` | same | MD + frontmatter | `[dir](https://code.claude.com/docs/en/claude-directory)` |
| Workflows | `<repo>\.claude\workflows\*.js` | `~/.claude/workflows/*.js` | same | JS | `[dir](https://code.claude.com/docs/en/claude-directory)` |

Managed-settings note: the legacy Windows path `C:\ProgramData\ClaudeCode\managed-settings.json` is **not** read (`[managed](https://code.claude.com/docs/en/managed-settings#where-each-mechanism-stores-the-policy)`).

---

## A. Paths and precedence

**Precedence — settings keys** (highest first): managed → `--settings`/CLI → `.claude/settings.local.json` → `.claude/settings.json` → `~/.claude/settings.json`. List keys (`permissions.allow`, `hooks`) **merge** across files rather than overriding; scalar keys take the highest level that sets them (`[settings:precedence](https://code.claude.com/docs/en/settings#settings-precedence)`).

**Precedence — subagent same name**: managed (1) > `--agents` (2) > `.claude/agents/` (3) > `~/.claude/agents/` (4) > plugin `agents/` (5) (`[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`).

**Precedence — skill same name**: enterprise > personal (`~/.claude/skills`) > project (`.claude/skills`); a skill beats a same-named `.claude/commands/` file; project skill replaces a same-named bundled skill but not its aliases (`[skills:resolve](https://code.claude.com/docs/en/skills#resolve-skills-that-share-a-name)`).

**Precedence — MCP same name**: local > project > user > plugin > claude.ai connectors; whole entry wins, **no field merging**; managed `managedMcpServers` ranks above all (v2.1.259+) (`[mcp:precedence](https://code.claude.com/docs/en/mcp#scope-hierarchy-and-precedence)`).

**Asymmetry the app must respect**: subagents and skills are discovered by **walking up** from the working directory to the repo root (every `.claude/agents/` and `.claude/skills/` on the path is read) (`[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`, `[skills:monorepo](https://code.claude.com/docs/en/skills#discovery-from-parent-and-nested-directories)`). Settings files are read from the **primary working directory only** (`[settings](https://code.claude.com/docs/en/settings#where-claude-code-keeps-the-local-file-in-a-git-repository)`).

`--add-dir` / `/add-dir` directories additionally load their `.claude/agents/`, `.claude/skills/`, `.claude/commands/` — that is *not* the same as `permissions.additionalDirectories`, which grants file access only (`[skills:add-dir](https://code.claude.com/docs/en/skills#skills-from-additional-directories)`).

`.claude/settings.local.json` placement on Windows: the "keep it at the git repo root" rule is explicitly excluded on Windows (and outside git repos, when repo root is home, or when ownership can't be established) — on Windows it stays with `.claude/settings.json` (`[settings](https://code.claude.com/docs/en/settings#where-claude-code-keeps-the-local-file-in-a-git-repository)`).

---

## B. Subagents

File = Markdown with YAML frontmatter; body is the system prompt. Only `name` + `description` are required (`[sub:frontmatter](https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields)`).

| Field | Req | Notes |
|---|---|---|
| `name` | **yes** | ≤256 chars, no `:` (reserved for plugin ids), must not start with `-`; filename need not match |
| `description` | **yes** | Drives delegation; all descriptions capped at 15k tokens combined (startup warning) |
| `tools` | no | Comma string or YAML list; MCP whole-server form `mcp__github__*` |
| `disallowedTools` | no | Removes the whole tool even with a specifier |
| `model` | no | `sonnet\|opus\|haiku\|fable\|<full id>\|inherit` |
| `permissionMode` | no | `default\|acceptEdits\|auto\|dontAsk\|bypassPermissions\|plan` (`manual`=alias of `default`); ignored for plugin agents |
| `maxTurns` | no | |
| `autoCompactWindow` | no | 100000–1000000 (v2.1.296+) |
| `skills` | no | Preload skill content at startup (first 32 names) |
| `mcpServers` | no | Inline defs or string refs to existing servers; ignored for plugin agents |
| `hooks` | no | Lifecycle hooks scoped to the agent; ignored for plugin agents |
| `memory` | no | `user\|project\|local` |
| `background` | no | `true` keeps agent backgrounded |
| `omitClaudeMd` | no | v2.1.271+ |
| `effort` | no | `low\|medium\|high\|xhigh\|max` |
| `isolation` | no | `worktree` |
| `color` | no | red/blue/green/yellow/purple/orange/pink/cyan |
| `initialPrompt` | no | Main-session-agent only |
| `experimental` | no | only `cacheTtl: 5m\|1h` |

Discovery / identity rules:

- Scanned **recursively**; subfolder path does not affect identity — identity is `name` only (`[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`).
- Two files in the *same* `agents/` tree declaring the same `name`: only one loads, chosen by filesystem read order — undocumented; `/doctor` reports it (`[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`).
- Nested project dirs: definition **closest to the working directory** wins (`[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`).
- Plugin subfolders *do* join the scoped id: `agents/review/security.md` → `my-plugin:review:security` (`[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`).
- Files silently skipped: no `name`; opening `---` not on line 1; bad `name`; `name` without `description`; unparsable YAML (debug log only) (`[sub:skips](https://code.claude.com/docs/en/sub-agents#subagent-files-claude-code-skips)`).

**Hot reload**: Claude Code watches `~/.claude/agents/` and `.claude/agents/`; add/edit is picked up within a few seconds, no restart (`[sub:write](https://code.claude.com/docs/en/sub-agents#write-subagent-files)`). Three restart cases:

1. First agent file created in a **new** `agents/` dir (watcher covers only dirs that existed at session start).
2. `.claude/agents/` inside an `--add-dir` / `/add-dir` directory (never watched).
3. Sessions started with `--disable-slash-commands` (no watching at all).

`[sub:write](https://code.claude.com/docs/en/sub-agents#write-subagent-files)`

**Disable vs delete**:
- Disable (non-destructive, keeps file): `permissions.deny: ["Agent(my-custom-agent)"]` in any settings file, or `claude --disallowedTools "Agent(Explore)"`. Works for built-in and custom agents (`[sub:disable](https://code.claude.com/docs/en/sub-agents#disable-specific-subagents)`). Deny rules apply without workspace trust; `allow` rules wait for trust (`[settings](https://code.claude.com/docs/en/settings#a-committed-key-doesn't-reach-teammates)`).
- Delete = remove the `.md` file; drops from the running session via the watcher (`[sub:write](https://code.claude.com/docs/en/sub-agents#write-subagent-files)`).

**Local machine**: **no** `C:\Users\raosu\.claude\agents` directory exists (verified by `Get-ChildItem -Force C:\Users\raosu\.claude`) — matching the task brief. The app must create it on first global-scope agent, and must expect the "new directory ⇒ restart" case above.

---

## C. Skills

Locations and load scope (`[skills:where](https://code.claude.com/docs/en/skills#where-skills-live)`):

| Location | Path | Loads in |
|---|---|---|
| Enterprise | `.claude/skills/<n>/SKILL.md` in managed settings dir | all users on the machine |
| Personal | `~/.claude/skills/<n>/SKILL.md` | all your projects (not cloud/Cowork) |
| Project | `.claude/skills/<n>/SKILL.md` | sessions in this repo |
| Nested | `<subdir>/.claude/skills/<n>/SKILL.md` | sessions started in/below `<subdir>`; otherwise loads lazily when Claude touches files there |
| Additional dir | `.claude/skills/` of an `--add-dir` dir | that session |
| Plugin | `<plugin>/skills/<n>/SKILL.md` | wherever plugin enabled, as `/plugin-name:skill-name` |
| claude.ai | downloaded to `~/.claude/skills/synced/` | signed-in sessions |

`SKILL.md` frontmatter — **all fields optional**, `description` recommended (`[skills:frontmatter](https://code.claude.com/docs/en/skills#frontmatter-reference)`): `name`, `description`, `when_to_use`, `argument-hint`, `arguments`, `disable-model-invocation`, `user-invocable`, `allowed-tools`, `disallowed-tools`, `model`, `effort`, `context`, `agent`, `background`, `hooks`, `paths`, `shell`, `metadata`, `license`, `compatibility` (20). `.claude/commands/*.md` accepts the same set **except** `name` and `paths` (`[dir](https://code.claude.com/docs/en/claude-directory)`).

Discovery rules that matter for a manager UI:
- Command name = frontmatter `name` else directory name; nested clashes get `/apps/web:deploy` (`[skills:name](https://code.claude.com/docs/en/skills#how-a-skill-gets-its-command-name)`).
- Reserved folder names: `synced` is skipped if you author it; `anthropic-skills` and `anthropic-skills:*` never load outside a plugin (`[skills:where](https://code.claude.com/docs/en/skills#where-skills-live)`).
- Name chars that break loading: zero-width/control chars, private-use glyphs, RTL marks, non-single spaces, `<` `>` (`[skills:names](https://code.claude.com/docs/en/skills#characters-a-skill-name-cant-hold)`).
- A skill folder containing `.claude-plugin/plugin.json` is loaded as a **plugin** `<name>@skills-dir`, which needs workspace trust in a project (`[skills:where](https://code.claude.com/docs/en/skills#where-skills-live)`).

Enable / disable / remove:
- Disable without editing the file: `skillOverrides` in settings — `"on" | "name-only" | "user-invocable-only" | "off"`; `/skills` menu writes it to `.claude/settings.local.json` (`[skills:overrides](https://code.claude.com/docs/en/skills#override-skill-visibility-from-settings)`).
- Disable model invocation only: `disable-model-invocation: true` in frontmatter (`[skills:control](https://code.claude.com/docs/en/skills#control-who-invokes-a-skill)`).
- Deny by permission: `Skill(deploy)` / `Skill(deploy *)` or deny the whole `Skill` tool (`[skills:restrict](https://code.claude.com/docs/en/skills#restrict-claude’s-skill-access)`).
- Remove: delete the skill directory (`~/.claude/skills/<n>/` or `.claude/skills/<n>/`); it drops from `/skills` in the live session (`[skills:remove](https://code.claude.com/docs/en/skills#remove-a-skill)`).

**Local machine** (`C:\Users\raosu\.claude\skills\`, verified): 44 entries — 43 authored skill dirs + the reserved `synced` dir (claude.ai sync + `.trash` handling). Each authored skill is a directory containing `SKILL.md` (sampled: `caveman\SKILL.md` + `README.md`). Example dirs: `caveman`, `caveman-commit`, `caveman-compress`, `caveman-optimize`, `code-review`, `research`, `tdd`, `wizard`, `writing-for-agents`, `…`. No project-scope `.claude/skills/` exists in the workspace under study.

---

## D. MCP

### Server JSON schema

Top-level wrapper for every file (`[mcp:scopes](https://code.claude.com/docs/en/mcp#project-scope)`):

```json
{ "mcpServers": { "<server-name>": { "type": "...", ... } } }
```

| Field | Applies to | Notes |
|---|---|---|
| `type` | all | `stdio` \| `http` \| `sse` \| `ws`; `streamable-http` accepted as alias of `http`. **Missing `type` = stdio** — an entry with `url` and no `type` is a config error and the server is skipped |
| `command` | stdio | executable path/name |
| `args` | stdio | array |
| `env` | stdio | object; `${VAR}` / `${VAR:-default}` expansion supported |
| `url` | http/sse/ws | |
| `headers` | http/sse/ws | `${VAR}` expansion; credential vars read as **empty** |
| `headersHelper` | http/sse/ws | command whose stdout JSON merges into headers; 10 s timeout |
| `oauth` | http/sse | `clientId`, `callbackPort`, `scopes`, `authServerMetadataUrl` |
| `timeout` | all | per-tool-call ms, values <1000 ignored |
| `alwaysLoad` | http/ws | listed alongside the ws field set |

Sources: `[mcp:http](https://code.claude.com/docs/en/mcp#option-1-add-a-remote-http-server)`, `[mcp:stdio](https://code.claude.com/docs/en/mcp#option-3-add-a-local-stdio-server)`, `[mcp:ws](https://code.claude.com/docs/en/mcp#option-4-add-a-remote-websocket-server)`, `[mcp:oauth](https://code.claude.com/docs/en/mcp#use-pre-configured-oauth-credentials)`, `[mcp:env](https://code.claude.com/docs/en/mcp#environment-variable-expansion-in-mcp-json)`.

`"type": "sdk"` entries are skipped by Claude Code — only an SDK host can register them (`[mcp:http](https://code.claude.com/docs/en/mcp#option-1-add-a-remote-http-server)`).

### Scopes

| Scope | Loads in | Shared | Stored in |
|---|---|---|---|
| `local` (**default**) | current project | no | `~/.claude.json` → `projects["<path>"].mcpServers` |
| `project` | current project | yes (commit) | `<repo>/.mcp.json` |
| `user` | all projects | no | `~/.claude.json` → top-level `mcpServers` |

`[mcp:scopes](https://code.claude.com/docs/en/mcp#mcp-installation-scopes)`

### CLI

- `claude mcp add [options] <name> <commandOrUrl> [args...]`; flags verified from local `claude mcp add --help` (2.1.215): `-s, --scope <local|user|project>` (default `local`), `-t, --transport <stdio|sse|http>` (default `stdio`), `-e/--env`, `-H/--header`, `--callback-port`, `--client-id`, `--client-secret`.
- `--` separates Claude's flags from the server command: `claude mcp add myserver -- npx my-mcp-server`. **`ws` cannot be added with `--transport`** — use `claude mcp add-json` (`[mcp:ws](https://code.claude.com/docs/en/mcp#option-4-add-a-remote-websocket-server)`).
- `claude mcp add-json <name> '<server-json>' [--scope ...]` (`[mcp:other](https://code.claude.com/docs/en/mcp#add-a-server-from-an-mcpservers-json-block)`).
- `claude mcp list`, `claude mcp get <name>`, `claude mcp remove <name> [-s <scope>]` (verified: without `--scope` it removes from whichever scope the server exists in), `claude mcp reset-project-choices`, `claude mcp login|logout <name>` (`[mcp:manage](https://code.claude.com/docs/en/mcp#managing-your-servers)`, local `claude mcp --help`).
- In-session `/mcp` panel for status, toggling, OAuth.

### Disable vs delete

Three **independent** mechanisms — the app must not conflate them (`[mcp:disable](https://code.claude.com/docs/en/mcp#disable-a-server-without-removing-it)`):

| Mechanism | Where | Effect |
|---|---|---|
| `/mcp` toggle → `disabledMcpServers` / `enabledMcpServers` | `~/.claude.json` → per project | per-project opt-out/in for user-configured, plugin, managed-provided, connector and default-on built-ins; lists are disjoint |
| `disabledMcpjsonServers` / `enabledMcpjsonServers` / `enableAllProjectMcpServers` | any `settings*.json` | approve/reject servers defined in the project's `.mcp.json` |
| Delete | `claude mcp remove`, or delete the `.mcp.json` entry | removes config; removing a remote server also deletes stored OAuth tokens + client registration (`[mcp:manage](https://code.claude.com/docs/en/mcp#managing-your-servers)`) |

Local evidence of all three shapes present: `C:\Users\raosu\.claude.json` line ~1570 `"disabledMcpServers": ["context7"]`, line ~2181 `"disabledMcpServers": [...]`, and per-project `"enabledMcpjsonServers": []` / `"disabledMcpjsonServers": []` (verified via `Get-Content C:\Users\raosu\.claude.json`).

---

## E. Other artifacts to surface

| Artifact | Location | One line |
|---|---|---|
| Hooks | `settings.json` → `hooks` (user/project/local/managed), plugin `hooks/hooks.json`, skill & subagent frontmatter | 3-level schema: event → matcher group → handler; handlers `command\|http\|mcp_tool\|prompt\|agent` (`[hooks](https://code.claude.com/docs/en/hooks#configuration)`) |
| Slash commands | `.claude/commands/*.md`, `~/.claude/commands/*.md` | Legacy single-file prompt; same mechanism as skills, same frontmatter minus `name`/`paths` (`[skills:where](https://code.claude.com/docs/en/skills#where-skills-live)`) |
| Plugins / marketplaces | `~/.claude/plugins/` + `settings.json` `enabledPlugins`, `extraKnownMarketplaces` | Managed with `claude plugin …`; `installed_plugins.json` is the install record (`[plugins-cli](https://code.claude.com/docs/en/plugins/cli-reference)`) |
| CLAUDE.md hierarchy | `~/.claude/CLAUDE.md`, `<repo>/CLAUDE.md`, `CLAUDE.local.md`, subdir `CLAUDE.md`, `AGENTS.md`, managed policy files | All levels load; `AGENTS.md` can substitute for `CLAUDE.md` (`[dir](https://code.claude.com/docs/en/claude-directory)`) |
| settings.json keys | `permissions.allow/ask/deny`, `hooks`, `env`, `model`, `enabledPlugins`, `extraKnownMarketplaces`, `skillOverrides`, `disableAllHooks`, `disableBundledSkills`, `agent`, `cleanupPeriodDays`, `$schema` | Strict JSON; a `//` comment or trailing comma is a Settings Error (`[settings](https://code.claude.com/docs/en/settings#edit-a-settings-file)`) |
| `.claude.json` | `~/.claude.json` | App state, per-project MCP/trust/history, global `/config` keys — "you don't need to edit it" (`[settings](https://code.claude.com/docs/en/settings#find-or-create-your-settings-files)`) |

---

## F. Mutation: what hot-reloads, what needs a restart

| Surface | Live behaviour | Source |
|---|---|---|
| `settings.json` / `settings.local.json` / managed file | **Watched**, most edits apply without restart incl. `permissions` and `hooks`; a file created mid-session is loaded if its folder already existed (`model`, `effortLevel`, `modelSettings` are read once at start) | `[settings](https://code.claude.com/docs/en/settings#when-edits-take-effect)` |
| `~/.claude/agents/`, `.claude/agents/` | **Watched** (few seconds) — 3 restart exceptions in §B | `[sub:write](https://code.claude.com/docs/en/sub-agents#write-subagent-files)` |
| `~/.claude/skills/`, `.claude/skills/`, `--add-dir` skills | **Watched** (SKILL.md text only); a *new* top-level skills dir needs `/reload-skills` | `[skills:live](https://code.claude.com/docs/en/skills#live-change-detection)` |
| Plugins | `/reload-plugins` (`--force` if MCP tools change); plugin `hooks/`, `.mcp.json`, `agents/`, `output-styles/` need it | `[skills:live](https://code.claude.com/docs/en/skills#live-change-detection)`, `[plugins-cli](https://code.claude.com/docs/en/plugins/cli-reference#reload-plugins)` |
| Hooks (settings edits) | Picked up by the file watcher | `[hooks](https://code.claude.com/docs/en/hooks#disable-or-remove-hooks)` |
| `.mcp.json` / `.claude.json` mcpServers | **No documented watcher** — new session, or reconnect from `/mcp` | inferred; see Unverified |

Safe disable semantics: prefer the documented "off switch" that keeps files intact — `permissions.deny: Agent(name)`, `skillOverrides: "off"`, `/mcp` toggle, `disabledMcpjsonServers`, `disableAllHooks` — because deletion is irreversible from the app's side and destroys user content (`[sub:disable](https://code.claude.com/docs/en/sub-agents#disable-specific-subagents)`, `[skills:overrides](https://code.claude.com/docs/en/skills#override-skill-visibility-from-settings)`, `[mcp:disable](https://code.claude.com/docs/en/mcp#disable-a-server-without-removing-it)`, `[hooks](https://code.claude.com/docs/en/hooks#disable-or-remove-hooks)`).

Risk of hand-editing:
- Settings files are strict JSON; invalid JSON or a schema-rejected value produces a **Settings Error** dialog at next start; individual bad entries produce a **Settings Warning** and are skipped (`[settings](https://code.claude.com/docs/en/settings#fix-a-broken-settings-file)`).
- `.claude.json` is the dangerous one: unparsable → **Configuration error**; Claude Code copies it to `~/.claude/backups/.claude.json.corrupted.<ts>` and offers reset. It keeps the **five** most recent `.claude.json.backup.<ts>` (`[settings](https://code.claude.com/docs/en/settings#fix-a-broken-settings-file)`, `[dir](https://code.claude.com/docs/en/claude-directory)`). Local machine has 5 such backups in `C:\Users\raosu\.claude\backups\` (verified).
- Docs explicitly say `~/.claude.json` is written for Claude Code itself ("you don't need to edit it") — an app writing it must be atomic and must tolerate concurrent writers (`[settings](https://code.claude.com/docs/en/settings#find-or-create-your-settings-files)`).

---

## G. V2 creation — exact scaffolding

### G1. Subagent (project or global)

`<scope>\agents\<name>.md` (`<scope>` = `<repo>\.claude` or `C:\Users\raosu\.claude`). Minimum valid file:

```markdown
---
name: code-reviewer
description: Reviews code for quality and best practices
tools: Read, Glob, Grep
model: sonnet
---
You are a code reviewer. When invoked, analyze the code and provide
specific, actionable feedback.
```

Required: `name`, `description`. `name` ≤256 chars, no `:`, must not start with `-`. Hook `---` must be line 1. (`[sub:write](https://code.claude.com/docs/en/sub-agents#write-subagent-files)`, `[sub:frontmatter](https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields)`)

Validation paths:
1. `claude plugin validate .claude\agents` — supports a bare component directory from **v2.1.233** (`[sub:skips](https://code.claude.com/docs/en/sub-agents#check-an-agents-directory-before-a-session)`, `[plugins-cli](https://code.claude.com/docs/en/plugins/cli-reference#validate-a-directory)`).
2. `claude --debug` then search `~/.claude/debug/<session-id>.txt` for `does not recognize or apply` (misspelled frontmatter, v2.1.296+) and for the skip reasons in §B.
3. Session-scope alternative with no files on disk: `claude --agents '{"reviewer":{"description":"...","prompt":"..."}}'` (ignores `color`/`experimental`) (`[sub:scope](https://code.claude.com/docs/en/sub-agents#choose-the-subagent-scope)`).

### G2. Skill

Directory `…\skills\<skill-name>\SKILL.md`:

```markdown
---
name: summarize-changes
description: Summarizes uncommitted changes and flags anything risky. Use when the user asks what changed.
---
Summarize the changes above in two or three bullet points, then list risks.
```

All frontmatter is optional; `description` is what drives auto-invocation (`[skills:frontmatter](https://code.claude.com/docs/en/skills#frontmatter-reference)`). Supporting files (`reference.md`, `scripts/`) may sit beside `SKILL.md`; keep `SKILL.md` under 500 lines (`[skills:support](https://code.claude.com/docs/en/skills#add-supporting-files)`). For a claude.ai-portable skill, restrict frontmatter to `name, description, license, compatibility, metadata, allowed-tools` (`[skills:outside](https://code.claude.com/docs/en/skills#using-skill-frontmatter-outside-claude-code)`).

### G3. MCP server — prefer the CLI

```powershell
# stdio, project scope (writes <repo>\.mcp.json)
claude mcp add --transport stdio myserver --scope project -- npx -y @scope/pkg

# stdio with env (put another option between --env and the name)
claude mcp add --env API_KEY=xxx --transport stdio api --scope user -- node server.js

# http, user scope (writes ~/.claude.json top-level mcpServers)
claude mcp add --transport http notion --scope user https://mcp.notion.com/mcp

# http with header
claude mcp add --transport http secure --scope project https://api.example.com/mcp --header "Authorization: Bearer X"

# ws (no --transport support) and pre-registered OAuth
claude mcp add-json events --scope project '{\"type\":\"ws\",\"url\":\"wss://mcp.example.com/socket\"}'
claude mcp add-json my-server '{"type":"http","url":"https://mcp.example.com/mcp","oauth":{"clientId":"id","callbackPort":8080}}' --client-secret
```

Required fields by transport: stdio → `command`; http/sse/ws → `type` + `url`. `type` omitted ⇒ stdio, so a `url` entry must carry an explicit type (`[mcp:http](https://code.claude.com/docs/en/mcp#option-1-add-a-remote-http-server)`). Server names must be letters/digits/`-`/`_`, and built-in names (`workspace`, `claude-in-chrome`, `computer-use`, `Claude Preview`, `Claude Browser`) are rejected (`[mcp:warnings](https://code.claude.com/docs/en/mcp#configuration-warnings)`).

Post-write verification: `claude mcp get <name>` (shows `✔ Connected`, `! Needs authentication`, `✘ Failed to connect`, `⏸ Pending approval`) (`[mcp:status](https://code.claude.com/docs/en/mcp#server-status)`).

Where to write (decision table for the app):

| User intent | CLI | File touched |
|---|---|---|
| Team-shared server | `--scope project` | `<repo>\.mcp.json` |
| Just me, this repo | `--scope local` (default) | `~/.claude.json` → `projects["<path>"].mcpServers` |
| Just me, all repos | `--scope user` | `~/.claude.json` → `mcpServers` |

---

## H. Version / stability notes (docs-stated)

| Item | Statement |
|---|---|
| Installed CLI here | `2.1.215` (verified `claude --version`) — several documented capabilities require later builds: `claude plugin validate <component-dir>` v2.1.233+, `/reload-skills` for new top-level skill dirs, `Claude Code watches ~/.claude/agents/` notes, `omitClaudeMd` v2.1.271+, `autoCompactWindow` v2.1.296+ |
| Skills standard | Follows the Agent Skills open standard; Claude Code adds `context`, `agent`, `paths`, `shell`, etc. (`[skills](https://code.claude.com/docs/en/skills)`) |
| Custom commands | "Custom commands have been merged into skills" — `.claude/commands/x.md` still works (`[skills](https://code.claude.com/docs/en/skills)`) |
| `/agents` | On v2.1.197 and earlier it opens an interactive wizard; now it only prints a reminder (`[sub](https://code.claude.com/docs/en/sub-agents)`) |
| SSE transport | Deprecated; HTTP tried first, auto-switch to SSE requires v2.1.265+ (`[mcp:sse](https://code.claude.com/docs/en/mcp#option-2-add-a-remote-sse-server)`) |
| WebSocket | No OAuth, no `--transport` flag; header-only auth (`[mcp:ws](https://code.claude.com/docs/en/mcp#option-4-add-a-remote-websocket-server)`) |
| `streamable-http` | Accepted as alias of `http` (`[mcp:http](https://code.claude.com/docs/en/mcp#option-1-add-a-remote-http-server)`) |
| Agent hooks | "experimental and may change" (`[hooks](https://code.claude.com/docs/en/hooks#hook-handler-fields)`) |
| MCP tasks | Public beta, v2.1.304+ (`[mcp:tasks](https://code.claude.com/docs/en/mcp#mcp-tasks-that-run-on-the-server)`) |

---

## I. Windows-specific issues

| Issue | Detail | Source |
|---|---|---|
| Home path | `~/.claude` = `%USERPROFILE%\.claude`; override with `CLAUDE_CONFIG_DIR` | `[settings](https://code.claude.com/docs/en/settings)`, `[dir](https://code.claude.com/docs/en/claude-directory)` |
| `settings.local.json` location | The git-root rule is skipped on Windows — the file stays with `.claude/settings.json` | `[settings](https://code.claude.com/docs/en/settings#where-claude-code-keeps-the-local-file-in-a-git-repository)` |
| Managed settings path | `C:\Program Files\ClaudeCode\managed-settings.json`; `C:\ProgramData\ClaudeCode\…` is legacy and ignored; HKLM `SOFTWARE\Policies\ClaudeCode` `Settings`, HKCU fallback | `[managed](https://code.claude.com/docs/en/managed-settings#where-each-mechanism-stores-the-policy)` |
| Hooks shell | Command hooks default to `bash`, or to **PowerShell on Windows when Git Bash isn't installed**; set `"shell": "powershell"`; matcher must be `Bash\|PowerShell` to cover both tools | `[hooks](https://code.claude.com/docs/en/hooks#command-hook-fields)` |
| Hooks without Git Bash | A `PreToolUse` / `PermissionRequest` hook that fails can **block** the tool call ("blocking because Git Bash wasn't found…"); only exit 0/2 counts. Fix with explicit `onFailure` + `shell: powershell` | `[hooks](https://code.claude.com/docs/en/hooks#failed-hooks-on-windows-without-git-bash)` |
| Exec form | Requires a real `.exe`; `.cmd`/`.bat` shims (npm/npx/eslint) cannot be spawned without a shell — use shell form or `node script.js` | `[hooks](https://code.claude.com/docs/en/hooks#exec-form-and-shell-form)` |
| PowerShell hooks + paths | `&` before a quoted path, `$env:CLAUDE_PROJECT_DIR` inline (a `check.ps1` example exists) | `[hooks](https://code.claude.com/docs/en/hooks#reference-scripts-by-path-in-powershell-hooks)` |
| Skill shell | `shell: powershell` in SKILL.md; `shell: bash` fails hard on Windows without Git Bash | `[skills:frontmatter](https://code.claude.com/docs/en/skills#frontmatter-reference)` |
| Trust dialog | Per-project `hasTrustDialogAccepted` in `~/.claude.json` gates: `.mcp.json` approvals committed to the repo, project subagent frontmatter hooks, inline MCP servers in project agent files, skills-dir plugins | `[mcp:trust](https://code.claude.com/docs/en/mcp#project-server-approvals-and-workspace-trust)`, `[sub:trust](https://code.claude.com/docs/en/sub-agents#inline-server-trust)`, `[sub:hooks](https://code.claude.com/docs/en/sub-agents#hooks-in-subagent-frontmatter)` |
| Trust is per-folder | A parent folder's trust doesn't count; a `-p` session doesn't count; `--add-dir` dirs need their own trust entry | `[sub:trust](https://code.claude.com/docs/en/sub-agents#inline-server-trust)` |
| stdio shutdown | Windows: stdin closed, then process tree ended after 300 ms (macOS/Linux: SIGINT→SIGTERM→SIGKILL) | `[mcp:stdio](https://code.claude.com/docs/en/mcp#how-claude-code-stops-a-stdio-server)` |
| File locking | Community reports: concurrent sessions on Windows hit `EBUSY`/`EPERM` writing `~/.claude.json` because `renameSync` fails when another process holds the target open. Not addressed in official docs | `[issue 25768](https://github.com/anthropics/claude-code/issues/25768)`, `[issue 46482](https://github.com/anthropics/claude-code/issues/46482)` |

---

## Unverified

1. **`.mcp.json` hot-reload.** Docs document watchers for settings files, skills dirs, agents dirs and plugins, but never for `.mcp.json`. Treat MCP config changes as session-restart/reconnect-required; confirm empirically before advertising live refresh.
2. **File-locking behaviour on Windows.** Only community issues describe `EBUSY`/`EPERM` on `~/.claude.json`; no official statement. The app should write atomically (temp + rename), retry, and never truncate-then-write.
3. **`claude plugin validate` on a bare `.claude\agents` / `.claude\skills` directory on 2.1.215.** Docs gate component-directory validation at v2.1.233; the local binary exposes `validate <path>` but the component-directory mode was not exercised.
4. **`/reload-skills` on 2.1.215.** Documented as the fix for a newly created top-level skills directory; whether this exact build has it was not tested (would require an interactive session).
5. **Subagent watcher behaviour on this machine.** No `~/.claude/agents/` exists here, so the "watcher covers only directories that existed at session start" case could not be observed.
6. **`.mcp.json` ancestor walk.** Docs place `.mcp.json` at "your project's root"; whether Claude Code walks up from a nested cwd like it does for `agents/` and `skills/` is not stated.
7. **Desktop-app parity.** Docs state the desktop app's Code tab reads the same settings files (`[settings](https://code.claude.com/docs/en/settings)`), but agent/skill/MCP discovery inside the desktop app was not observed directly.

---

## Sources

Official docs (code.claude.com):

- [Subagents](https://code.claude.com/docs/en/sub-agents) — scope, frontmatter, skip rules, hot reload, disable, `plugin validate`, Windows hooks note
- [Skills](https://code.claude.com/docs/en/skills) — locations, frontmatter, precedence, overrides, live change detection, removal
- [MCP](https://code.claude.com/docs/en/mcp) — schema, scopes, CLI, precedence, disable, status, trust, Windows stdio shutdown
- [Settings](https://code.claude.com/docs/en/settings) — four settings files, precedence, upgrade of `.claude.json`, strict JSON, Windows notes
- [Managed settings](https://code.claude.com/docs/en/managed-settings) — per-OS paths, registry keys, legacy path
- [Claude directory](https://code.claude.com/docs/en/claude-directory) — full file inventory, frontmatter-by-file table, app data
- [Hooks](https://code.claude.com/docs/en/hooks) — hook locations, handler fields, PowerShell/Windows failures
- [Plugin CLI reference](https://code.claude.com/docs/en/plugins/cli-reference) — `claude plugin validate|disable|enable|list`, `/reload-plugins`

Local evidence (absolute paths on this machine):

- `C:\Users\raosu\.claude\settings.json` — keys: `env`, `model`, `hooks.PreToolUse`, `enabledPlugins` (4), `extraKnownMarketplaces` (2), `effortLevel`, `modelSettings`, `autoUpdatesChannel`, `theme`, `skipAutoPermissionPrompt`
- `C:\Users\raosu\.claude\CLAUDE.md` — contains `@RTK.md`
- `C:\Users\raosu\.claude\skills\` — 43 authored skill dirs + `synced`
- `C:\Users\raosu\.claude\commands\` — exists, **empty**
- `C:\Users\raosu\.claude\agents\` — **does not exist**
- `C:\Users\raosu\.claude.json` — top-level `mcpServers` = `{context7: {type: stdio, command: npx, args: [-y, @upstash/context7-mcp]}}`; 23 `projects` entries each with `mcpServers`, `enabledMcpjsonServers`, `disabledMcpjsonServers`, `hasTrustDialogAccepted`; `disabledMcpServers: ["context7"]` in ≥2 projects
- `C:\Users\raosu\.claude\plugins\installed_plugins.json`, `known_marketplaces.json` — 4 plugin installs, 3 marketplaces
- `C:\Users\raosu\.claude\backups\` — 5 `.claude.json.backup.<ts>` files
- `claude --version` → `2.1.215`; `claude mcp add --help`, `claude mcp remove --help`, `claude plugin --help`, `claude plugin validate --help` outputs used for exact flags

Community (non-official, flagged as such):

- [claude-code#25768 EBUSY on .claude.json](https://github.com/anthropics/claude-code/issues/25768)
- [claude-code#46482 EBUSY Windows concurrent writes](https://github.com/anthropics/claude-code/issues/46482)
