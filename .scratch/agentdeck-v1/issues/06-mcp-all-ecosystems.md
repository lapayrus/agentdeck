# 06: MCP servers across all four ecosystems

**What to build:** An MCP view that reads every ecosystem's configuration, across all three scopes. Servers show transport, command or URL, enabled state and the exact file and key that define them. Project-local scope is a first-class filter, so the user can see servers that are private to this machine for this project.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] All four ecosystems list servers with transport, endpoint or command, enabled state, origin file and key path
- [ ] JSON, JSONC and TOML sources are read without being rewritten
- [ ] Global, Project and Project-local scopes are distinguished, including the per-project local files that live outside the project root
- [ ] A server defined in more than one scope shows the winner as effective and the loser as shadowed, per that ecosystem's rule
- [ ] Servers whose defining file is malformed show a warning naming the file, without hiding other servers
- [ ] A file that an ecosystem does not actually read is never presented as a source
- [ ] Fixtures cover stdio, remote HTTP and remote SSE server shapes
