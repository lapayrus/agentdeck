# 14: Live refresh and reload hints

**What to build:** The view stays truthful while the app is open. Watchers cover every discovery root, debounced, and trigger a rescan when files change. A manual refresh rescans everything. Each ecosystem and artifact family shows a reload hint describing whether a change applies on the next turn, the next session, or after a restart, so a toggle that appears to do nothing is explained.

**Blocked by:** 06

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1.

- [ ] Adding, editing or deleting a file in a watched root updates the list within the debounce window
- [ ] A burst of changes produces one rescan, not one per file event
- [ ] Manual refresh rescans all ecosystems and scopes
- [ ] Watch failures fall back to manual refresh and surface a non-blocking warning
- [ ] Every ecosystem and artifact family shows its documented reload requirement
- [ ] The reload hint is visible from the detail pane and after a successful action
- [ ] Watching does not follow paths outside the discovery manifest
- [ ] A change made by the app itself does not cause an endless rescan loop
