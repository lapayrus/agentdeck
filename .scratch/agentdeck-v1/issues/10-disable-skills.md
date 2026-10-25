# 10: Disable skills

**What to build:** Disabling and re-enabling skills using each ecosystem's own mechanism, including the settings-level lists and the permission rules. A disabled skill stays visible and says which mechanism disabled it and which scopes it affects. For a skill directory loaded by several ecosystems, the plan lists each ecosystem's mechanism before anything is written.

**Blocked by:** 05, 09

**Status:** ready-for-agent

- [ ] Claude Code skills disable through the native skill override or an equivalent documented switch and re-enable cleanly
- [ ] Codex skills disable through the per-skill configuration entry and re-enable cleanly
- [ ] OpenCode skills disable through the permission rule and re-enable cleanly
- [ ] Command Code skills disable through the disabled-skills list and re-enable cleanly
- [ ] A disabled skill remains listed, marked, and offers re-enable
- [ ] For a shared skill, the plan names each affected ecosystem and its mechanism
- [ ] Disabling for one scope does not change the other scope
- [ ] Settings files carrying unrelated keys are edited surgically, preserving comments and formatting
- [ ] Every write is snapshotted and hash-checked, with the same abort behaviour as agents
