# 07: Read-only artifacts

**What to build:** An Other view that reports everything the app does not manage: hooks, slash commands, plugins and marketplaces, memory and instruction files, and settings files. Each entry shows its path and offers open and reveal actions. Nothing here can be changed from the app.

**Blocked by:** 04

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1.

- [ ] Hooks, slash commands, plugins and marketplaces, memory and instruction files, and settings files are listed per ecosystem with their scope and path
- [ ] Every entry offers open in the default application and reveal in Explorer
- [ ] No control in this view can modify a file
- [ ] A missing category shows an empty state naming the checked paths rather than an error
- [ ] Instruction files that several ecosystems read appear once with badges
- [ ] Fixtures cover a plugin/marketplace install and a settings file carrying per-project data
