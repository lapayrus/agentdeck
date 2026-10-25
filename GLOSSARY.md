# Agent Configuration Manager

A Windows desktop app that shows and controls the agents, skills, and MCP servers installed for several AI coding tools, in one project and globally.

## Language

**Ecosystem**:
One vendor's agent product whose configuration the app reads, such as Claude Code, Codex, OpenCode, or Command Code.
_Avoid_: Tool, provider, platform

**Artifact**:
A single definition that an ecosystem loads at runtime, such as one agent, one skill, or one MCP server.
_Avoid_: Item, entry, component

**Managed artifact**:
An artifact the app can enable, disable, and remove. In V1 these are agents, skills, and MCP servers.
_Avoid_: Editable artifact, writable artifact

**Read-only artifact**:
An artifact the app displays but does not change, such as hooks, slash commands, plugins, or memory files.
_Avoid_: Unsupported artifact, other artifact

**Project root**:
The directory the user selected, resolved upward to the enclosing git root when one exists.
_Avoid_: Workspace, folder, repo

**Global scope**:
Artifacts that apply to every project on the machine, stored in the user's home directory.
_Avoid_: User scope, machine scope, personal

**Project scope**:
Artifacts stored inside the project root, which anyone who checks out the project receives.
_Avoid_: Workspace scope, repository scope, shared

**Project-local scope**:
Artifacts scoped to one project but stored outside the project root, so they are not shared with collaborators.
_Avoid_: Local scope, private project scope

**Effective state**:
The single definition an ecosystem will actually load for a given artifact name, after applying that ecosystem's precedence rules.
_Avoid_: Resolved config, merged config, final state

**Shadowed**:
An artifact definition that exists on disk but loses precedence to another definition of the same name, so the ecosystem does not load it.
_Avoid_: Duplicate, overridden copy, inactive

**Disable**:
Make an ecosystem stop loading an artifact while leaving its definition and content in place.
_Avoid_: Turn off, deactivate, toggle off

**Remove**:
Take an artifact out of service by moving it into the app's trash, from which it can be restored.
_Avoid_: Delete, uninstall, purge

**Quarantine**:
The app-managed holding area for a disabled artifact whose ecosystem offers no native disable switch.
_Avoid_: Disabled folder, recycle bin, graveyard

**Restore**:
Return a removed artifact from the app's trash to the location it came from.
_Avoid_: Undo delete, recover
