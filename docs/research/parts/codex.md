# OpenAI Codex — on-disk configuration surface

Scope: what a Windows desktop app must read/write to **list, enable/disable, remove (V1)** and
**create (V2)** subagents, skills, and MCP servers for Codex at **project** and **global** scope.

Inspected machine (ground truth): `codex-cli 0.164.0-alpha.11.1` at
`C:\Users\raosu\AppData\Local\OpenAI\Codex\bin\2993767bd1dfb009\codex.exe` (not on `PATH`;
path discovered from `CODEX_CLI_PATH` inside the live config). `CODEX_HOME` = `C:\Users\raosu\.codex`.

Citation tags are expanded in **Sources**. `[local:…]` = absolute path on this machine.

---

## Path inventory

| Artifact | Project scope | Global scope (Windows) | macOS / Linux | Format | Source |
|---|---|---|---|---|---|
| Main config | `<repo>/.codex/config.toml` (trusted projects only; walks root→CWD, closest wins) | `%USERPROFILE%\.codex\config.toml` | `~/.codex/config.toml` | TOML | [BASIC], [ADV], [local:C:\Users\raosu\.codex\config.toml] |
| Profile overlay | — | `%USERPROFILE%\.codex\<profile>.config.toml` | `~/.codex/<profile>.config.toml` | TOML | [ADV] |
| System config | — | *not documented for Windows* (unverified) | `/etc/codex/config.toml` | TOML | [BASIC] |
| Admin policy | — | system `requirements.toml` (path unverified) | system `requirements.toml` | TOML | [REF], [BASIC] |
| Global instructions | `AGENTS.md` + `AGENTS.override.md` in each dir root→CWD | `%USERPROFILE%\.codex\AGENTS.md` (+ `AGENTS.override.md`) | `~/.codex/AGENTS.md` | Markdown | [AGENTS], [local:…\.codex\AGENTS.md] |
| **Subagent (custom agent)** | `<repo>/.codex/agents/<name>.toml` | `%USERPROFILE%\.codex\agents\<name>.toml` | `~/.codex/agents/<name>.toml` | TOML (1 file = 1 agent) | [SUB], [local:…\.codex\agents\*.toml] |
| **Skill** | `.agents/skills/<skill>/SKILL.md` in CWD, any ancestor, and repo root | `%USERPROFILE%\.agents\skills\<skill>\SKILL.md` — **note: `.agents`, not `.codex`** | `~/.agents/skills/…`; admin `/etc/codex/skills`; bundled `.codex/skills/.system` | Markdown + YAML frontmatter | [SKILLS], [CUSTOM], [local:…\.codex\skills\.system\*] |
| **MCP server** | `.codex/config.toml` → `[mcp_servers.<id>]` | `config.toml` → `[mcp_servers.<id>]` | same | TOML sub-table | [MCP], [REF] |
| Plugin enable/disable | `.codex/config.toml` → `[plugins."<name>@<marketplace>"]` | `config.toml` | same | TOML | [REF], [local:…\.codex\config.toml] |
| Plugin marketplace | `[marketplaces.<name>]` (source_type/source/ref/sparse_paths) | `config.toml` | same | TOML | [REF] |
| Hooks | `<repo>/.codex/hooks.json` or inline `[hooks]` | `%USERPROFILE%\.codex\hooks.json` or inline `[hooks]` | same | JSON / TOML | [ADV] |
| Rules (execpolicy) | `<repo>/.codex/rules` (trusted only) | `%USERPROFILE%\.codex\rules` | `~/.codex/rules` | file format undocumented here | [ADV], [CMDS] |
| Project trust | n/a (lives in user config) | `config.toml` → `[projects.'<abs path>'] trust_level = "trusted"` | same | TOML | [REF], [local:…\.codex\config.toml] |
| Custom prompts | — | **removed in 0.117.0** (`~/.codex/prompts/*.md` no longer read) | removed | — | [GH15941] |
| Auth / state | — | `auth.json`, `history.jsonl`, `state_*.sqlite`, `logs_*.sqlite`, `sessions/` | same | mixed | [ADV], [ENV], [local listing] |
| Desktop app settings | — | `config.toml` → `[desktop]` (user-level only for some keys) | same | TOML | [REF], [local:…\.codex\config.toml] |

**Windows app note:** the ChatGPT desktop app on Windows uses `%USERPROFILE%\.codex` — the *same*
Codex home as the native CLI ([WINAPP]). There is no separate app-only config root.

---

## A. Config roots and precedence

