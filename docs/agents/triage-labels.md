# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the strings used in this repo.

| Label in mattpocock/skills | Label in our tracker | Meaning                                  |
| -------------------------- | -------------------- | ---------------------------------------- |
| `needs-triage`             | `needs-triage`       | Maintainer needs to evaluate this issue  |
| `needs-info`               | `needs-info`         | Waiting on reporter for more information |
| `ready-for-agent`          | `ready-for-agent`    | Fully specified, ready for an AFK agent  |
| `ready-for-human`          | `ready-for-human`    | Requires human implementation            |
| `wontfix`                  | `wontfix`            | Will not be actioned                     |

This repo uses the local-markdown tracker, so a label is recorded as a `Status: <label>` line near the top of the issue file rather than applied through a tracker CLI.

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), use the corresponding string from this table.
