# Command Code (`command-code` / `cmdc`) — on-disk config surface

Research target: a Windows desktop app that must **list / enable-disable / remove** (V1) and
**create** (V2) subagents, skills and MCP servers for Command Code, at project and global scope.

Installed build on this machine: **`command-code@1.88.0`**
(`C:\nvm4w\nodejs\node_modules\command-code\package.json`), bin entries
`cmd`, `cmdc`, `command-code`, `commandcode` → `dist/index.mjs` (1.8 KB loader) → `dist/cli.mjs`
(3.4 MB minified bundle).

Everything below is either **observed on this machine** (absolute path cited), **read out of the
shipped bundle** (`dist/cli.mjs`, cited by symbol name), or **read out of the docs bundle that
ships inside the package** (`dist/bundled/command-code-knowledge/reference/*.md`, which is
generated from `https://commandcode.ai/docs`) and cross-checked against the live docs site.

---

## Path inventory

`~` = `C:\Users\raosu`. `<root>` = project root (git root → else nearest dir containing
`.commandcode` → else cwd; `dist/cli.mjs:resolveProjectDir`).

| Artifact | Project | Global (Windows) | macOS / Linux | Format | Source |
|---|---|---|---|---|---|
| User dir | — | `~\.commandcode\` | `~/.commandcode/` | dir | `dist/cli.mjs` const `Kl=".commandcode"`; `C:\Users\raosu\.commandcode\` |
| Project dir | `<root>\.commandcode\` | — | `<root>/.commandcode/` | dir | observed `…\isolated-environment\.commandcode\`; `resolveProjectDir` |
| Settings — user | — | `~\.commandcode\settings.json` | `~/.commandcode/settings.json` | JSON | `dist/cli.mjs:settingsPath` (`"user"`); docs `skills.md` §Enable or disable |
| Settings — project shared | `<root>\.commandcode\settings.json` | — | same | JSON | `settingsPath` (`"projectShared"`); observed `…\.commandcode\settings.json` |
| Settings — project local (locked) | `<root>\.commandcode\settings.local.json` | — | same | JSON | `settingsPath` (`"projectLocal"`); `dist/cli.mjs:getProjectLocalSettingsPath` |
| App state / user config | — | `~\.commandcode\config.json` | `~/.commandcode/config.json` | JSON | `dist/cli.mjs:getConfigFile`; observed keys `installed, provider, model, firstMessageSent`; docs `custom-slash-commands.md` §`/statusline` ("saved in `~/.commandcode/config.json`") |
| Auth | — | `~\.commandcode\auth.json` | `~/.commandcode/auth.json` | JSON | `dist/cli.mjs:writeCommandAuthFileObject` (keys `apiKey,userId,userName,keyName,authenticatedAt`); observed (values redacted) |
| MCP — user | — | `~\.commandcode\mcp.json` | `~/.commandcode/mcp.json` | JSON `{"mcpServers":{…}}` | `dist/cli.mjs:getUserMcpConfigPath`; observed |
| MCP — user legacy | — | `~\.commandcode\.mcp.json` | `~/.commandcode/.mcp.json` | JSON | `dist/cli.mjs:loadMcpServers` (read second, overrides above) |
| MCP — project | `<root>\.mcp.json` | — | `<root>/.mcp.json` | JSON | `dist/cli.mjs:getProjectMcpConfigPath`; docs `mcp.md` §Configuration & scopes |
| MCP — local (per-project, private) | — | `~\.commandcode\projects\<slug>\mcp.json` | `~/.commandcode/projects/<slug>/mcp.json` | JSON | `dist/cli.mjs:localMcpConfigPath` + `slugifyProjectPath`; docs `mcp.md` |
| MCP OAuth tokens | — | `~\.commandcode\mcp-tokens.json` | `~/.commandcode/mcp-tokens.json` | JSON `{version,tokens}` | `dist/cli.mjs:vaultPath`; docs `mcp.md` §Authentication |
| Skills — project | `<root>\.commandcode\skills\<name>\SKILL.md` | — | same | dir + MD | `dist/cli.mjs` `fl=".commandcode/skills"`, `getProjectSkillsDir`; docs `skills.md` |
| Skills — project (`.agents` compat) | `<root>\.agents\skills\<name>\SKILL.md` | — | same | dir + MD | `dist/cli.mjs` `yl=".agents/skills"`, `getProjectAgentsCompatSkillsDir` |
| Skills — global | — | `~\.commandcode\skills\<name>\SKILL.md` | `~/.commandcode/skills/` | dir + MD | `dist/cli.mjs:getGlobalSkillsDir`; observed (44 junctions) |
| Skills — global (`.agents` compat) | — | `~\.agents\skills\<name>\SKILL.md` | `~/.agents/skills/` | dir + MD | `dist/cli.mjs:getGlobalAgentsCompatSkillsDir`; observed (62 skills) |
| Skills — bundled | — | `C:\nvm4w\nodejs\node_modules\command-code\dist\bundled\<skill>\SKILL.md` | same relative path | dir + MD | `dist/cli.mjs:bundledSkillsDir()` = `dirname(import.meta.url)/bundled`; observed 7 dirs |
| Agents (subagents) — project | `<root>\.commandcode\agents\<name>.md` | — | same | MD + YAML FM | `dist/cli.mjs:projectAgentDirs`, `agentDirForLocation`; docs `custom-agents.md` |
| Agents — global | — | `~\.commandcode\agents\<name>.md` | `~/.commandcode/agents/` | MD + YAML FM | `dist/cli.mjs:userAgentDir`; docs `custom-agents.md` |
| Custom slash commands | `<root>\.commandcode\commands\**.md` | `~\.commandcode\commands\` | same | MD (name = filename) | `dist/cli.mjs:loadCustomCommands` (`Kl`+`zl="commands"`); docs `custom-slash-commands.md` L331‑341 |
| Memory | `<root>\AGENTS.md` else `<root>\.commandcode\AGENTS.md` | `~\.commandcode\AGENTS.md` | same | MD | docs `memory.md` §Where memory files live |
| Taste (learned profile) | `<root>\.commandcode\taste\` (`taste.md` + `<category>/taste.md`) | `~\.commandcode\taste\` (home-rooted variant) | same | MD | `dist/cli.mjs:tasteDirForRoot`, `getRootTasteFile`, `getCategoryTasteFile`; observed `…\.commandcode\taste\taste.md` (0 B) |
| Plans | — (project has none) | `~\.commandcode\plans\*.md` | `~/.commandcode/plans/` | MD | `dist/cli.mjs:resolvePlanContent` (`${HOME}/.commandcode/plans`); plan-mode error text; **dir absent on this machine** |
| Mods | `<root>\.commandcode\mods\*.ts` | `~\.commandcode\mods\` | same | TS (jiti) | `dist/bundled/mod-builder/SKILL.md` §1; docs `custom-slash-commands.md` L233 |
| Sessions / checkpoints | — | `~\.commandcode\projects\<slug>\<sessionId>.jsonl`, `.checkpoints.jsonl`, `.meta.json` | same | JSONL/JSON | observed |
| Prompt history | — | `~\.commandcode\history.jsonl` | same | JSONL | `dist/cli.mjs:defaultHistoryPath`; observed |
| Model catalog cache | — | `~\.commandcode\cache\model-catalog\v1\*.json` | same | JSON | `dist/cli.mjs:resolveCacheDir`; observed |
| Edit history (undo) | — | `~\.commandcode\file-history\<session>\<hash>` | same | blobs | observed |
| Telemetry id | — | `~\.commandcode\telemetry-install-id` | same | UUID text | `dist/cli.mjs` const `xD="telemetry-install-id"`; observed |
| Update state | — | `~\.commandcode\updates.json` | same | JSON `{autoUpdate,lastCheckedAt,pending}` | observed |
| **Not read by Command Code** | — | `~\.agents\mcp.json`, `~\.agents\.skill-lock.json` | `~/.agents/…` | JSON | see §B/§D — zero references in `dist/cli.mjs` |

Side note on `~\.agents\`: it holds `skills\` (62 skills), `mcp.json`, `.skill-lock.json`
(24 KB). `.skill-lock.json` is **not** a Command Code file (`grep` for `skillFolderHash`,
`.skill-lock` in `dist/cli.mjs` → 0 hits); its shape `{version:3, skills:{<name>:{source,
sourceType,sourceUrl,skillPath,skillFolderHash,installedAt,updatedAt}}, dismissed,
lastSelectedAgents:[… "command-code" …]}` and `lastSelectedAgents` naming 16 other agents shows it
is written by a *multi-agent* skills installer that targets `~/.agents/skills` and links each skill
into per-agent dirs. On this machine every entry in `~\.commandcode\skills\` is a **junction** to
`C:\Users\raosu\.agents\skills\<name>` (44 junctions; `Get-ChildItem` `LinkType=Junction`), which is
exactly that installer's doing — not Command Code's.

---

## A. Config roots, layers, precedence

| Layer | Path | Read by |
|---|---|---|
| User settings | `~/.commandcode/settings.json` | `dist/cli.mjs:getSettingsUserPath` |
| Project shared | `<root>/.commandcode/settings.json` | `getSettingsProjectSharedPath` |
| Project local (overrides, "locked") | `<root>/.commandcode/settings.local.json` | `getSettingsProjectLocalPath`; `cmd config set` refuses keys locked here (`"… is locked by settings.local.json — edit that file to change it."`) |
| User config/state | `~/.commandcode/config.json` | `getConfigFile` |
| Memory | `~/.commandcode/AGENTS.md` → `<root>/AGENTS.md` → `<root>/.commandcode/AGENTS.md` | docs `memory.md` |

- **Root resolution:** home = `HOME ?? USERPROFILE` (`dist/cli.mjs:homeDir2/homeDir10/homeDir11`).
  Project root = git root, else walk up ≤ 10 levels for `.commandcode`
  (`walkUpToFindDir` with `_a=10`), else cwd (`resolveProjectDir`).
- **`config.json` shape** (observed `C:\Users\raosu\.commandcode\config.json`, 124 B):
  `{"installed":true,"provider":"command-code","model":"deepseek/deepseek-v4.1-flash","firstMessageSent":true}`.
  Same file also carries `statusline` (docs `custom-slash-commands.md` L270‑273),
  `ideContextEnabled`, `artifactsEnabled`, `imageVisionEnabled` (`dist/cli.mjs` getters/setters),
  and `localOnly` (`dist/cli.mjs` error text: `"localOnly": true in ~/.commandcode/config.json`).
- **Env overrides found in the bundle:** `HOME`/`USERPROFILE` (all paths), `COMMANDCODE_API_URL`,
  `COMMANDCODE_API_ENV` (`local|staging|prod`), `COMMANDCODE_SANDBOX`, `CMD_LOCAL_ONLY`,
  `MCP_TOOL_TIMEOUT` (`resolveMcpTimeoutMs`), `COMMAND_CODE_API_KEY` (docs `reference/cli`),
  `EDITOR` (docs `skills.md`). **No env var overrides the config directory** (searched
  `COMMANDCODE_`, `CMD_`, `XDG*` — nothing remaps `~/.commandcode`).
- **Name collisions:** settings layers overwrite whole keys (`cmd config` in docs `config/references/settings.md`);
  skills resolve project → `.agents` project → user → `.agents` user → extras → bundled (`createSkillRegistry`);
  MCP resolves user → project → local (`loadMcpServers`, later `set` wins).

## B. Subagents (custom agents)

- **First-class.** Three immutable built-ins (**General**, **Explore**, **Plan**), any number of
  user files. `/agents` is the manager UI; docs: <https://commandcode.ai/docs/agents>.
- **Locations:** `~/.commandcode/agents/<name>.md` (personal) and `<root>/.commandcode/agents/<name>.md`
  (project) — `dist/cli.mjs:userAgentDir` / `projectAgentDirs`. Project scan covers **both**
  `${cwd}/.commandcode/agents` and `<gitRoot>/.commandcode/agents` (cwd first, first name wins).
  **`~/.agents/agents` is never read** (`grep '\.agents/agents'` → 0; `Get-ChildItem ~\.agents` has
  no `agents` dir).
- **Format:** Markdown, YAML frontmatter + system prompt body. Observed dist parser fields:
  `name, description, tools, disallowedTools, model, reasoningEffort, maxTurns, permissionMode,
  background, showOutput` (`scanAgentsDir` → `parseToolAccess/parseDenyList/parseReasoningEffort/
  parseMaxTurns/parsePermissionMode/parseBooleanFlag`); docs `custom-agents.md` §Frontmatter fields
  (only these keys are read, `name` required, others optional).
- **Reserved names:** `["explore","plan","review","general"]` (`dist/cli.mjs` `BC=[…]`,
  `isReservedAgentName`) — a custom file with a reserved name is ignored.
- **Precedence:** bundled → personal → project, **first definition of a name wins** (docs
  `custom-agents.md` §Where agents load from), i.e. *personal shadows project* — the opposite of
  skills. `discoverAgents` returns `personal` and `project` lists separately and the registry
  resolves user-first, consistent with the docs.
- **Disable/remove:** no `disabledAgents` key exists anywhere in the bundle (0 hits). Disabling =
  edit `tools:`/delete/rename the file; removing = delete the `.md` (`deleteAgentDefinition`;
  bundled agents throw `"Cannot delete bundled agent"`). Files are re-scanned **each turn**
  (`createAgentRegistryRefreshMod` → `onTurnStart → registry.refresh()`), so no restart.

## C. Skills

- **Discovery roots** (`dist/cli.mjs:createSkillRegistry`, in this exact push order):
  1. `<cwd>/.commandcode/skills` (project)
  2. `<cwd>/.agents/skills` (project)
  3. `<gitRoot>/.commandcode/skills`, `<gitRoot>/.agents/skills` (when git root ≠ cwd)
  4. no git root: nearest-ancestor `.commandcode/skills` and `.agents/skills` (walk-up ≤ 10, stops at home)
  5. `~/.commandcode/skills` (user), `~/.agents/skills` (user)
  6. `extraDirs` (`--skill <path>`, settings `skills:[]`)
  7. bundled dir = `<package>/dist/bundled` (`bundledSkillsDir()`); the 7 shipped skills are
     `agent-browser`, `command-code-knowledge`, `config`, `design`, `loop`, `mod-builder`,
     `skill-builder` (verified on disk; `CHANGELOG` 1.0.0 added the last three).
  Project roots are scanned **only when the workspace is trusted** (`r = !e.isTrusted || e.isTrusted()`
  gate in `createSkillRegistry`).
- **Recursive scan:** nested grouping dirs allowed; each skill = the dir that directly contains
  `SKILL.md`; scan depth + result count are capped (`findSkillMdFiles` `maxDepth`, `foundCount`).
- **Frontmatter, confirmed from the shipped parser** (`createCommandSkillFrontmatterParser` return
  object): `name`, `description`, `license`, `compatibility`, `allowed-tools`, `metadata`, `model`,
  `argument-hint`, `disable-model-invocation`, `when_to_use`, `user-invocable`, `disallowed-tools`,
  `arguments`, `effort`. Docs add constraints (`name` ≤ 64, `[a-z0-9-]`, must equal dir name;
  `description` required, ≤ 1024). Real installed files confirm the minimal shape:
  `~\.agents\skills\caveman\SKILL.md` (`name:`, `description: >`) and
  `~\.agents\skills\code-review\SKILL.md` (`name:`, quoted one-line `description:`). Bundled
  `dist\bundled\config\SKILL.md` is the same two-key shape.
- **Collisions:** `deduplicateSkills({project,global})` — first occurrence wins, order = the list
  above, so **project beats global and `.commandcode` beats `.agents`**; shadowed copies are
  reported as "Duplicate names" in `/skills` + `cmd skills list --debug`. A skill colliding with a
  built-in/custom command is reachable only as `/skill:<name>`.
- **`~/.agents/.skill-lock.json`:** **not** consumed by Command Code (0 hits for `.skill-lock`,
  `skillFolderHash`). It belongs to the external multi-agent installer that populates `~/.agents/skills`.
- **Enable/disable:** `disabledSkills: ["name", …]` in `settings.json`. `createDisabledSkillsStore`
  unions the key across **user**, **projectShared** and **projectLocal** settings files, and
  `filterDisabledSkills` drops the name from *both* the project and global lists — disabling a name
  disables every copy of it. Toggle writes via `setEnabled` keep the array sorted and delete the key
  when empty (`disabledSkillsIn` / `readSettingsForWrite`). Equivalent UI: `/skills` → Enter.
- **Remove:** `cmd skills remove <name> [--global|-g] [--yes|-y]` → `removeSkill`, which resolves
  **only** `~/.commandcode/skills` or `<project>/.commandcode/skills` (`getGlobalSkillsDir` /
  `getProjectSkillsDir`) and `fs.rm` the dir. It never touches `~/.agents/skills` — so on this
  machine removing a junctioned skill leaves the `.agents` original in place, and it reappears with
  the `[.agents]` badge. Install mirror: `cmd skills add <owner/repo> [-g] [-f] [-s <name>]`.

## D. MCP

- **Files that hold MCP config** (runtime loader, `dist/cli.mjs:loadMcpServers`, applied in this
  order; later wins):

  | # | Path | Scope label |
  |---|---|---|
  | 1 | `~/.commandcode/settings.json` → `mcp.servers: [ … ]` (array, only when `includeProjectServers === false`) | user settings |
  | 2 | `~/.commandcode/mcp.json` | user |
  | 3 | `~/.commandcode/.mcp.json` (legacy) | user |
  | 4 | `<cwd>/.mcp.json` | project |
  | 5 | `~/.commandcode/projects/<slug>/mcp.json` | local |

  So `local > project > user`, matching docs `mcp.md` §Precedence. **`<root>/.commandcode/mcp.json`
  and `~/.agents/mcp.json` are not read** (0 hits for `agents/mcp.json`; the loader's only
  `.commandcode/mcp.json` reference is home-rooted). On this machine `~\.agents\mcp.json` and
  `~\.commandcode\mcp.json` both define `rea`, but only the latter is live — and it is the only one
  carrying `enabled:false`, i.e. someone disabled `rea` through Command Code, which wrote the flag
  into `~\.commandcode\mcp.json` only.
- **`cmd mcp` CLI config paths** are separate and enumerated by
  `getAllMcpConfigPaths` → exactly `user`, `project`, `local` (`getMcpConfigPath`); `saveMcpConfig`
  writes the file for that scope with `mode 0600`.
- **JSON shape** (docs `mcp.md` §Config schema + `toServerConfig`):
  ```json
  {"mcpServers": {"<name>": {
    "transport": "stdio|http|sse",   // "type" accepted as alias (inferTransport)
    "enabled": true,                  // optional; anything but false → enabled
    "command": "npx", "args": ["-y","pkg"],
    "url": "https://…", "headers": {"Authorization":"Bearer ${API_KEY}"},
    "env": {"API_KEY": "…"},
    "oauth": {"authorizationUrl":"…","tokenUrl":"…","clientId":"…","scopes":["…"],"resource":"…","clientSecret":"…"}
  }}}
  ```
  Validation (`parseStoredMcpServer`): `enabled` must be boolean, `headers`/`env` must be
  `Record<string,string>`, http/sse requires `url`, stdio requires `command`; `${VAR}` /
  `${VAR:-default}` resolve at runtime in `headers`/`env`, `$${VAR}` escapes.
- **Disable vs delete:** `enabled:false` is honoured at connection time —
  `mcpClientsFromConfig` does `if (!1 !== o.enabled)` before constructing a client, so a disabled
  server is never spawned; `/mcp` shows status `disabled`, offers **Enable**, and
  `setMcpServerEnabled` rewrites `enabled` in the *same scope file* the server came from.
  Delete = `cmd mcp remove <name>` (rewrites that scope's file). Other subcommands: `add`,
  `add-json`, `list`, `get`, `auth [--status|--list|--clear]` (docs `mcp.md` §Command reference;
  `dist/cli.mjs` carries the `--scope`/`--transport`/`--env`/`--header` strings and the OAuth DCR flow).
- **Tool naming:** `mcp__<server>__<tool>` (docs `mcp.md`; also cited in `custom-agents.md` §Tools).
- **Safe-write behaviour:** `readRawMcpConfigForWrite` refuses to touch a file that is not valid JSON
  or lacks the `{"mcpServers":{…}}` shape ("… so it was left untouched. Fix or remove the file, then
  retry.") — a V1 editor should preserve unknown top-level keys and rewrite with 2-space indent.

## E. Other artifacts

| Artifact | Detail | Source |
|---|---|---|
| Taste | `<root>/.commandcode/taste/taste.md` (+ `<category>/taste.md`); project dir auto-created with an empty `taste.md` by `initTasteDir`; global variant is home-rooted (`tasteDirForRoot({projectRoot: home})`, target `local-global`). Toggle via `taste-learning` (user) / `taste-learning-project` (project) settings, `cmd taste`/`npx taste`. | `dist/cli.mjs`; observed `…\.commandcode\taste\taste.md` (0 B); docs `config/references/settings.md` |
| Plans | `~/.commandcode/plans/*.md`; plan mode may only write there (`isPlanPath` gate); `/plans` browses, `/plan-review` opens latest | `dist/cli.mjs:resolvePlanContent`, `isPlanPath`; dir absent on this machine |
| Commands | `<root>/.commandcode/commands/**/<file>.md` (project) and `~/.commandcode/commands/` (user); name = filename, subdirectory becomes a label, body = prompt, `$1`/`${@:2}` args | `loadCustomCommands`; docs `custom-slash-commands.md` L331‑341, dispatch order L398‑406 (built-ins → custom → skills) |
| Memory | `AGENTS.md` only, three tiers (user, project, subdirectory), `@path` imports ≤ 5 deep; **`CLAUDE.md` is not read** — `/import` copies it and rewrites the name | docs `memory.md`; `rewriteClaudeTerms` (`CLAUDE.md` → `AGENTS.md`) in `dist/cli.mjs` |
| Modes | permission modes `default|accept-edits|plan|dont-ask|yolo`; `/mode`, `--permission-mode`, `permissions.defaultMode` setting; `settings.local.json` can lock a setting | docs `config/references/settings.md`; `dist/cli.mjs` mode validation |
| Mods | `~/.commandcode/mods/*.ts` (personal) / `<root>/.commandcode/mods/*.ts` (project, loads once workspace trusted); `cmd mods add|list|remove|update|enable|disable`, `--mod`, `--safe-mode`, `/mods reload` | `dist/bundled/mod-builder/SKILL.md`; docs `reference/cli` |
| Sessions | `~/.commandcode/projects/<slug>/<sessionId>.jsonl` + `.checkpoints.jsonl` + `.meta.json` | observed |
| History / cache / telemetry / updates / file-history | `history.jsonl`, `cache/model-catalog/v1/*.json`, `telemetry-install-id`, `updates.json`, `file-history/<session>/<hash>` | `dist/cli.mjs`; observed |

## F. Hot-reload vs restart, safe disable, hand-editing risk

| Change | Takes effect | Source |
|---|---|---|
| Agent file add/edit/delete | **next turn** (registry refresh in `onTurnStart`) | `createAgentRegistryRefreshMod`; docs `custom-agents.md` |
| Skill add/edit/delete | next turn — no restart ("After editing, the changes are immediately applied… No restart is required") | docs `skills.md` §Manual editing; catalog is rebuilt per turn |
| `disabledSkills` write | next session is guaranteed; running session re-reads on redraw of `/skills` | docs `skills.md` §Where the setting lives |
| `AGENTS.md` edit | next request (system prompt rebuilt each request) | docs `memory.md` §How memory reaches the model |
| Mods | **process lifetime** — `/reload` restarts and re-imports; `cmd --mod` for one session | `mod-builder/SKILL.md` §5 |
| MCP server add/enable/disable/remove on disk | **new session** — clients are built once per session from `loadMcpServers` → `mcpClientsFromConfig`; `/mcp` toggles affect the live session through the manager | `dist/cli.mjs` (loader called in session setup), docs `mcp.md` |
| Staged CLI update | `/reload` | docs `custom-slash-commands.md` |

Safe enable/disable matrix (what a desktop app should write):

| Thing | Disable | Enable | Remove | Never |
|---|---|---|---|---|
| Skill | add name to `disabledSkills` in `settings.json` (user for global, project for project) | remove name from that array | `cmd skills remove` → deletes `<scope>/.commandcode/skills/<name>` | don't delete a skill you only mean to hide; don't remove via `~/.agents` (invisible to `cmd skills`) |
| Agent | no flag exists — rename/delete file, or narrow `tools:` | restore file | delete `<scope>/.commandcode/agents/<name>.md` | don't hand-write reserved names (`explore/plan/review/general`) |
| MCP | `"enabled": false` **in the same file the server is defined in** | `"enabled": true` or delete the key | delete the entry from that file (`cmd mcp remove <name>`) | don't write `~/.agents/mcp.json` or `<root>/.commandcode/mcp.json` (not read); don't reorder keys/indent destructively (loader tolerates, but `cmd mcp` rewrites the whole file with 2-space indent) |

Hand-editing risks, concretely:
- `settings.json` with invalid JSON is silently treated as `{}` by `readSettings` (catch → `{}`),
  i.e. every user setting is ignored until fixed — while `cmd config set` refuses to write it.
- `settings.local.json` **locks** settings; a stray key there makes `cmd config set` fail for that key.
- `mcp.json` with bad JSON/shape makes `cmd mcp add/enable` refuse ("left untouched").
- Invalid frontmatter: a skill is skipped with a warning (visible in `/skills`, `cmd skills list --debug`);
  a bad agent frontmatter surfaces a warning on the feed (`CHANGELOG` 1.38.2).
- Deleting `.commandcode/skills/<name>` when it is a junction to `~/.agents/skills/<name>` leaves the
  `.agents` original live.

## G. V2 creation — canonical layouts

**Skill** (validated by `validateSkillMetadata`; must match dir name; `description` required):

```
<scope-root>\skills\<name>\SKILL.md     # required
                          scripts\      # optional
                          references\   # optional
                          assets\       # optional
```
`<scope-root>` = `<root>\.commandcode` (project, trusted workspace) or `~\.commandcode` (global).
Frontmatter minimum (real files in this repo's environment use exactly this):
`---\nname: <name>\ndescription: <one line, with trigger phrases>\n---` then the instruction body.
**Do not copy into both `~\.commandcode\skills` and `~\.agents\skills`** — Command Code reads both
and `.commandcode` wins, so a duplicate just creates a shadowed copy and two things to keep in sync
(the junctions on this machine are an artefact of an external installer, not a requirement).
Validation: `cmd skills list --debug` (per-row path + reason), or `/skills` issues view. Creating via
CLI: `cmd skills add <owner/repo>`; authoring via the shipped `skill-builder` skill / `/skill-builder`.

**Agent:**

```
<scope-root>\agents\<name>.md
---
name: <name>                 # required, [a-zA-Z0-9_-], not explore/plan/review/general
description: <when to delegate>
tools: read_file, grep, glob # or "*" ; omitted = no tools
disallowedTools: shell_command
model: <any /model id>       # omit = inherit
reasoningEffort: high
maxTurns: 40
permissionMode: plan
background: false
showOutput: false
---
<system prompt>
```
Second path that must be honoured for project scope: `<gitRoot>\.commandcode\agents` when cwd is a
subdirectory (`projectAgentDirs`). Validation: file loads next turn; invalid frontmatter warnings
appear on the feed; `deleteAgentDefinition` refuses `__builtin__:` paths.

**MCP server** — write the *scope file*, keeping `{"mcpServers": {…}}` and merging by name:

| Intent | File | CLI equivalent |
|---|---|---|
| global | `~\.commandcode\mcp.json` | `cmdc mcp add --scope user …` |
| project (committed) | `<root>\.mcp.json` | `cmdc mcp add --scope project …` |
| project (private, default) | `~\.commandcode\projects\<slug>\mcp.json` | `cmdc mcp add …` / `--scope local` |

Prefer the CLI for writes (`cmdc mcp add|add-json|remove`, `cmdc mcp add-json <name> '{…}'`), because
it validates, normalizes `type`→`transport`, defaults `enabled`, strips OAuth secrets into
`mcp-tokens.json`, and writes `0600`. If writing JSON directly, mirror `toServerConfig`'s accepted
keys and `readRawMcpConfigForWrite`'s "leave untouched on bad shape" contract, and rewrite atomically.

## H. Version / stability

- Installed: **1.88.0** (`package.json`), newest CHANGELOG heading `## 1.88.0`
  (`C:\nvm4w\nodejs\node_modules\command-code\CHANGELOG.md`).
- Feature timeline relevant here: `.agents/skills` support landed in **0.18.9** (`CHANGELOG` L2392);
  the **1.0.0** rewrite (L1137‑1184) introduced `/agents` overhaul + create-by-asking, per-agent
  models, `/import`, bundled `command-code-knowledge` / `mod-builder` / `skill-builder`, and the
  "complete overhaul of agent skills support"; frontmatter warnings for agents on the feed at
  **1.38.2** (L667). Mods are still labelled beta in 1.0.0.
- Shipped schema is **authoritative in the bundle**, not just the docs: the package ships
  `dist/bundled/command-code-knowledge/reference/*.md` (13 doc pages: `skills.md`, `mcp.md`,
  `custom-agents.md`, `custom-slash-commands.md`, `memory.md`, `hooks.md`, `permissions.md`,
  `models.md`, `plan-mode.md`, `plans.md`, `sessions.md`, `tools.md`, `product-help.md`, `byok.md`)
  plus `dist/bundled/config/references/settings.md`, which says the *executable registry* is
  authoritative — verify with `cmd config list --scope effective --json`.
- Stability for a desktop app: file formats have been stable since 1.0.0; the only doc/code
  divergence found is `.commandcode` vs `.agents` MCP handling (see §D) and the fact that agent
  precedence (personal > project) is the reverse of skill precedence (project > personal).
- Auto-update is on by default (`updates.json {"autoUpdate": true}`, `--no-auto-update`); a V2 app
  must tolerate `dist/bundled` changing under it between releases.

## I. Windows specifics

- **npm global prefix** = `C:\nvm4w\nodejs` (`npm config get prefix`). Package at
  `C:\nvm4w\nodejs\node_modules\command-code`.
- **Shims** in that prefix: `cmd`, `cmd.cmd`, `cmd.ps1`, `cmdc`, `cmdc.cmd`, `cmdc.ps1`
  (each 341–873 B). `cmdc.ps1` resolves `$basedir`, prefers `$basedir\node.exe` over `node`, and
  runs `node_modules/command-code/dist/index.mjs` with `$args`, forwarding `$LASTEXITCODE`;
  `cmdc.cmd` does the same via `%dp0%\node.exe`. All four bin names map to the same file
  (`package.json` `bin`).
- **`cmd` collides with the Windows shell** — docs `reference/cli` states: "On Windows the alias is
  `cmdc`… Read `cmd` as `cmdc` throughout … or use the full `command-code`". A desktop app must
  shell out to `cmdc`, `command-code`, or `node <prefix>\node_modules\command-code\dist\index.mjs`
  — never `cmd`.
- **Bundled resources** resolve relative to the module, not the cwd:
  `bundledSkillsDir() = dirname(fileURLToPath(import.meta.url)) + "/bundled"` →
  `C:\nvm4w\nodejs\node_modules\command-code\dist\bundled`. Only `dist`, `vsix`, `CHANGELOG.md` are
  published (`package.json.files`), so `dist/bundled` is always present in a fresh install; a
  read-only or relocated install (e.g. an app-private copy) still works as long as the file layout
  is preserved. Bundled skills cannot be deleted (`deleteAgentDefinition` guard; `/agents` marks
  defaults read-only).
- **File locking / atomicity:** config and auth writes are `write(temp) → rename(temp, dest)` with
  0600 (`tempPathFor` + `fs.rename`; `writeCommandAuthFileObject` `mode:384`), which is the
  Windows-safe pattern. `EPERM` is treated as "process alive"/"permission denied" in liveness and
  error classification, and `EBUSY` appears once — there is **no retry loop** for a locked config
  file, so concurrent writers can surface a raw rename failure. Practical rule for the desktop app:
  write temp + `File.Move(replace)`, back off on `EPERM`/`EBUSY`, and never hold a handle on
  `~/.commandcode\*.json` while a `cmdc` session is starting.
- Path joins in the bundle are `/`-separated template strings over `HOME||USERPROFILE`
  (`~/.commandcode/...`); Windows accepts them, so both separators work, and the app should emit
  `\` for display but accept either.

---

## Unverified

1. **`.agents/mcp.json` authorship.** Not read by Command Code (verified) but its writer is not
   identified here — assumed to be the same external multi-agent installer as `.skill-lock.json`.
2. **Exact write race semantics on Windows** for `settings.json` when a running session and the
   desktop app write simultaneously — no lock file was found for settings (a lock exists only for
   the cron scheduler lease).
3. **Whether a disabled skill that is *shadowed* by a custom command still lists as disabled** —
   `filterDisabledSkills` runs before shadowing resolution; ordering of the two in `/skills` output
   was not exercised.
4. **`~/.commandcode/taste/` global layout on this machine** (no global taste dir exists; inferred
   from `tasteDirForRoot({projectRoot: home})` and the `local-global` target).
5. **`--skill` / settings `skills[]` precedence against `.commandcode/skills`** — code pushes extras
   after user roots (so they lose), docs list extras at step 5; identical conclusion but the doc
   text is looser about intra-extra ordering.
6. **Docs page for taste/settings** (`https://commandcode.ai/docs/taste`, `…/docs/settings`) was
   cited from in-package links but not fetched; only `/docs`, `/docs/skills`,
   `/docs/agents`, `/docs/mcp`, `/docs/reference/cli` were fetched live.
7. **`vsix/` directory** in the package (VS Code extension payload) was not inspected; if the
   desktop app ever needs the IDE bridge, that is a separate surface.
8. **Agent precedence personal-vs-project** is taken from the docs' explicit "load in that order and
   the first definition of a name wins"; the bundle keeps the two lists separate, so the final
   arbitration happens in code I did not decompile to a definitive branch.

## Sources

**On disk (this machine):**
- `C:\nvm4w\nodejs\node_modules\command-code\package.json` — version 1.88.0, bins, files, deps
- `C:\nvm4w\nodejs\node_modules\command-code\CHANGELOG.md` — L2392 (`.agents/skills` in 0.18.9), L1137‑1184 (1.0.0), L667 (1.38.2)
- `C:\nvm4w\nodejs\node_modules\command-code\dist\cli.mjs` — symbols: `Kl`, `zl`, `fl`, `yl`, `_a=10`, `BC`, `resolveProjectDir`, `getProjectSkillsDir`, `getGlobalSkillsDir`, `getProjectAgentsCompatSkillsDir`, `getGlobalAgentsCompatSkillsDir`, `bundledSkillsDir`, `createSkillRegistry`, `walkUpToFindDir`, `bundledSkillsDir`, `deduplicateSkills`, `filterDisabledSkills`, `createDisabledSkillsStore`, `disabledNamesIn`, `removeSkill`, `discoverAgents`, `userAgentDir`, `projectAgentDirs`, `scanAgentsDir`, `deleteAgentDefinition`, `isReservedAgentName`, `createAgentRegistryRefreshMod`, `loadMcpServers`, `readMcpJson`, `readUserSettingsMcpServers`, `mcpClientsFromConfig`, `toServerConfig`, `inferTransport`, `getAllMcpConfigPaths`, `getMcpConfigPath`, `saveMcpConfig`, `readRawMcpConfigForWrite`, `addMcpServer`, `setMcpServerEnabled`, `vaultPath`, `localMcpConfigPath`, `slugifyProjectPath`, `getConfigFile`, `writeCommandAuthFileObject`, `settingsPath`, `getProjectLocalSettingsPath`, `getSettingsProjectSharedPath`, `getSettingsUserPath`, `loadCustomCommands`, `resolvePlanContent`, `isPlanPath`, `tasteDirForRoot`, `initTasteDir`, `defaultHistoryPath`, `resolveCacheDir`, `tempPathFor`
- `C:\nvm4w\nodejs\node_modules\command-code\dist\bundled\command-code-knowledge\reference\{skills.md, mcp.md, custom-agents.md, custom-slash-commands.md, memory.md, plans.md}` — in-package doc bundle
- `C:\nvm4w\nodejs\node_modules\command-code\dist\bundled\config\SKILL.md`, `…\config\references\settings.md`
- `C:\nvm4w\nodejs\node_modules\command-code\dist\bundled\mod-builder\SKILL.md`, `…\skill-builder\SKILL.md`
- `C:\nvm4w\nodejs\cmdc.ps1`, `C:\nvm4w\nodejs\cmdc.cmd`
- `C:\Users\raosu\.commandcode\{config.json, mcp.json, auth.json (redacted), updates.json, history.jsonl}`; `…\.commandcode\skills\` (44 junctions), `…\.commandcode\projects\`, `…\.commandcode\cache\model-catalog\`, `…\.commandcode\file-history\`
- `C:\Users\raosu\.agents\{mcp.json, .skill-lock.json, skills\*\SKILL.md}`
- `C:\Users\raosu\SuryapalDev\Projects\randoms\isolated-environment\.commandcode\{settings.json, taste\taste.md}`

**Official docs (live, fetched):**
- <https://commandcode.ai/docs> (site root / package author URL)
- <https://commandcode.ai/docs/reference/cli> — `cmdc` on Windows, subcommands (`cmd skills`, `cmd mcp`, `cmd mods`, `cmd taste`), `--skill`, `--no-skills`, `--mod`, `--safe-mode`
- <https://commandcode.ai/docs/skills> — discovery, `.agents` compat, selection priority, frontmatter, `disabledSkills`, `cmd skills add|list|remove`
- <https://commandcode.ai/docs/agents> (served for `/docs/core-concepts/custom-agents`) — agent locations, precedence, frontmatter fields, reserved names
- <https://commandcode.ai/docs/mcp> — scopes, precedence, config schema, `cmd mcp` reference, OAuth, `mcp-tokens.json`
- Referenced but not fetched: `/docs/memory`, `/docs/taste`, `/docs/settings`, `/docs/mods`, `/docs/import`, `/docs/reference/slash-commands`
