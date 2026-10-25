# 11: Disable MCP servers

**What to build:** Per-server disable and enable using each ecosystem's `enabled`/`disabled` flag in the correct scope. Configuration is never removed by a disable. Because several ecosystems only build their server list at startup, the result states plainly whether the change applies immediately, on the next turn, or after a restart or reconnect.

**Blocked by:** 06, 10

**Status:** ready-for-agent

- [ ] Each ecosystem's server disable writes the correct flag in the file that defines that server
- [ ] Enabling restores the previous value and leaves every other field untouched
- [ ] The action honours the selected scope and never edits a different scope's definition
- [ ] For the ecosystem whose file also carries unrelated per-project state, the disable touches only the server list and preserves that state
- [ ] TOML and JSONC files are edited surgically, preserving comments and formatting
- [ ] The result explains whether a restart or reconnect is needed
- [ ] A server whose defining file changed since the scan aborts the write and prompts a rescan
- [ ] A malformed file is refused rather than rewritten