**Roots.** `CODEX_HOME` sets the root for all state (config, auth, logs, sessions, skills); default
`~/.codex`, and *if you set it the directory must already exist* ([ENV]). Project overrides live in
`.codex/config.toml` inside the repo — **project config files are first-class**, not just `AGENTS.md`
([BASIC], [ADV]). Codex walks project root → CWD and loads every `.codex/config.toml`, closest wins
([ADV]).

**Precedence, highest first** ([BASIC]):

1. CLI flags and `-c/--config key=value` overrides
2. Project `.codex/config.toml` layers (root → CWD, closest wins; **trusted projects only**)
3. Profile file `~/.codex/<profile>.config.toml` (selected by `--profile`)
4. User `~/.codex/config.toml`
5. Cloud-managed `config.toml` defaults (workspace)
6. System `/etc/codex/config.toml` (Unix)
7. Built-in defaults

**Project layer is gated on trust.** Untrusted projects skip the whole `.codex/` layer — config,
hooks, and rules ([BASIC], [REF `projects.<path>.trust_level`]). Trust is recorded in the *user*
config, e.g. `[projects.'c:\users\raosu\suryapaldev\projects\company-os'] trust_level = "trusted"`
([local:…\.codex\config.toml]).

**Project config cannot set these keys** (ignored, with a startup warning): `openai_base_url`,
`chatgpt_base_url`, `apps_mcp_product_sku`, `model_provider`, `model_providers`, `notify`, `profile`,
`profiles`, `experimental_realtime_ws_base_url`, `otel` ([ADV], [REF]). Relative paths inside a
project config resolve against the `.codex/` folder that contains it ([ADV]).

**Env overrides.** `CODEX_HOME`, `CODEX_SQLITE_HOME`, `CODEX_API_KEY`, `CODEX_ACCESS_TOKEN`,
`CODEX_CA_CERTIFICATE`, `RUST_LOG`, plus installer vars ([ENV]). Per-run config override is
`-c foo.bar=value` (value parsed as TOML; falls back to literal string) and `--enable/--disable
<feature>` which is sugar for `-c features.<name>=true|false` (local `codex --help`, [CMDS]).

**Merge semantics.** Layer values override key-by-key; e.g. `shell_environment_policy.filters` keys
merge case-insensitively across layers, while `filter`/`exclude`/`include_only` forms must not be
mixed in one layer ([ADV]). `[features]` keys are per-key overrides; omitted keys keep defaults
([BASIC]).

---

## B. Subagents (`~/.codex/agents/*.toml`)

**Status: first-class and officially documented (stable).** Multi-agent tools are stable and on by
default (`features.multi_agent = true`) ([REF], [BASIC]); custom agents are a documented feature, not
a plugin/community hack ([SUB]). Availability: "Current Codex releases enable subagent workflows by
default" for app/CLI/IDE ([SUB]).

**Discovery roots** ([SUB]): `~/.codex/agents/` (personal) and `.codex/agents/` (project-scoped).
One file = one agent. **Codex identifies an agent by its `name` field, not its filename** — matching
filename to name is only a convention ([SUB]).

**Real files on this machine** (`[local:C:\Users\raosu\.codex\agents\code-reviewer.toml]`,
`red-team.toml`). Both use exactly the documented minimum field set:

```toml
name = "code-reviewer"
description = "Review completed implementation work against requirements, correctness, and production readiness."
developer_instructions = """
You are a senior code reviewer. ...
"""
```

`red-team.toml` is the same 3-field shape with a ~6 KB `developer_instructions` block. Neither file
sets `model` or `model_reasoning_effort`.

**File schema** ([SUB] § Custom agent file schema):

| Field | Type | Required | Notes |
|---|---|---|---|
| `name` | string | **Yes** | Identity used when spawning; source of truth ([SUB]) |
| `description` | string | **Yes** | Guidance shown to the model when choosing a role ([SUB], [REF]) |
| `developer_instructions` | string | **Yes** | Core behaviour ([SUB]) |
| `model`, `model_reasoning_effort`, `model_reasoning_summary`, `model_verbosity` | string | No | File value wins over `[agents]` defaults ([SUB]) |
| `features.shell_tool`, `features.apps`, `features.plugins`, `features.memories`, `features.request_permissions_tool` | bool | No | Set `false` to reduce capability ([SUB]) |
| `skills.config` (`[[skills.config]] path/enabled`), `skills.bundled.enabled`, `skills.include_instructions` | | No | Per-agent skill gating ([SUB]) |

