# Non-destructive mutation of other tools' configuration

## Context

The app disables and removes artifacts belonging to four other ecosystems, and edits files those tools also write: `~/.claude.json` (which carries per-project trust state), Codex `config.toml` (which the tool rewrites itself), OpenCode `opencode.json` (JSONC with comments), and Command Code `settings.json` and `mcp.json`. A bad write can lose user content or trigger a tool's config-corruption reset. Removal from a manager UI is also irreversible from the user's point of view if the file is deleted outright.

## Decision

Every mutation is reversible and format-preserving:

- A timestamped snapshot of the whole affected file goes to the app's `history/` before any write, and the edit is applied surgically so comments and formatting survive.
- Removal moves the artifact into the app's `trash/` with a manifest; nothing is hard-deleted.
- Artifacts whose ecosystem has no native disable switch are disabled by moving them into a `quarantine/` area, not by renaming them in place.
- Every write re-reads the file and compares a content hash with the last scan, aborting and prompting a rescan on mismatch.

## Consequences

Disk usage grows with history and trash, so both need retention policies. The app must implement format-preserving editors per file type rather than serialising parsed objects. Restoring a config-key artifact can conflict with later edits made by the user or the ecosystem, so restore must surface a conflict rather than overwrite.
