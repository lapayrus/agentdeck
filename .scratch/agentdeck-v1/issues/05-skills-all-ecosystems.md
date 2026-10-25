# 05: Skills across all four ecosystems

**What to build:** A skills view covering every ecosystem, including the compatibility roots and the shared roots one directory can serve. Each ecosystem's own precedence applies, but a directory loaded by several ecosystems is one entry with several badges. Disabled skills are visible with the mechanism that disabled them.

**Blocked by:** 04

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1.

- [ ] All four ecosystems list skills with name, description, origin path, scope and ecosystem badges
- [ ] Codex skills resolve from the `.agents` roots, never from `.codex`, and the fixture proves it
- [ ] Compatibility roots are honoured for OpenCode and Command Code
- [ ] A directory loaded by three ecosystems appears once, with the affected-ecosystem set available to later confirmation dialogs
- [ ] Skill precedence per ecosystem is applied and shadowed copies are marked
- [ ] A skill directory missing its manifest, or with an invalid name, shows as a warning rather than disappearing silently
- [ ] Bundled skills shipped inside an ecosystem's installation are distinguished from user-authored ones