**Cannot be set in an agent file** — ignored; inherited from the parent: `sandbox_mode`,
`approval_policy`, `model_provider`, `mcp_servers`, service tier ([SUB]). "Configure MCP servers in
the parent session so the agent can use them" ([SUB]).

**Built-ins** ([SUB]): `default` (general fallback), `worker` (implementation), `explorer`
(read-heavy exploration). A custom agent whose `name` matches a built-in **takes precedence**.

**Global table `[agents]`** in `config.toml` ([SUB], [REF]): `agents.enabled` (bool, default true —
disables all multi-agent tools), `agents.max_concurrent_threads_per_session` (alias
`agents.max_threads`), `agents.default_subagent_model`, `agents.default_subagent_reasoning_effort`,
`agents.interrupt_message`. Scalar setting names are reserved and cannot be used as role names
([REF]), and `[agents.<name>.config_file]` / `[agents.<name>.description]` also exist as an
alternative inline role declaration ([REF]) — *the newer standalone-TOML path and the inline
`agents.<name>` tables are both documented; see Unverified for their interaction.*

**Disable / remove semantics.** There is **no per-file `enabled` flag documented** for a standalone
agent TOML — removal = delete/rename the file; global off-switch = `agents.enabled = false` or
`features.multi_agent = false` ([SUB], [REF]). There is also no documented `codex agents`
create/edit CLI — `codex agents` only browses sessions on the shared app-server daemon (local
`codex --help`).

---

## C. Skills

**Supported, first-class, and the officially recommended replacement for custom prompts** ([SKILLS],
[CUSTOM], [GH15941]).

**Format.** A skill = a directory containing `SKILL.md` plus optional `scripts/`, `references/`,
`assets/`, and `agents/openai.yaml`; `SKILL.md` **must** declare `name` and `description` in YAML
frontmatter ([SKILLS]):

```markdown
---
name: skill-name
description: Explain exactly when this skill should and should not trigger.
---
```

Verified on disk at `[local:C:\Users\raosu\.codex\skills\.system\skill-creator\SKILL.md]` (bundled
`.system` skills: `imagegen`, `openai-docs`, `review-agent`, `skill-creator`, `skill-installer`). The
frontmatter on this machine also carries a nested `metadata.short-description` key.

**Discovery roots** ([SKILLS] table): `REPO` = `$CWD/.agents/skills`, every ancestor up to repo root,
`$REPO_ROOT/.agents/skills`; `USER` = `$HOME/.agents/skills`; `ADMIN` = `/etc/codex/skills`;
`SYSTEM` = bundled with Codex. Symlinked skill folders are followed. Duplicate `name`s are **not**
merged — both appear in the picker ([SKILLS]).

> **Critical for the app:** user-scope skills live in `%USERPROFILE%\.agents\skills` — **not**
> `%USERPROFILE%\.codex\skills`. `.codex\skills` on this machine holds only bundled `.system` skills.
> Writing new user skills to `.codex\skills` is off-spec. There is no documented Windows admin path
> equivalent to `/etc/codex/skills` (see Unverified).

**Disable without deleting** — `[[skills.config]]` in `~/.codex/config.toml` ([SKILLS], [REF]):

```toml
[[skills.config]]
path = "/path/to/skill/SKILL.md"
enabled = false
```

`skills.config` is an array of `{ path, enabled }` objects keyed by path to `SKILL.md` ([REF]).
`skills.max_context_tokens` (default 2% of context, capped at 10000) bounds the initial skill catalog
([SKILLS], [REF]). Implicit invocation can be suppressed per skill via
`agents/openai.yaml` → `policy.allow_implicit_invocation: false` ([SKILLS]).

**Hot detection.** "Codex detects skill changes automatically. If an update doesn't appear, restart
Codex." ([SKILLS]) — but the disable-list doc says "Restart Codex after changing
`~/.codex/config.toml`" ([SKILLS]).

---

## D. MCP

Config lives in `config.toml` under `[mcp_servers.<id>]`, global or project-scoped; the desktop app,
CLI, and IDE extension **share** this config for the same host ([MCP]).

### D.1 Field surface (from [REF])

