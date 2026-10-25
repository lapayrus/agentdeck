# 08: Ecosystem detection and capability gating

**What to build:** The app detects which ecosystems are present by their config roots, probes each installed version, and respects a user-supplied binary override when a tool is installed outside `PATH`. An ecosystem that is not detected appears collapsed with the paths that were checked. A capability matrix records what each detected version supports, so later action tickets can disable controls that the installed version cannot honour, with the reason shown.

**Blocked by:** 04, 03

**Status:** ready-for-agent

- [ ] An ecosystem with no config root appears as a collapsed not-detected row listing the checked paths
- [ ] A detected ecosystem shows its version, or an explicit unknown when the probe fails
- [ ] A binary override makes version detection work for a tool installed outside `PATH`
- [ ] The capability matrix is per ecosystem and per version and is surfaced in the detail pane
- [ ] No action is offered as available when the detected version cannot support it
- [ ] Detection and probing are tested through injected fakes, not by running real vendor binaries
