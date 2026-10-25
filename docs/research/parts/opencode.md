# OpenCode (sst/opencode) — on-disk configuration surface

Scope: what a Windows desktop app must read/write to **list, enable/disable, remove (V1)** and
**create (V2)** subagents, skills, and MCP servers for OpenCode at **project** and **global** scope.

Inspected machine (ground truth): **OpenCode 2.1.4** — the installed desktop app is
`C:\Users\raosu\AppData\Local\Programs\@opencodedesktop\OpenCode.exe` with a bundled CLI
`…\resources\opencode-cli.exe` and `…\resources\opencode-cli.version` = `2.1.4`. The CLI is **not on
`PATH`**; `Get-Command opencode` fails, so all CLI facts below come from the bundled binary.
`opencode debug paths` on this machine reports `config = C:\Users\raosu\.config\opencode`
(see §I). Older logs on disk show a previous **1.1.65** install, so both config generations exist here.

Citation tags are expanded in **Sources**. `[local:…]` = absolute path on this machine. Because
the machine runs V2, V2 is treated as authoritative and V1 is documented as the compatibility layer
(the V2 CLI reads V1 shapes — verified, §H).

---

## Path inventory

| Artifact | Project scope | Global scope (Windows) | macOS / Linux | Format | Source |
|---|---|---|---|---|---|
| Main config document | `<repo>/opencode.json` \| `.jsonc`, and `<repo>/.opencode/opencode.json(c)` | `C:\Users\raosu\.config\opencode\opencode.json` (`.jsonc` also read); legacy `config.json` also probed by V1 | `~/.config/opencode/opencode.json(c)`; `$XDG_CONFIG_HOME/opencode/…` when set | JSON / JSONC | [V2CFG], [V1CFG], [local CLI] |
| System / managed config | — | `%ProgramData%\opencode\opencode.json(c)` (absent on this machine) | `/etc/opencode/`; macOS `/Library/Application Support/opencode/` + MDM plist | JSON / JSONC | [V2CFG], [V1CFG], [local] |
| **Subagent / agent file** | `.opencode/agents/<name>.md` (V1 also `agent/`, `mode/`, `modes/`) | `C:\Users\raosu\.config\opencode\agents\<name>.md` | `~/.config/opencode/agents/<name>.md` | Markdown + YAML frontmatter (1 file = 1 agent) | [V2AGENTS], [V1AGENTS], [MIGRATE] |
| Agents inline | `agents{}` (V2) / `agent{}` (V1) in any config document | same | same | JSON | [V2AGENTS], [V1AGENTS], [SCHEMA] |
| **Skill** | `.opencode/skills/<id>/SKILL.md` (V2 also `.opencode/skill/`); `.claude/skills/…`; `.agents/skills/…` | `C:\Users\raosu\.config\opencode\skills\<id>\SKILL.md`; `~/.claude/skills`; `~/.agents/skills` | same | Markdown + YAML frontmatter (+ sibling files) | [V2SKILLS], [V1SKILLS] |
| **MCP server** | `mcp.servers.<name>` (V2) / `mcp.<name>` (V1) in project config | same key in `%USERPROFILE%\.config\opencode\opencode.json` | same | JSON object | [V2MCP], [V1MCP], [SCHEMA] |
| MCP OAuth credentials | — | `~/.local/share/opencode/mcp-auth.json` (V1 doc; V2 says "outside project configuration", path not restated) | same | JSON, CLI-managed | [V1MCP] |
| Plugins (local code) | `.opencode/plugins/*.ts`, `*.js`, `<pkg>/` (V1 also `plugin/`) | `C:\Users\raosu\.config\opencode\plugins\` (present: `rtk.ts`) | `~/.config/opencode/plugins/` | TypeScript / JavaScript | [V2PLUG], [V2PLUGAPI], [local:\Users\raosu\.config\opencode\plugins\rtk.ts] |
| Plugins (npm packages) | `plugins[]` (V2) / `plugin[]` (V1) | same array in global config | same | JSON array of strings/objects | [V2PLUG], [V1PLUG], [SCHEMA] |
| Commands | `.opencode/commands/<name>.md` (V2 & V1; V1 also `command/`) | `%USERPROFILE%\.config\opencode\commands\` | `~/.config/opencode/commands/` | Markdown + frontmatter | [MIGRATE], [V1CFG] |
| Instruction files | `AGENTS.md` in workspace → project root dirs; `instructions[]` globs/URLs | `C:\Users\raosu\.config\opencode\AGENTS.md` | `~/.config/opencode/AGENTS.md` (V1 also `~/.claude/CLAUDE.md`) | Markdown | [V2INSTR], [V1RULES] |
| Themes | — | V1 `~/.config/opencode/themes/` + `tui.json`; V2 `~/.config/opencode/cli.json` → `theme` | same | JSON | [V1CFG], [V2CLICFG] |
| Keybinds / TUI settings | — (V2 has no project-local client config) | V1 `tui.json`; V2 `~/.config/opencode/cli.json` | same | JSON | [V2CLICFG], [V1CFG] |
| Providers / models | `providers{}` (V2) / `provider{}` (V1) | same | same | JSON | [V2CFG], [MIGRATE] |
| Provider credentials | — | `~/.local/share/opencode/auth.json` | same | JSON, CLI-managed | [V1CLI] |
| Formatter / LSP | `formatter`, `lsp` | same | same | JSON | [V1CFG], [MIGRATE] |
| Backing service secret | — | `C:\Users\raosu\.config\opencode\service.json` (contains `password`) | unknown | JSON | [local:\Users\raosu\.config\opencode\service.json] |
| Data / state / cache | — | `~/.local/share/opencode` (db, `log/`, `storage/`, `snapshot/`, `repos/`, `bin/`), `~/.local/state/opencode`, `~/.cache/opencode`, `%TEMP%\opencode` | same | mixed | [local CLI] |
| Desktop app install | — | `%LOCALAPPDATA%\Programs\@opencodedesktop\` (`OpenCode.exe`, `resources\opencode-cli.exe`) | `.dmg` / `.AppImage` / `.deb` / `.rpm` | binary | [local], [GH] |

**Note:** `.opencode/` subdirectories use **plural** names (`agents/`, `commands/`, `plugins/`,
`skills/`, `themes/`, plus `modes/`, `tools/`); singular forms are accepted for backwards
compatibility ([V1CFG]).

---

## A. Config roots and precedence

**Roots.** Global config is `~/.config/opencode/opencode.json` — on Windows that literally is
`%USERPROFILE%\.config\opencode`, **not** `%APPDATA%` ([V2CFG], [V1CFG]; verified by
`opencode debug paths` → `config C:\Users\raosu\.config\opencode` [local CLI]). Project config is
`opencode.json(c)` in the project, and/or `.opencode/opencode.json(c)` ([V2CFG]).

**Precedence, V2 (lowest → highest)** ([V2CFG]):

| # | Source | Notes |
|---|---|---|
| 1 | `~/.config/opencode/opencode.json(c)` | user-global |
| 2 | ancestor project `opencode.json(c)`, farthest → nearest | directories merging, not one file |
| 3 | `.opencode/opencode.json(c)` in each discovered dir | "every discovered `.opencode` config overrides every direct config" |
| 4 | `/etc/opencode/` · `%ProgramData%\opencode` | system/managed, wins over all user/project files |
| 5 | macOS MDM `ai.opencode.managed` plist | highest, not user-overridable |

**Precedence, V1 (8 tiers, later overrides earlier)** ([V1CFG]): remote `.well-known/opencode` →
global → `OPENCODE_CONFIG` → project → `.opencode` dirs → `OPENCODE_CONFIG_CONTENT` → managed dir →
macOS managed prefs.

**Merge rules — the part that matters for a writer:**

| Type | Behaviour | Source |
|---|---|---|
| Object keys | deep-merged; later source wins per conflicting key, non-conflicting keys preserved | [V1CFG], [V2CFG] |
| `mcp.servers.<name>` | **whole server object is replaced** by the higher-precedence config — repeat every required field | [V2MCP] |
| `plugins[]` | applied lowest → highest precedence, **combined, not replaced** | [V2PLUG] |
| `skills[]` | "combined rather than replaced" | [V2SKILLS] |
| `instructions[]` | combined across every config document, in listed order | [V2INSTR] |
| Agent permission rules | **append**; later scalar values replace earlier ones | [V2AGENTS] |
| V1 vs V2 field mix | allowed at top level; a valid V2 value wins over the V1 equivalent. **Nested mixing is bounded** — inside a single `agents`/`providers`/`commands`/`models` entry keep one format | [MIGRATE] |

**Document probing on Windows (ground truth).** V1 (1.1.65) probed three filenames in the global dir
in order — `config.json`, `opencode.json`, `opencode.jsonc` — per `[local:…\.local\share\opencode\log\2026-02-18T010158.log]`.
V2 `opencode debug config` lists loaded documents as `{type:"document", path, info}` plus
`{type:"directory", path}` for the scanned config dir [local CLI].

**Env overrides** ([V1CLI], [V2CLICFG]):

| Variable | Effect |
|---|---|
| `OPENCODE_CONFIG` | path to a config **file**, loaded between global and project |
| `OPENCODE_CONFIG_DIR` | extra config **directory**, loaded after global + `.opencode` (can override both) |
| `OPENCODE_CONFIG_CONTENT` | inline JSON config, highest of the file-ish layers (V1) |
| `XDG_CONFIG_HOME` | relocates the config dir to `$XDG_CONFIG_HOME/opencode` — **verified on Windows**: with it set, `debug paths` reported `config …\xdgtest\opencode` [local CLI] |
| `OPENCODE_DISABLE_PROJECT_CONFIG` | skip project config **and** project `AGENTS.md` | 
| `OPENCODE_DISABLE_EXTERNAL_SKILLS` | skip `.claude`/`.agents` skill sources |
| `OPENCODE_DISABLE_CLAUDE_CODE` / `_PROMPT` / `_SKILLS` | V1 `.claude` compatibility kill-switches |
| `OPENCODE_DB`, `OPENCODE_GIT_BASH_PATH`, `OPENCODE_DISABLE_FFF` | db path; Git Bash path on Windows; force ripgrep file finder (default on Windows) |

**Managed settings cannot be beaten by settings** — restrictions need `policies`
([V2CFG], [MIGRATE]). A desktop app must therefore show "locked" state for managed sources instead of
offering edits it cannot win.

---

## B. Agents / subagents

**Two forms, same ID space** ([V2AGENTS], [V1AGENTS]):

| Form | Path / key | Format |
|---|---|---|
| Markdown file | project `.opencode/agents/<name>.md`; global `~/.config/opencode/agents/<name>.md` | YAML frontmatter + body = system prompt |
| Inline JSON | `agents.<name>` (V2) / `agent.<name>` (V1) in any config document | object |

**Name/ID derivation** ([V1AGENTS], [V2AGENTS]): the file name is the agent name (`review.md` →
`review`); in V2 a nested path becomes part of the ID (`.opencode/agents/team/reviewer.md` →
`team/reviewer`). V1 also discovers `agent/`, `mode/`, `modes/`; V2 still discovers all four, and
`mode/`-`modes/` files are treated as primary agents ([MIGRATE]).

**Frontmatter / JSON fields.**

| Field | V2 (current) | V1 (legacy) | Source |
|---|---|---|---|
| description | `description` (required, shown to the model for subagents) | `description` | [V2AGENTS], [V1AGENTS] |
| mode | `mode: primary \| subagent \| all` (default for a new custom agent: **primary** in V2; `all` in V1) | `mode` | [V2AGENTS], [V1AGENTS] |
| model | `model: provider/model` with optional `#variant`, or `{providerID,model,variant}` | `model` + separate `variant` | [V2AGENTS], [MIGRATE] |
| system prompt | `system` (JSON) or Markdown body | `prompt` (file body in MD) | [V2AGENTS], [MIGRATE] |
| permissions | `permissions: [{action,resource,effect}]` ordered, last match wins | `permission: {tool: allow\|ask\|deny}` + `tools: {tool: bool}` (deprecated) | [V2AGENTS], [V1AGENTS] |
| temperature / top_p | **not used** — set on a model variant (`request.body` is preserved but not sent) | `temperature`, `top_p` | [V2AGENTS], [MIGRATE], [SCHEMA] |
| steps | `steps` (positive int) | `steps`, legacy `maxSteps` | [V2AGENTS], [V1AGENTS] |
| visibility | `hidden: true` (subagents only; not a security control) | `hidden` | [V2AGENTS], [V1AGENTS] |
| colour | `color: "#rrggbb"` (V2 schema: hex only) | hex **or** theme name | [V2AGENTS], [V1AGENTS], [SCHEMA] |
| disable | `disabled: true` | `disable: true` | [V2AGENTS], [V1AGENTS] |
| extra | `request.{headers,body}`; any other JSON key is passed to the provider as a model option (V1) | | [V2AGENTS], [V1AGENTS] |