| Field | Type | Notes |
|---|---|---|
| `command` | string | stdio launcher (required for stdio) |
| `args` | array<string> | stdio args |
| `env` | map<string,string> | env vars forwarded to the server |
| `env_vars` | array<string \| {name, source="local"\|"remote"}> | **allowlist** of names to forward; orthogonal to `env` |
| `cwd` | string | working dir for stdio process |
| `url` | string | streamable HTTP endpoint |
| `bearer_token_env_var` | string | env var holding bearer token (HTTP) |
| `http_headers` | map<string,string> | static headers |
| `env_http_headers` | map<string,string> | header name → env var name |
| `http_headers_helper` | string (command) | local command printing a JSON header map; local HTTP only |
| `auth` | `oauth` \| `chatgpt` | fallback auth for HTTP |
| `oauth.client_id`, `oauth.client_secret`, `oauth.callback_url`, `oauth.callback_port`, `scopes`, `oauth_resource` | | pre-registered OAuth client |
| `startup_timeout_sec` / `startup_timeout_ms` | number | default 10 s |
| `tool_timeout_sec` | number | default 60 s |
| `enabled` | bool | **disable without removing** — [REF]: "Disable an MCP server without removing its configuration" |
| `required` | bool | fail startup/resume if enabled server can't init |
| `enabled_tools` / `disabled_tools` | array<string> | allow list then deny list |
| `default_tools_approval_mode`, `tools.<tool>.approval_mode` | `auto\|prompt\|writes\|approve` | |
| `tools.<tool>.output_token_limit` | integer | |
| `experimental_environment` | `local` \| `remote` | remote stdio placement (streamable-HTTP remote not implemented) |

Plugin-bundled servers are controlled separately via
`[plugins.<plugin>.mcp_servers.<server>]` → `enabled`, `enabled_tools`, `disabled_tools`,
`default_tools_approval_mode`, `tools.<tool>.approval_mode`; transport comes from the plugin manifest,
not user config ([MCP], [REF]).

### D.2 Real entries on this machine (secrets redacted)

`[local:C:\Users\raosu\.codex\config.toml]` — two stdio servers, both hand-appended to the global
config:

```toml
[mcp_servers.node_repl]
args = []
command = 'C:\Users\raosu\AppData\Local\OpenAI\Codex\runtimes\cua_node\1fb7137c46617d50\bin\node_repl.exe'
env_vars = ["BROWSER_USE_ACCESS_EXECUTOR_ID", "BROWSER_USE_ACCESS_PIPE_PATH", "BROWSER_USE_ACCESS_VERIFYING_IDENTITY", "CODEX_WINDOWS_REGISTERED_CORE"]
startup_timeout_sec = 120

[mcp_servers.node_repl.env]
NODE_REPL_NODE_PATH = 'C:\Users\raosu\AppData\Local\OpenAI\Codex\runtimes\cua_node\1fb7137c46617d50\bin\node.exe'
CODEX_HOME = 'C:\Users\raosu\.codex'
BROWSER_USE_AVAILABLE_BACKENDS = "chrome,iab,mcpapps"
# ~10 further env keys, redacted

[mcp_servers.rea]
command = "cmd.exe"
args = [ "/d", "/c", "npx", "-y", "rea-agents@6.8.0", "mcp" ]
startup_timeout_sec = 30
tool_timeout_sec = 450
```

Note: `env_vars` (the allowlist) and `[mcp_servers.<id>.env]` (explicit values) are used *together* in
one entry — confirming they are independent mechanisms. `codex mcp list` on this machine shows four
servers (`codex_app` *disabled*, `cua_repl`, `node_repl`, `rea` *enabled*) — i.e. the list is the
*merged effective* view including plugin-provided servers, not a raw dump of `config.toml`
(local `codex mcp list`, [MCP]).

### D.3 CLI

