# 02: Claude Code agents end-to-end

**What to build:** The first complete path through every layer. A user picks a project folder, the app resolves the project root to the enclosing git root (badging a non-git folder), remembers the project in a recents list, and scans Claude Code agents at Global and Project scopes. The list shows the effective definition per name, shadowed definitions mark and collapse under it, and the raw inspector shows every source path with its content. Unreadable or malformed files produce per-file warnings instead of failing the scan.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Selecting a folder resolves and displays the project root, with a badge when it is not a git repository
- [ ] Recents persist across restarts and reopen a project in one click
- [ ] Global and Project scopes list Claude Code agents with name, description, origin path and scope
- [ ] A same-named agent in two scopes shows one effective entry and one clearly marked shadowed entry
- [ ] The raw inspector lists every contributing path and its exact content
- [ ] A malformed agent file yields a warning on that entry while the rest of the list still loads
- [ ] An empty result shows an empty state naming the paths that were checked
- [ ] Core tests drive these behaviours against synthetic fixture trees through the injected roots, never a real home directory