A Markdown file that uses **any** of `permissions`, `request`, `system`, `disabled` is parsed as V2 and
its V1 fields (`name`, `tools`, `permission`, `temperature`) are **ignored with a warning**
([MIGRATE]) — so a writer must emit one format per file, not a blend.

**Built-in agents on this machine (2.1.4, `opencode debug agents`)** [local CLI]:

| id | mode | hidden | notes |
|---|---|---|---|
| `build` | primary | no | default; `*` allow, `read *.env*` ask, `browser` deny |
| `plan` | primary | no | `edit` deny, except `C:\Users\raosu\.opencode\plan\*` allow; `question` allow |
| `general` | subagent | no | broad tools; `subagent` deny, `question` deny |
| `explore` | subagent | no | read/search only; `subagent` deny, `edit` absent |
| `compaction` | primary | **yes** | maintenance |
| `summary` | primary | **yes** | maintenance |
| `title` | primary | **yes** | maintenance |

Docs list build/plan as the visible primaries and general/explore as subagents, and state V2 has no
`scout` agent ([V2AGENTS], [V1AGENTS]); the hidden trio matches [V2AGENTS].

**Disable vs remove.**

| Goal | V2 | V1 | Source |
|---|---|---|---|
| Disable a built-in / JSON agent | `agents.<id>.disabled: true` ("removes … at that point in configuration loading") | `agent.<id>.disable: true` | [V2AGENTS], [V1AGENTS] |
| Remove | delete the Markdown file, or delete the JSON key (and, for built-ins, delete the override entry) | same | [V1AGENTS] ("create by adding a markdown file"; file name = name) |
| Per-agent subagent gating | `permissions: [{action:"subagent", resource:"*", effect:"deny"}, …]` — a `deny` also strips the agent from the tool description | `permission.task` globs | [V2AGENTS], [V1AGENTS] |