`codex mcp` = `list | get | add | remove | login | logout` (local `codex mcp --help`; [CMDS]: "Manage
MCP server entries stored in `~/.codex/config.toml`").

```text
codex mcp add <NAME> (--url <URL> | -- <COMMAND>...)   [--env K=V]... [--bearer-token-env-var E]
                                                       [--oauth-client-id ID] [--oauth-client-secret S]
                                                       [--oauth-resource R]
codex mcp list [--json]
codex mcp get <NAME>
codex mcp remove <NAME>
```

Behavior: **write path goes to global `~/.codex/config.toml` only** ([CMDS]). `--json` on `list` is
the machine-readable surface to build a UI on (local `codex mcp list --help`).

**`--scope` does not exist in this version.** `codex mcp add --help` at 0.164.0-alpha.11.1 offers no
`--scope`/`--project`/`--global` flag (local `codex mcp add --help`); the docs likewise document only
`~/.codex/config.toml` for the CLI and tell you to hand-edit a project-scoped `.codex/config.toml`
for per-project servers ([MCP], [CMDS]). Therefore: **project-scope MCP creation must be done by
editing `<repo>/.codex/config.toml` directly** — see G for the safe procedure.

### D.4 Precedence & scope

Project `.codex/config.toml` layers merge over user config (layer order in A) but only for trusted
projects ([BASIC], [ADV]). Server identity is the table key `<id>`; an admin allowlist can pin
`mcp_servers.<id>.identity` (command or url, with exact/prefix/regex arg matchers) and any configured
server not in the allowlist is disabled ([REF] `requirements.toml`).

### D.5 Disable vs delete

- **Disable:** `enabled = false` on the table (documented explicitly as "without removing its
  configuration") ([REF], [MCP]). Non-destructive, reversible, and the preferred V1 "toggle".
- **Delete:** `codex mcp remove <name>` (CLI, global config) or removing the `[mcp_servers.<id>]`
  table/keys by hand (project config) ([CMDS], [MCP]).
- **Per-tool narrowing** is possible without disabling the server: `enabled_tools` / `disabled_tools`
  ([MCP]).

---

## E. Other artifacts (one line each)

- **AGENTS.md hierarchy:** global `~/.codex/AGENTS.override.md` else `AGENTS.md` (first non-empty only), then per-directory `AGENTS.override.md` → `AGENTS.md` → `project_doc_fallback_filenames`, root→CWD, concatenated with blank lines, capped by `project_doc_max_bytes` (32 KiB default) ([AGENTS], [REF]).
- **Project root detection:** nearest dir containing a `project_root_markers` entry, default `.git`; `[]` = stop at CWD ([ADV], [REF]).
- **Custom prompts:** **removed in 0.117.0** — `~/.codex/prompts/*.md` is dead; convert to skills ([GH15941]). No `prompts` directory exists on this machine ([local listing]).
- **Profiles:** `~/.codex/<profile>.config.toml` selected by `--profile`; since 0.134.0 the legacy `[profiles.<name>]` table and top-level `profile =` selector are no longer read ([ADV]).
- **Plugins:** `[plugins."<name>@<marketplace>"] enabled = true|false`, installed/removed by `codex plugin add|remove`, listed by `codex plugin list --json` ([REF], [CMDS], local `codex plugin --help`).
- **Marketplaces:** `[marketplaces.<name>] source_type = "git"|"local"`, `source`, optional `ref`/`sparse_paths`; may be defined in system, cloud-managed, user, or trusted-project config ([REF]).
- **notify hook:** top-level `notify = [argv...]`; receives a JSON payload (currently only `agent-turn-complete`) ([ADV], [REF]); this machine: `notify = [ "C:\\Users\\raosu\\…\\codex-computer-use.exe", "turn-ended" ]` ([local]).
- **Model/provider:** `model`, `model_reasoning_effort`, `service_tier`, `model_provider` + `[model_providers.<id>]` (`openai`/`ollama`/`lmstudio` reserved; `wire_api = "responses"` only) ([ADV], [REF]).
- **Hooks:** `~/.codex/hooks.json`, `<repo>/.codex/hooks.json`, or inline `[hooks]`; `features.hooks` stable ([ADV], [REF]).
- **Rules/execpolicy:** `~/.codex/rules`, validatable via `codex execpolicy check --rules <file>` (preview) ([CMDS]).
- **Desktop app keys:** `[desktop]` (and `desktop.custom_file_handlers.<id>`, user-level only) already live in the same user `config.toml` ([REF], [local]).

---

## F. Mutation, hot reload, and hand-editing risk

| Artifact | When re-read | Evidence |
|---|---|---|
| `config.toml` | at process/session start; changing it generally needs a restart | "Restart Codex after changing `~/.codex/config.toml`" ([SKILLS]); app: "Save the server, then select **Restart**" for MCP ([MCP]) |
| `AGENTS.md` chain | rebuilt every run / every TUI session; "there is no cache to clear" | [AGENTS] |
| Skills | detected automatically; restart only as fallback | [SKILLS] |
| Agent TOMLs / MCP servers | loaded as part of session config at start; no documented live-watcher | [SUB], [MCP] (inference — see Unverified) |

**Who writes config.toml.** The CLI/app persist settings into `config.toml` themselves:
`codex features enable|disable` "persist changes so they apply to future sessions" ([CMDS]);
`/keymap`, `/statusline`, `/title`, `/theme`, `/experimental` all persist into `config.toml`
([CMDS]). Observed consequence on this machine: the live `config.toml` has **no comments at all and
long runs of blank lines** where commented-out keys used to be, plus a sibling
`config.toml.rea.backup` — i.e. programmatic round-trips drop comments and leave whitespace
([local:C:\Users\raosu\.codex\config.toml]). **A Windows app that rewrites this file must preserve
comments and key order itself, or it will silently strip the user's annotations.**

**Unknown keys.** Silent tolerance by default: `--strict-config` exists specifically to "error out
when config.toml contains fields that are not recognized by this version" (local `codex --help`).
Without it, typos and future/renamed keys are accepted and ignored — a mistyped `enabled` key will
*not* disable a server, the server simply stays enabled.

**Safe disable semantics (V1)** — prefer these over deletes:

| Kind | Safe disable | Source |
|---|---|---|
| MCP server | `mcp_servers.<id>.enabled = false` (or `/mcp` server toggle in app UI) | [REF], [MCP] |
| MCP tools subset | `enabled_tools` / `disabled_tools` | [MCP] |
| Skill | `[[skills.config]] path=…/SKILL.md, enabled=false` | [SKILLS] |
| Plugin | `[plugins."x@mkt"] enabled = false` | [REF] |
| All subagents | `agents.enabled = false` or `features.multi_agent = false` | [SUB], [REF] |
| Single subagent | no documented flag → move file out of `agents/` | [SUB] (absence) |

**Verification hooks for a mutator:** `codex mcp list --json`, `codex mcp get <name>`,
`codex doctor` ("installation, config, auth, runtime health"), `/debug-config` (prints layer order and
policy sources), `--strict-config` (fail on unknown keys) ([CMDS], local `codex --help`).

---

## G. V2 creation

### G.1 MCP server — prefer the CLI

```powershell
# global, stdio
codex mcp add myserver --env API_KEY=abc -- npx -y some-mcp-server
# global, streamable HTTP
codex mcp add myserver --url https://mcp.example.com/mcp --bearer-token-env-var MY_TOKEN
# verify
codex mcp get myserver ; codex mcp list --json
```

Sources: local `codex mcp add --help`; [MCP] (`codex mcp add <server-name> --env K=V -- <cmd>`,
`codex mcp add context7 -- npx -y @upstash/context7-mcp`); [CMDS].

**Project scope** (no `--scope` flag — see D.3): create/append `<repo>/.codex/config.toml` with only
the server table, e.g.

```toml
[mcp_servers.myserver]
command = "npx"
args = ["-y", "some-mcp-server"]
startup_timeout_sec = 30   # optional
enabled = true             # optional; default true
```

Required: `command` (stdio) or `url` (HTTP). Everything else optional ([MCP] § STDIO/Streamable HTTP
servers). Caveat: the project must be trusted (`[projects.'<path>'] trust_level = "trusted"` in the
user config) or the whole `.codex/` layer is skipped ([BASIC], [REF]). Do **not** put `notify`,
`model_provider(s)`, `openai_base_url`, `otel`, or profile keys in a project config — they are ignored
with a warning ([ADV]).

### G.2 Agent TOML

Write `<repo>/.codex/agents/<name>.toml` or `%USERPROFILE%\.codex\agents\<name>.toml`. Required
fields: `name`, `description`, `developer_instructions` ([SUB]). Minimum viable:

```toml
name = "docs_researcher"
description = "Documentation specialist that verifies APIs through the docs MCP server."
model = "gpt-6-luna"                 # optional
model_reasoning_effort = "high"      # optional
developer_instructions = """
Use the docs MCP server to confirm APIs and options. Do not make code changes.
"""
```

Rules that matter for a generator: `name` is the identity (filename is cosmetic); a custom name that
collides with `default`/`worker`/`explorer` overrides the built-in; `mcp_servers`, `sandbox_mode`,
`approval_policy`, `model_provider` inside the file are ignored ([SUB]). Optional per-agent skill
gating uses `[[skills.config]] path = "<abs>/SKILL.md"` + `enabled = false` ([SUB] example, [SKILLS]).

### G.3 Skills

Create `%USERPROFILE%\.agents\skills\<skill-name>\SKILL.md` (global) or
`<repo>/.agents/skills/<skill-name>/SKILL.md` (project) with YAML frontmatter containing at least
`name` and `description`; optional `scripts/`, `references/`, `assets/`, `agents/openai.yaml`
(`interface.display_name`, `policy.allow_implicit_invocation`, `dependencies.tools[]` for MCP deps)
([SKILLS]).

### G.4 Prompts / instructions

Custom prompts are gone ([GH15941]) — the equivalents are: a **skill** for a reusable workflow, and
`AGENTS.md` for persistent instructions (global `~/.codex/AGENTS.md`, project `AGENTS.md` per dir,
`AGENTS.override.md` to shadow) ([AGENTS], [SKILLS]). `model_instructions_file` can replace built-in
instructions wholesale ([REF]).

### G.5 Validation checklist

1. Serialize TOML with a real encoder — never string-concatenate.
2. Re-read with `codex mcp list --json` / `codex mcp get` for MCP; run `codex --strict-config` (or
   `codex doctor`) to catch unknown keys ([CMDS], local `--help`).
3. Keep `[agents]`, `[mcp_servers]`, `[plugins]`, `[projects]` tables intact — merge into the existing
   document, do not replace the file.
4. For skills, validate frontmatter has non-empty `name`/`description` ([SKILLS]); duplicate names are
   allowed and both surface, so do not rely on name uniqueness ([SKILLS]).

---

## H. Version / stability notes (only where sources state them)

| Fact | Source |
|---|---|
| Installed CLI reports `codex-cli 0.164.0-alpha.11.1` | local `codex --version` |
| Multi-agent tools **stable, on by default**; `features.multi_agent` stable | [REF], [BASIC], [SUB] |
| `code_mode`, `features.network_proxy`, `rollout_budget`, `features.prevent_idle_sleep` = under development / off by default | [REF] |
| `features.personality` = removed flag, accepted and ignored; `personality = friendly/pragmatic` deprecated | [REF], [BASIC] |
| Custom prompts removed starting 0.117.0 (announced deprecation) | [GH15941] |
| `--profile` no longer reads `[profiles.<name>]` from 0.134.0 | [ADV] |
| Managed permission-profile allowlists require 0.138.0+; ≤0.137.0 ignore them | [REF] |
| WSL1 supported through 0.114; Linux sandbox moved to bubblewrap in 0.115 | [WSL], [WINAPP] |
| `approval_policy = "untrusted"` no longer supported; `on-failure` deprecated | [REF] |
| `experimental_instructions_file` renamed to `model_instructions_file` | [REF] |

**Implication:** treat this surface as fast-moving. Pin behaviour to the detected version
(`codex --version`) and avoid writing keys you have not verified against
`https://developers.openai.com/codex/config-schema.json` ([REF]).

---

## I. Windows-specific issues

1. **Codex home.** `%USERPROFILE%\.codex` for the Windows app and native CLI; `CODEX_HOME` overrides
   it and **must already exist** if set ([WINAPP], [ENV]). Never assume `~/.codex` expands — resolve
   `%USERPROFILE%` (or `CODEX_HOME`) explicitly.
2. **TOML backslash escaping.** Windows paths must be either a TOML *literal* string (single quotes,
   no escapes) or a basic string with doubled backslashes. Both forms appear in the live config:
   `command = 'C:\Users\raosu\…\node_repl.exe'` alongside
   `notify = [ "C:\\Users\\raosu\\…\\codex-computer-use.exe", "turn-ended" ]`
   ([local:…\.codex\config.toml]). A naive `"C:\Users\..."` basic string breaks on `\U`. Forward
   slashes also work in practice (`CODEX_CLI_PATH = 'C:\…'` vs `"C:/Users/…"` inside the same file).
3. **Case-insensitive, lowercased project keys.** Project trust entries on this machine are stored
   lowercased (`[projects.'c:\users\raosu\suryapaldev\projects\company-os']`) even though the real path
   is mixed-case — a lookup keyed on raw casing will miss ([local]). (Normalization behaviour is
   inferred; see Unverified.)
4. **WSL vs native split.** The Windows app uses `%USERPROFILE%\.codex`; the CLI inside WSL uses the
   Linux home, so **config, auth, and sessions are not shared by default**. Documented fix:
   `export CODEX_HOME=/mnt/c/Users/<windows-user>/.codex` in `~/.bashrc`/`~/.zshrc` ([WINAPP],
   [WSL]). WSL1 unsupported since 0.115 ([WSL]). Keep repos on the native Windows drive and access
   them from WSL via `/mnt/<drive>/...` for reliability ([WINAPP]).
5. **Windows sandbox knob lives in config.** `[windows] sandbox = "mxc" | "elevated" | "unelevated"`
   plus `features.prefer_mxc`; this machine has `[windows] sandbox = "elevated"` ([REF], [local]).
6. **Windows-only hook override.** `hooks.<Event>[].hooks[].commandWindows` (alias
   `command_windows`) supplies a Windows command for a hook ([REF]).
7. **No documented Windows admin skills dir.** Docs give `/etc/codex/skills` for ADMIN scope only
   ([SKILLS]); no Windows equivalent is documented (see Unverified).
8. **PowerShell/cmd quoting.** Real MCP entries wrap via `cmd.exe /d /c npx …` ([local]) — a Windows
   app should prefer `command` + `args` arrays over shell strings, and quote `-c key="value"` TOML
   fragments carefully ([ADV]).

---

## Unverified

1. **`[agents.<name>]` inline tables vs standalone `agents/*.toml`.** Both are documented ([REF] vs
   [SUB]); precedence when both define the same `name`, and whether `config_file` can point at a
   standalone agent file, are not stated.
2. **Duplicate agent `name` across global and project scopes** — no documented winner (skills
   explicitly do *not* dedupe; agents are silent on it).
3. **Windows admin/system paths.** No documented Windows equivalent of `/etc/codex/config.toml`,
   `/etc/codex/skills`, or the system `requirements.toml` path; MDM/registry delivery is described
   only at a high level ([REF], [BASIC]).
4. **Live hot-reload of `config.toml`, `agents/*.toml`, and `[mcp_servers]`** — docs say "restart"; no
   watcher is documented. Not tested by mutating files on this machine (would modify `C:\Users\raosu`).
5. **Path normalization for `[projects.*]` keys** (lowercasing, `\\?\` prefixes) — observed as
   lowercase on disk, not documented.
6. **Exact `[mcp_servers.<id>]` write order / formatting produced by `codex mcp add`** (whether it
   rewrites or appends, and whether it preserves comments) — not tested to avoid mutating the user's
   config; the observed no-comments/blank-line state suggests a full rewrite.
7. **Whether `codex mcp remove` also deletes OAuth credentials** stored for that server.
8. **Whether a per-agent `enabled = false` key is accepted** (not in [SUB] or [REF]; assumed
   unsupported).
9. **`.codex/skills` vs `.agents/skills` for user scope** — one GitHub commenter claims Codex also
   reads `.codex/skills` ([GH15941] comment), which conflicts with the official table showing
   `$HOME/.agents/skills` ([SKILLS]). Official docs were treated as authoritative.
10. **`requirements.toml` exact filename/locations** for admin policy — referenced but path not
    captured here ([REF] links to the managed-configuration page, not fetched).

---

## Sources

| Tag | URL / path |
|---|---|
| [REF] | https://developers.openai.com/codex/config-file/config-reference (→ https://learn.chatgpt.com/docs/config-file/config-reference) — incl. `config-schema.json` link |
| [BASIC] | https://developers.openai.com/codex/config-file/config-basic |
| [ADV] | https://developers.openai.com/codex/config-file/config-advanced |
| [ENV] | https://developers.openai.com/codex/config-file/environment-variables |
| [SUB] | https://developers.openai.com/codex/agent-configuration/subagents |
| [AGENTS] | https://developers.openai.com/codex/agent-configuration/agents-md |
| [SKILLS] | https://developers.openai.com/codex/build-skills |
| [MCP] | https://developers.openai.com/codex/extend/mcp |
| [CUSTOM] | https://developers.openai.com/codex/customization/overview |
| [CMDS] | https://developers.openai.com/codex/developer-commands |
| [WINAPP] | https://developers.openai.com/codex/windows/windows-app |
| [WSL] | https://developers.openai.com/codex/windows/wsl |
| [GH15941] | https://github.com/openai/codex/issues/15941 — OpenAI maintainer: "custom prompts … removed completely starting in 0.117.0. You should convert your custom prompts to skills." |
| [GH config.md] | https://raw.githubusercontent.com/openai/codex/main/docs/config.md — repo stub that now delegates to the developers.openai.com pages above |
| local | `C:\Users\raosu\.codex\config.toml`, `config.toml.rea.backup`, `AGENTS.md`, `agents\code-reviewer.toml`, `agents\red-team.toml`, `skills\.system\*\SKILL.md`, directory listing of `C:\Users\raosu\.codex` |
| local CLI | `C:\Users\raosu\AppData\Local\OpenAI\Codex\bin\2993767bd1dfb009\codex.exe` → `--version`, `--help`, `mcp --help`, `mcp add --help`, `mcp list --help`, `mcp get --help`, `mcp remove --help`, `plugin --help`, `mcp list` |

Docs note: `developers.openai.com/codex/*` 302-redirects to `learn.chatgpt.com/docs/*`; appending
`.md` to any docs URL returns the markdown source ([REF] page banner).
