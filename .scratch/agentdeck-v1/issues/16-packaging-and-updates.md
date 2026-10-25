# 16: Packaging and one-click updates

**What to build:** Release builds of AgentDeck as an MSI and an NSIS installer, with signed updater artifacts published to GitHub Releases by a tag-triggered workflow. The app checks for updates at launch with a throttle and on demand, shows the available version, and installs it in one click after verifying the signature. The check can be disabled, and it remains the app's only network request.

**Blocked by:** 13, 03

**Status:** ready-for-agent

- [ ] A tagged push produces a draft GitHub Release containing an MSI, an NSIS installer and the signed updater artifacts
- [ ] The MSI installs machine-wide and the NSIS installer installs per-user
- [ ] The app checks for updates at launch, subject to a throttle, and via a manual check
- [ ] An available update is shown with its version and installs in one click
- [ ] An artifact with an invalid signature is refused with a clear error
- [ ] The update check can be disabled in settings, after which the app makes no network request
- [ ] No other network host is contacted by the app
- [ ] The repository is MIT licensed and a release includes the license and third-party notices
