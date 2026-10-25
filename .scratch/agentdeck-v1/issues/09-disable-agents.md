# 09: Disable agents

**What to build:** Disabling and re-enabling agents from the list. Where an ecosystem has a native switch the app writes that switch; where it does not, the definition moves to quarantine. Before anything is written the user sees a plan naming every file and every affected ecosystem, and the action is only applied after confirmation. Every write is snapshotted and hash-checked, and the view refreshes to show the new state.

**Blocked by:** 04, 08

**Status:** ready-for-agent

- [ ] Claude Code agents disable through the native permissions denial and re-enable cleanly
- [ ] OpenCode agents disable through their native disabled flag and re-enable cleanly
- [ ] Codex and Command Code agents, which have no native switch, move to quarantine and re-enable by moving back
- [ ] The confirmation plan lists every file touched and every ecosystem affected
- [ ] A shared artifact warns which other ecosystems change before the action is applied
- [ ] Disabling and re-enabling returns the affected files to byte-identical content where the ecosystem allows it
- [ ] A write aborts with a rescan prompt when the target file changed since the scan
- [ ] An unsupported action is disabled with a reason when the detected version cannot honour it
- [ ] Quarantine mirrors the original path structure and carries a manifest that survives an app restart
- [ ] A broken or unwritable target surfaces an actionable error and leaves the original untouched
