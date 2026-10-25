# 01: Walking skeleton

**What to build:** A Tauri 2 app that launches into the single-window shell: left rail of ecosystems, scope switch, artifact-type tabs, empty list and empty detail pane. A UI-free Rust core crate and a React frontend exist with one passing test each, and CI builds the app on Windows.

**Blocked by:** None (can start immediately)

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1.

- [ ] The app launches on Windows and shows the shell with all four regions present
- [ ] The core crate builds and has at least one passing test that exercises a public function
- [ ] The frontend builds and has at least one passing component test
- [ ] CI runs the Rust build, Rust tests, frontend build and frontend tests on Windows
- [ ] The frontend has no filesystem capability; all disk access is reachable only through a Rust command
- [ ] The window title and package metadata use the name AgentDeck
