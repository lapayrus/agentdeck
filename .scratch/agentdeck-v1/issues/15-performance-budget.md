# 15: Performance budget

**What to build:** A reproducible benchmark that proves the app is fast and light, run in CI. It builds a synthetic project with 10,000 files across all four ecosystems, measures cold start, full scan and idle memory, and fails the build when the scan budget regresses beyond its allowance.

**Blocked by:** 06

**Status:** done

**Resolution:** Implemented on integration/agentdeck-v1. The 10,000-file benchmark scans in 1.105s on this machine and runs in CI with `--release --ignored`.

- [ ] A synthetic 10,000-file fixture across all ecosystems is generated deterministically by a script
- [ ] Cold start, full scan and idle working set are measured and reported by the benchmark
- [ ] The benchmark fails CI when a full scan exceeds 2 seconds on the reference runner beyond a documented allowance
- [ ] No filesystem work runs on the UI thread, demonstrated by the UI remaining responsive during a scan
- [ ] Idle working set stays under 150 MB across app and webview processes in the measured build
- [ ] Results are printed in a form a human can compare release to release