Defaults: `subagent_depth` = 1 (subagents cannot spawn subagents), V2 under `experimental.subagent_depth`
([V1CFG], [V2CFG], [SCHEMA]).

---

## C. Skills

**Supported — fully first-class in both V1 and V2** ([V2SKILLS], [V1SKILLS]).

**Discovery locations** ([V2SKILLS], [V1SKILLS]):

| Scope | Sources (V2) | V1 equivalents |
|---|---|---|
| Global | `~/.config/opencode/skills` | same |
| Global compat | `~/.claude/skills`, `~/.agents/skills` | same |
| Project | `.opencode/skills` (+ V2 also `.opencode/skill/`) | `.opencode/skills/<name>/SKILL.md` |
| Project compat | `.claude/skills`, `.agents/skills` | same |

Project sources are searched from the CWD up to the project root, **including every level in between**;
search stops six directories below a source root; hidden dirs and `node_modules` are skipped
([V2SKILLS]). Extra sources come from `skills: []` config (`./relative` from the working directory,
`~/`, absolute, or an HTTP catalog URL) ([V2SKILLS], [SCHEMA] has both the V2 array and the deprecated
V1 `{paths,urls}` object).

**Two shapes per source** ([V2SKILLS]): `<source>/<id>.md` at the root, or
`<source>/<grouping>/<id>/SKILL.md` (ID = the directory holding `SKILL.md`; a directory containing
`SKILL.md` is exactly one skill).

**Frontmatter.**

| Field | V2 | V1 | Source |
|---|---|---|---|
| `name` | display name (defaults to path ID) | **required**, 1–64 chars, must match directory, `^[a-z0-9]+(-[a-z0-9]+)*$` enforced | [V2SKILLS], [V1SKILLS] |
| `description` | optional at runtime; **without it the skill is not advertised** | **required**, 1–1024 chars | [V2SKILLS], [V1SKILLS] |
| `metadata.opencode/autoinvoke: false` | hides from the model's list (still loadable by ID) | ignored | [V2SKILLS] |
| `disable-model-invocation: true` | same effect; `opencode/autoinvoke` wins if both | — | [V2SKILLS] |
| `license`, `compatibility` | accepted, not interpreted | accepted | [V2SKILLS], [V1SKILLS] |

V2 does **not** enforce the ID pattern, length, name/dir match, or description length; IDs are exact
and case-sensitive ([V2SKILLS]).

**Precedence** (later wins for the same ID) ([V2SKILLS]): built-ins → `.claude/skills` → `.agents/skills`
→ `~/.config/opencode/skills` → project `.opencode/skills` → explicit `skills` config entries.

**Enable / disable** ([V2SKILLS], [V1SKILLS]):

