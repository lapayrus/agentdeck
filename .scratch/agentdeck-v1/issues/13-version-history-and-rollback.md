# 13: Version history and rollback

**What to build:** A history browser showing every snapshot the app has taken, per file. The user can compare a snapshot with the current file and roll the file back. Rolling back is itself snapshotted, so it is undoable. Pruning is manual and explicit.

**Blocked by:** 12

**Status:** ready-for-agent

- [ ] Every write performed by the app appears in history with timestamp, action and affected file
- [ ] A snapshot can be compared with the current file, with added, removed and changed lines visible
- [ ] Rolling back replaces the file with the snapshot after taking a fresh snapshot of the current content
- [ ] Rolling back a file that changed since the snapshot warns before proceeding
- [ ] Pruning removes selected snapshots or everything older than a chosen date, after confirmation
- [ ] History entries link to the trash or quarantine item they belong to, where one exists
- [ ] History is readable after an app restart and reports a damaged store without losing other entries
