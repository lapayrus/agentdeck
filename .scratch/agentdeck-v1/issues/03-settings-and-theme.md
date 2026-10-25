# 03: Settings and theme

**What to build:** A settings surface and a Claude-inspired visual system. Light, dark and system themes apply across the whole window; the window remembers its size and position; the settings screen manages the recents list, per-ecosystem binary path overrides and the update-check preference. All preferences persist in one app-data file written atomically.

**Blocked by:** 02

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1.

- [ ] Light, dark and system modes switch without a restart, and system mode follows the OS setting as it changes
- [ ] The palette reads as Claude-inspired: warm off-white surfaces, a single warm accent, serif display headings, generous spacing
- [ ] Window size and position are restored on next launch, including after the window was moved to another monitor
- [ ] Recents can be reordered by use and individually removed
- [ ] Binary path overrides can be set and cleared per ecosystem
- [ ] The update-check preference can be turned off
- [ ] Preferences survive a corrupted state file by falling back to defaults and reporting the reset
- [ ] A component test covers theme switching through the faked command client
