# 12: Remove and Restore

**What to build:** Removing an artifact moves it into the app-managed trash, with a manifest recording origin, scope, ecosystem and timestamp; configuration-key artifacts are removed from their key after a whole-file snapshot. A restore returns a trashed artifact to its origin, or re-inserts the key, and refuses to overwrite a conflicting later edit. Every removal asks for confirmation and names the affected ecosystems.

**Blocked by:** 11

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1.

- [ ] Removing a file-based artifact moves it to trash and it disappears from the effective list
- [ ] Removing a configuration-key artifact deletes only that key after snapshotting the whole file
- [ ] The trash manifest records origin, scope, ecosystem, timestamp and the associated snapshot
- [ ] Restore returns the artifact to its original path with original content
- [ ] Restore of a configuration-key artifact re-inserts the key without disturbing later unrelated edits
- [ ] Restore refuses and explains when the origin now exists or the file changed incompatibly
- [ ] Removal of a shared artifact lists every affected ecosystem in the confirmation
- [ ] Trash survives an app restart and is browsable
- [ ] Nothing is hard-deleted by the app in V1
