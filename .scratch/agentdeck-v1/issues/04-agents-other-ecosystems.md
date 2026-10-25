# 04: Agents across Codex, OpenCode and Command Code

**What to build:** The agent view covers all four ecosystems. Codex agents are read from TOML, OpenCode from Markdown frontmatter and Command Code from Markdown frontmatter, each with its own discovery roots, recursion rules and precedence order. An artifact reachable through more than one ecosystem's root appears once with ecosystem badges.

**Blocked by:** 02

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1.

- [ ] Each of the three ecosystems lists its agents with the same metadata and raw inspector as Claude Code
- [ ] Codex TOML parses without rewriting the source file
- [ ] Same-named agents within an ecosystem resolve to one effective entry plus shadowed entries, per that ecosystem's precedence
- [ ] A path loaded by more than one ecosystem appears once with one badge per ecosystem
- [ ] Missing or unreadable roots degrade to a warning row rather than an error screen
- [ ] Fixtures cover each ecosystem's malformed-frontmatter case and its reserved-name rule
- [ ] The list marks the scope of every entry, including any project-local origin