| Effect | V2 | V1 |
|---|---|---|
| deny (hidden from model, load rejected) | `permissions: [{action:"skill", resource:"internal-*", effect:"deny"}]` | `permission: {skill: {"*":"allow","internal-*":"deny"}}` |
| ask | `effect:"ask"` | `"ask"` |
| hide from model but keep loadable | frontmatter `metadata.opencode/autoinvoke: false` | not available |
| disable the whole skill tool | agent `tools.skill: false` (or omit permission) | `tools: {skill: false}` per agent |

Removal is filesystem-only: delete the skill directory (`SKILL.md` **must** be all-caps; mismatch is
the #1 troubleshooting cause) ([V1SKILLS], [V2SKILLS]).

---

## D. MCP servers

**Shape changed in V2.** V1 puts servers directly under `mcp`; V2 groups them under **`mcp.servers`**
and inverts the flag ([V2MCP], [MIGRATE]):

```jsonc
// V2 (current)
{ "mcp": { "timeout": {"startup":45000,"catalog":30000,"execution":600000},
           "servers": { "name": { "type": "local", "command": ["npx","-y","pkg"],
                                   "environment": {"K":"v"}, "disabled": false } } } }
// V1 (legacy, still read)
{ "mcp": { "name": { "type": "local", "command": ["npx","-y","pkg"], "enabled": true } } }
```

| Field | V2 local | V2 remote | V1 | Source |
|---|---|---|---|---|
| `type` | `"local"` (required) | `"remote"` (required) | same | [V2MCP], [V1MCP] |
| `command` | array, required | — | `command` array, required | [V2MCP], [V1MCP], [SCHEMA] |
| `cwd` | relative to workspace, optional | — | `cwd` (V1 doc) | [V2MCP], [V1MCP] |
| `environment` | string map, optional | — | `environment` | [V2MCP], [V1MCP] |
| `url` | — | absolute Streamable-HTTP URL, required | `url` | [V2MCP], [V1MCP] |
| `headers` | — | string map | `headers` | [V2MCP], [V1MCP] |
| `enabled` | **not used** | **not used** | `enabled: bool` | [V2MCP], [V1MCP] |
| `disabled` | `true` prevents connection (default `false`) | same | — | [V2MCP], [SCHEMA] |
| `oauth` | — | `{client_id,client_secret,scope,callback_port,redirect_uri,auth_server_metadata_url}` or `false` | `{clientId,clientSecret,scope,callbackPort,redirectUri}` or `false` | [V2MCP], [V1MCP] |
| `timeout` | per-purpose object `{startup,catalog,execution}` | same | single integer ms | [V2MCP], [MIGRATE] |
| `codemode` | default `true`; `false` exposes tools natively | same | — | [V2MCP] |
| `protocol` | `legacy` (default) \| `auto` \| `2026-07-28` | same | — | [V2MCP] |

Timeouts default to 30 s startup, 30 s catalog, 12 h execution ([V2MCP]); the V1 schema doc text says
`timeout` "defaults to 5000" ([SCHEMA] `McpLocalConfig`) while the V1 page says 30000 — a doc/schema
mismatch, see Unverified.

**Per-server enable flag = `disabled` in V2, `enabled` in V1.** `opencode debug config` proves the
translation: the on-disk V1 entry `"mcp": {"rea": {… "enabled": true}}` is reported as
`mcp.servers.rea … "disabled": false` [local CLI] / [local:\Users\raosu\.config\opencode\opencode.json].

**Project vs global.** Servers live in whichever config document defines them; a higher-precedence
document **replaces the whole server object** with the same name ([V2MCP]). Organisations can ship
disabled servers via `.well-known/opencode` and users opt in with `enabled: true` (V1) / by overriding
the object (V1 docs, [V1CFG], [V1MCP]).

**CLI (verified on 2.1.4)** [local CLI]:

| Command | Flags (from `--help`) |
|---|---|
| `opencode mcp list` | "List configured MCP servers and their status" |
| `opencode mcp add <name> [-- <command…>]` | `--url`, `--header key=value`, `--env key=value`, `--global` (write global instead of project config) |
| `opencode mcp auth [name]` | OAuth browser flow; `auth` with no name prompts |
| `opencode mcp logout [name]` | removes stored OAuth credentials |
| `opencode mcp` subcommands | exactly `list`, `add`, `auth`, `logout` — **no `remove`/`disable` subcommand** |

Docs add `opencode mcp auth list` / `mcp auth ls` and `opencode mcp debug <name>` ([V1CLI], [V1MCP]);
`debug` is **not** in the 2.1.4 help output [local CLI].

**Disable vs delete.**

| Intent | Action | Source |
|---|---|---|
| temporarily off | V2 `"disabled": true` (stays configured, does not connect); V1 `"enabled": false` | [V2MCP], [V1MCP] |
| delete | remove the `mcp.servers.<name>` entry from the project **or** global document where it was added | [V2MCP] |
| disconnect only | `/mcps` in the TUI — **does not change configuration** | [V2MCP] |
| revoke credentials | `opencode mcp logout <name>` | [V2MCP], [V1MCP] |

Tool naming: `<server>_<tool>`, non-alphanumerics normalised to `_`; hide tools per server with a
permission action `"<server>_*"` ([V2MCP], [V1MCP]).

---

## E. Other artifacts

| Artifact | Location / key | Notes | Source |
|---|---|---|---|
| Plugins (local) | `.opencode/plugins/` (project), `~/.config/opencode/plugins/` (global), or `plugins: ["./dir"]` | configured local paths must be **directories**; OpenCode loads the dir's `index`/`server` module; a single-file path is skipped (V2) | [V2PLUG], [MIGRATE] |
| Plugins (npm) | `plugins: ["pkg", "pkg@1.2.0", "@scope/pkg", {package, options}]`; wildcards `*`, `.*`; `-id` disables | V1 used `plugin`; V1 tuple `[pkg, options]` became an object | [V2PLUG], [MIGRATE] |
| Plugin load order | global config → project config → global plugin dir → project plugin dir | all hooks run in sequence | [V1PLUG] |
| Commands | `.opencode/commands/<name>.md`; JSON `commands{}` (V2) / `command{}` (V1) | fields `template`, `description`, `agent`, `model`, `subagent` (was `subtask`) | [V2CFG], [MIGRATE] |
| Instructions | `AGENTS.md` (global + workspace→root/shared-parent), plus `instructions: []` paths/globs/URLs | V2 loads **only** `AGENTS.md`; V1 fell back to `CLAUDE.md` / `~/.claude/CLAUDE.md` | [V2INSTR], [V1RULES], [MIGRATE] |
| Instructions hot-edit | edits to global/upward `AGENTS.md` are detected before the next model request; nested ones are not | | [V2INSTR] |
| Themes | V1 `tui.json` `theme` + `~/.config/opencode/themes/`; V2 `cli.json` → `theme: {name, mode}` | V2 CLI settings are one global file, no project layer | [V2CLICFG], [V1CFG] |
| Keybinds | V1 `tui.json`; V2 `cli.json` → `keybinds`, `leader.timeout` | `"none"`/`false` disables a binding | [V2CLICFG], [V1CFG] |
| Providers | `providers{}` (V2: `package`, `settings`, `headers`, `body`, `models`, `compatibility`) / `provider{}` (V1: `npm`, `api`, `options`) | `opencode auth login` writes `~/.local/share/opencode/auth.json` | [V2CFG], [MIGRATE], [V1CLI] |
| `disabled_providers` / `enabled_providers` | V1 arrays; V2 maps them to internal `policies` | `disabled_` wins over `enabled_` | [V1CFG], [MIGRATE] |
| Formatter | `formatter: true` or per-tool object `{disabled, command[], environment, extensions}` | omit = disabled | [V1CFG], [SCHEMA] |
| LSP | `lsp: true` or per-server `{command[], extensions, disabled, env, initialization}` | **V2 accepts and preserves but does not run LSP** | [V1CFG], [MIGRATE], [SCHEMA] |
| Policies (V2) | `policies: [{action, resource, effect}]`; actions `provider.use`, `tool.use`, `integration.use` | the only way to *enforce* restrictions against user edits | [V2CFG], [SCHEMA] |
| Misc V2 keys | `references{}`, `worktree.directory`, `warming`, `tool_output`, `media.image`, `websearch`, `compaction`, `snapshots`, `update`, `username`, `experimental{}` | | [V2CFG], [SCHEMA] |
| Not a config surface | `service.json` (service password), `opencode.db` + `-wal` (105 MB here), `log/`, `snapshot/`, `storage/`, `tool-output/` | app must not hand-edit; read-only at most | [local] |

---

## F. Hot reload vs restart; safe disable; JSONC risk

**Reload paths that exist** [local CLI], [V2PLUG], [V2CLICFG], [V2INSTR]:

| Change | Mechanism | Confidence |
|---|---|---|
| `config`/agents/skills/MCP | `opencode reload` — a real subcommand on 2.1.4 ("Reload configuration", accepts `--standalone`/`--server`) | verified in `--help` |
| Plugins / watched config dirs | "Changes under watched config directories reload automatically"; otherwise `opencode service restart` | documented [V2PLUG] |
| `cli.json` (terminal client) | "Valid edits reload while the TUI is running" | documented [V2CLICFG] |
| `AGENTS.md` | re-read before the next model request | documented [V2INSTR] |
| Everything else | background service restart: `opencode service restart` (subcommands `start/restart/status/stop/get/set/unset`, verified) | [local CLI], [V2CLI] |

A desktop app should treat writes as **two-phase**: write atomically → `opencode reload` (or
`service restart`) → re-read `debug config`/`debug agents`/`mcp list` and show the resolved state.
Never assume the app's own JSON view is the source of truth: the resolved view comes from
`opencode debug config` (documents + normalised info) and `opencode debug agents` (JSON array)
[local CLI].

**Safe disable patterns** (prefer over deletion — reversible, and survives a service restart):

| Artifact | Disable | Rationale |
|---|---|---|
| Agent | V2 `agents.<id>.disabled: true` / V1 `agent.<id>.disable: true` | documented switch ([V2AGENTS], [V1AGENTS]) |
| Skill | `permissions`/`permission.skill` → `deny` (hides from model) or frontmatter `metadata.opencode/autoinvoke: false` (hide, keep loadable) | [V2SKILLS], [V1SKILLS] |
| MCP | V2 `disabled: true` / V1 `enabled: false` | [V2MCP], [V1MCP] |

**JSONC risks when hand-editing** ([SCHEMA], [MIGRATE], [V2CFG]):

1. The schema explicitly allows comments and trailing commas (`"allowComments": true`,
   `"allowTrailingCommas": true`), and V2 reads `.jsonc`. A naive `JSON.parse` → `JSON.stringify`
   round-trip **destroys user comments and formatting** — use a comment-preserving JSONC parser, or
   refuse to write when comments are detected and offer a diff.
2. The root schema is `"additionalProperties": false` — one misspelled top-level key makes the
   document invalid. Validate before/after writing against `https://opencode.ai/config.json`
   (JSON Schema 2020-12).
3. The schema `$ref`s `https://models.dev/model-schema.json` for `model`-typed fields, so full
   offline validation needs that document vendored.
4. V2 **never rewrites** the source file — it normalises V1 → V2 in memory. So a V1-shaped file stays
   V1-shaped on disk while the app "sees" V2 values — so write back the same shape you read, or
   migrate deliberately and warn.
5. Malformed values / unsupported legacy fields / conflicting V1+V2 values emit warnings and the rest
   still loads; do not treat "it loaded" as "it was valid" ([MIGRATE]).
6. Back up, write atomically (temp + rename), keep a `.bak`, and never write while the background
   service is mid-restart.

---

## G. V2 creation — exact files, fields, commands

**Add a subagent** — one Markdown file (preferred) or one JSON entry.

```markdown
<!-- project: <repo>\.opencode\agents\reviewer.md
     global:  %USERPROFILE%\.config\opencode\agents\reviewer.md -->
---
description: Reviews changes for correctness and regressions
mode: subagent
model: anthropic/claude-sonnet-5-5#high
permissions:
  - action: edit
    resource: "*"
    effect: deny
  - action: shell
    resource: "*"
    effect: deny
---
Review the current changes. List findings in severity order with file and line references.
```
Minimum viable: `description` + `mode: subagent` (+ body). Source: [V2AGENTS], [MIGRATE].
JSON alternative — `agents.reviewer = {description, mode:"subagent", system, permissions:[…]}`
([V2AGENTS]). Creating a *primary* agent omits `mode` (V2 default is `primary`) or sets it explicitly;
then set `default_agent` if it should be the startup agent ([V2CFG], [V2AGENTS]).

**Add a skill** — one directory per skill; supporting files sit beside `SKILL.md` ([V2SKILLS]):

```
<root>\git-release\
├── SKILL.md            # required, all-caps
├── scripts/
└── references/
```
```markdown
---
name: git-release
description: Create consistent releases and changelogs
metadata:
  opencode/autoinvoke: true
---
## Workflow
1. Read `references/release-policy.md`. …
```
`<root>` = `<repo>\.opencode\skills` or `%USERPROFILE%\.config\opencode\skills` (compat:
`.claude\skills`, `.agents\skills`). ID = directory name; the frontmatter `name` is only a label
([V2SKILLS]). Register out-of-tree roots via `skills: []` ([V2SKILLS], [V2CFG]).

**Add an MCP server** — prefer the CLI so the same code path the app relies on does the write
([V2MCP], [local CLI]):

```
opencode mcp add context7 --url https://mcp.context7.com/mcp                 # project
opencode mcp add context7 --global --url https://mcp.context7.com/mcp        # global
opencode mcp add everything --env LOG_LEVEL=info -- npx -y @modelcontextprotocol/server-everything
opencode mcp add context7 --url … --header 'CONTEXT7_API_KEY={env:CONTEXT7_API_KEY}'
opencode mcp list                                                            # verify state
opencode mcp auth context7                                                   # OAuth sign-in
opencode mcp logout context7                                                 # revoke
```
Hand-written equivalent (V2, canonical):

```jsonc
{ "$schema": "https://opencode.ai/config.json",
  "mcp": { "servers": { "context7": {
    "type": "remote", "url": "https://mcp.context7.com/mcp",
    "headers": { "CONTEXT7_API_KEY": "{env:CONTEXT7_API_KEY}" },
    "disabled": false } } } }
```
Local command arrays on Windows should be explicit, as the machine's own config does:
`["cmd.exe","/d","/c","npx","-y","pkg","mcp"]` [local:\Users\raosu\.config\opencode\opencode.json].

**Validation & verification loop.** `$schema` field → editor/validator; `opencode debug config`
(shows documents + normalised values), `opencode debug agents` (JSON), `opencode debug paths`,
`opencode mcp list` [local CLI]. `opencode api` exists for calls against the running server
(`opencode api --help`) and plugins can read the same registries via `ctx.agent.list()`,
`ctx.skill.list()`, `ctx.mcp.list()` ([V2PLUGAPI]) — the cleanest way to get *live* resolved state
instead of re-implementing precedence.

**Auth steps.** Providers: `opencode auth login` → `~/.local/share/opencode/auth.json` ([V1CLI]).
Remote MCP: OAuth by default (CIMD → DCR → pre-registered client precedence), tokens stored outside
project config; `opencode mcp auth <name>` opens a browser, `opencode mcp logout` clears it
([V2MCP], [V1MCP]).

---

## H. Version / stability

| Fact | Detail | Source |
|---|---|---|
| Version on this machine | desktop `OpenCode.exe` + bundled `opencode-cli.exe` = **2.1.4** (`resources/opencode-cli.version`); logs show `version=2.1.4 … args=["serve","--service"]` and an older `version=1.1.65` | [local], [local CLI] |
| Repo | `github.com/sst/opencode` **redirects to** `github.com/anomalyco/opencode` (branch `dev`, `specs/v2` dir, `V2_HTTP_API_AUDIT.md`) | [GH] |
| Docs split | V1 pages at `opencode.ai/docs/*`, V2 at `opencode.ai/v2/docs/*`; a "New OpenCode v2 is available" banner sits on V1 pages | [V1CFG], [V2CFG] |
| Published schema (one URL) | `https://opencode.ai/config.json` — JSON Schema draft 2020-12, `$id` = that URL. **Current content is the V2 schema with every V1 key present but `"deprecated": true`** (verified by fetching it) | [SCHEMA] |
| V2 CLI-settings schema | `https://opencode.ai/v2/cli.json` (V1 used `https://opencode.ai/tui.json`) | [V2CLICFG], [V1CFG] |
| Backward compatibility | V2 reads V1 system/global/project config "from the same locations" and normalises in memory without rewriting; supported V1 files (agents, commands, skills, `.opencode/`) keep working | [MIGRATE] |
| Verified compatibility | V1-shaped `mcp.rea.enabled:true` on disk → `debug config` output `mcp.servers.rea.disabled:false` | [local CLI] |

**Shape drift to code against** ([MIGRATE], [SCHEMA]):

| Concern | V1 | V2 |
|---|---|---|
| Agent map | `agent` | `agents` |
| Agent prompt / off-switch / perms | `prompt` / `disable` / `permission` | `system` / `disabled` / `permissions` (ordered array) |
| MCP | `mcp.<name>`, `enabled`, scalar `timeout` | `mcp.servers.<name>`, `disabled`, `timeout{startup,catalog,execution}`, `codemode`, `protocol` |
| MCP OAuth keys | `clientId`, `clientSecret`, `callbackPort`, `redirectUri` | `client_id`, `client_secret`, `callback_port`, `redirect_uri`, `auth_server_metadata_url` |
| Commands / providers / plugins / references | `command`, `provider`, `plugin`, `reference` | `commands`, `providers`, `plugins`, `references` |
| Skills sources | `{paths, urls}` | array |
| Permissions | `permission` + `tools` (`bash`, `task`, `write`, `patch`) | `permissions` actions (`shell`, `subagent`, `edit`) |
| Snapshots / media | `snapshot`, `attachment` | `snapshots`, `media` |
| Compaction | `preserve_recent_tokens`, `reserved` | `keep.tokens`, `buffer` (V2 has no `tail_turns`/`prune`) |
| Fields ignored in V2 | — | `logLevel`, `server`, top-level `subagent_depth`, agent `name`, an MCP entry with no `type`, `lsp` execution, V1 provider fields `id`/`whitelist`/`blacklist` |

Two features are **not** stable surfaces: `experimental{}` ("may change or be removed without notice",
[V2CFG]) and arbitrary extra provider options passed through per-agent ([V1AGENTS]).

---

## I. Windows specifics

| Topic | Finding | Source |
|---|---|---|
| Global config dir | `C:\Users\raosu\.config\opencode` — dot-config under the user profile, **not** `%APPDATA%` (`C:\Users\raosu\AppData\Roaming\opencode` exists but holds only `EBWebView` webview data) | [local CLI], [local] |
| Data / cache / state / tmp | `C:\Users\raosu\.local\share\opencode`, `C:\Users\raosu\.cache\opencode`, `C:\Users\raosu\.local\state\opencode`, `C:\Users\raosu\AppData\Local\Temp\opencode`; db `…\.local\share\opencode\opencode.db` | [local CLI] `debug paths` |
| XDG | `XDG_CONFIG_HOME` **is** honoured on Windows (verified: config path moved to `<xdg>\opencode`) | [local CLI], [V2CLICFG] |
| System/managed dir | `%ProgramData%\opencode` documented; **absent** on this machine | [V2CFG], [local] |
| Desktop app layout | `%LOCALAPPDATA%\Programs\@opencodedesktop\OpenCode.exe` (Electron), `resources\opencode-cli.exe` (246 MB sidecar CLI), `resources\opencode-cli.version`, `resources\app.asar`, `app-update.yml` → `https://opencode.ai/update/api/latest/desktop/opencode/`, updater cache `%LOCALAPPDATA%\@opencodedesktop-updater` | [local], [GH] |
| Shared background service | app spawns `opencode serve --service` (log); CLI defaults to that shared server and only uses a private one with `--standalone`/`--server <url>`; `opencode service set disabled true` opts out | [local], [V2CLI] |
| Service secret | `service.json` in the config dir holds `{"password": "…"}` — treat as a secret, never log; it authenticates clients to the background service | [local:\Users\raosu\.config\opencode\service.json], [V2CLI] |
| Bun / runtime | npm plugins are installed **at startup using Bun** and cached in `~/.cache/opencode/node_modules` (absent here — this machine has no npm plugins); local plugins run under Bun and receive Bun's shell `$` | [V1PLUG], [local] |
| Bun-on-Windows gotcha (real) | `[local:…\.config\opencode\plugins\rtk.ts]` guards with `await $\`which rtk\``; `which` does not exist on Windows, so the `catch` branch returns `{}` and the plugin silently does nothing. A manager UI should surface plugin load errors/warnings rather than assume "file present = active" | [local:\Users\raosu\.config\opencode\plugins\rtk.ts] |
| CLI install paths | curl installer → `$HOME/.opencode/bin`; npm `@opencode/cli`; desktop installer → `%LOCALAPPDATA%\Programs\@opencodedesktop`. `OPENCODE_GIT_BASH_PATH` exists for Windows shells; `OPENCODE_DISABLE_FFF` is on by default on Windows | [GH], [V1CLI] |
| JSON path escaping | Windows paths in config must escape backslashes (`"C:\\Users\\…"`) or use forward slashes; the resolved `plan` agent permission on this machine shows the escaped form | [local CLI] `debug agents` |
| Encoding | read/write OpenCode's `.md`/`.ts`/`.json` files as UTF-8; PowerShell 5.1 `Get-Content -Raw` defaults to ANSI and mangles non-ASCII (observed while inspecting `rtk.ts`) | [local] |
| `debug config` quirk | with `XDG_CONFIG_HOME` set, `debug paths` moved but `debug config` still reported the home `.config/opencode/opencode.json` document — do not build path logic on `debug config` alone | [local CLI] |

---

## Unverified

1. **Whether `agents.<id>.disabled: true` disables a *file-defined* agent of the same ID.** Docs say
   `disabled` removes a built-in or custom agent "at that point in configuration loading"
   ([V2AGENTS]); no statement about suppressing a Markdown file. The safe V1-app action is to
   delete/move the `.md` file, or add the JSON entry and verify with `opencode debug agents`.
2. **Rename semantics for agents and skills.** Neither doc defines a rename; both derive the ID from
   the path ([V1AGENTS], [V2SKILLS]). A rename therefore looks like delete + create, and any
   references (`permissions[].resource`, `default_agent`) must be rewritten.
3. **MCP `timeout` default.** [V2MCP] says 30 s startup / 30 s catalog / 12 h execution; the V1
   section of the published schema says "Defaults to 5000 (5 seconds)" for `McpLocalConfig.timeout`
   ([SCHEMA]). Not resolved by testing.
4. **V2 location of MCP OAuth credentials.** V1 doc names `~/.local/share/opencode/mcp-auth.json`
   ([V1MCP]); V2 only says "stored outside project configuration" ([V2MCP]). No `mcp-auth.json`
   exists on this machine (no OAuth MCP configured), so the path could not be confirmed.
5. **Automatic reload of `opencode.json(c)` itself.** Automatic reload is documented for plugin
   directories ([V2PLUG]), `cli.json` ([V2CLICFG]) and `AGENTS.md` ([V2INSTR]); for the main config the
   docs are silent and the shipped escape hatch is `opencode reload` / `opencode service restart`.
   Not tested by mutating files (would modify `C:\Users\raosu`).
6. **`opencode service.json` on macOS/Linux.** Present in the Windows config dir with a `password`
   key and undocumented on the docs site; its portability is unknown.
7. **Whether `opencode mcp auth`/`disconnect` state is stored per-scope or per-user**, and whether
   `logout` also clears a `headers`-based credential (it should not — headers live in the JSON).
8. **`opencode debug skill`** existed in V1 docs; the 2.1.4 CLI exposes `debug agents|config|
   paths|heap-dump` only, so skill enumeration from a desktop app must read the filesystem or use the
   server API/plugin `ctx.skill.list()`.
9. **Exact on-disk formatting produced by `opencode mcp add`** (whether it rewrites the file, sorts
   keys, or preserves comments) — deliberately not run against the user's live config.
10. **`%ProgramData%\opencode` behaviour on Windows** (managed/system config) — documented in both
    doc generations, directory absent here, so precedence/`policies` enforcement was not observed.

---

## Sources

| Tag | URL / path |
|---|---|
| [V1CFG] | https://opencode.ai/docs/config/ (V1 config: locations, 8-tier precedence, JSONC, schema URLs, `autoupdate`, formatter/lsp, themes/keybinds) |
| [V2CFG] | https://opencode.ai/v2/docs/config/ (V2 config: directory-walk merge order, system/managed dirs, `agents`/`mcp.servers`/`permissions`/`policies`/`skills`) |
| [V1AGENTS] | https://opencode.ai/docs/agents/ |
| [V2AGENTS] | https://opencode.ai/v2/docs/agents/ |
| [V1SKILLS] | https://opencode.ai/docs/skills/ |
| [V2SKILLS] | https://opencode.ai/v2/docs/skills/ |
| [V1MCP] | https://opencode.ai/docs/mcp-servers/ |
| [V2MCP] | https://opencode.ai/v2/docs/mcp-servers/ |
| [V1PLUG] | https://opencode.ai/docs/plugins/ (npm plugins installed via Bun, `~/.cache/opencode/node_modules`, load order) |
| [V2PLUG] | https://opencode.ai/v2/docs/plugins/ (configure/discover/control/reload, `opencode plugin add` / `list` / `check` / `update` / `remove`) |
| [V2PLUGAPI] | https://opencode.ai/v2/docs/build/plugins/ (plugin context: `agent`, `skill`, `mcp`, `tool` transforms + `list()`) |
| [V1RULES] | https://opencode.ai/docs/rules/ (AGENTS.md, `CLAUDE.md` fallback, `instructions[]`) |
| [V2INSTR] | https://opencode.ai/v2/docs/instructions/ (AGENTS.md only, discovery order, hot edit detection) |
| [V1CLI] | https://opencode.ai/docs/cli/ (`agent create`/`list`, `mcp add`/`list`/`auth`/`logout`/`debug`, `auth`, env var table, `debug config` / `paths` / `info`) |
| [V2CLI] | https://opencode.ai/v2/docs/cli/ (shared background service, `--standalone`/`--server`, `service set disabled`, `debug paths <selector>`, uninstall semantics) |
| [V2CLICFG] | https://opencode.ai/v2/docs/cli/config/ (`~/.config/opencode/cli.json`, `$XDG_CONFIG_HOME`, theme/keybinds, schema `https://opencode.ai/v2/cli.json`) |
| [MIGRATE] | https://opencode.ai/v2/docs/migrate-v1/ (V1→V2 field mapping, `mcp.servers`, bounded nested mixing, agent/skill/command/plugin file dirs, ignored fields) |
| [SCHEMA] | https://opencode.ai/config.json (fetched; draft 2020-12, `additionalProperties:false`, V1 keys `deprecated:true`, `McpLocalConfig`, `Mcp.RemoteConfigEncoded`, `Config.AgentEncoded`, `allowComments`/`allowTrailingCommas`) |
| [GH] | https://github.com/sst/opencode → redirects to https://github.com/anomalyco/opencode (install methods, desktop app downloads, `specs/v2`) |
| local | `C:\Users\raosu\.config\opencode\opencode.json` (V1 `mcp.rea`, `cmd.exe /d /c npx …`), `package.json` (`@opencode-ai/plugin` 1.14.29), `bun.lock` (1.2.6), `service.json`, `plugins\rtk.ts`, `node_modules\@opencode-ai\{plugin,sdk}`; `C:\Users\raosu\.local\share\opencode\{opencode.db,log,bin,storage,snapshot,repos}`; `C:\Users\raosu\.cache\opencode\bin`; `C:\Users\raosu\AppData\Local\Programs\@opencodedesktop\resources\{opencode-cli.exe,opencode-cli.version,app-update.yml}`; no `opencode.json`/`.opencode` found under `C:\Users\raosu\SuryapalDev` (depth 3) |
| local CLI | `C:\Users\raosu\AppData\Local\Programs\@opencodedesktop\resources\opencode-cli.exe` → `--help`, `mcp --help`, `mcp add --help`, `mcp list --help`, `plugin --help`, `service --help`, `debug --help`, `reload --help`, `debug paths`, `debug config`, `debug agents` (with and without `XDG_CONFIG_HOME`) |

Docs note: the "New OpenCode v2 is available" banner on `opencode.ai/docs/*` marks those pages as the
legacy (V1) documentation; `opencode.ai/v2/docs/*` is current. The single schema URL
`opencode.ai/config.json` now serves the V2 schema with V1 keys retained as `deprecated`.
