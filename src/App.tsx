import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import ActionDialog from "./components/ActionDialog";
import SettingsPanel from "./components/SettingsPanel";
import TrashPanel from "./components/TrashPanel";
import { createFakeClient, tauriClient, type CommandClient } from "./api";
import type {
  ActionPlan,
  AppSettings,
  Artifact,
  ArtifactType,
  Ecosystem,
  EcosystemDetection,
  HistoryEntry,
  ScanResult,
  Scope,
  SnapshotDiff,
  TrashEntry,
} from "./types";

const ECOSYSTEMS: { id: Ecosystem; label: string }[] = [
  { id: "claude", label: "Claude Code" },
  { id: "codex", label: "Codex" },
  { id: "opencode", label: "OpenCode" },
  { id: "commandcode", label: "Command Code" },
];

const TABS: { id: ArtifactType; label: string }[] = [
  { id: "agent", label: "Agents" },
  { id: "skill", label: "Skills" },
  { id: "mcp", label: "MCP" },
  { id: "hook", label: "Other" },
];

const SCOPES: { id: Scope; label: string }[] = [
  { id: "global", label: "Global" },
  { id: "project", label: "Project" },
  { id: "projectLocal", label: "Project-local" },
];

export function isOtherType(type: ArtifactType): boolean {
  return type !== "agent" && type !== "skill" && type !== "mcp";
}

function applyTheme(mode: AppSettings["theme"] | undefined) {
  const resolved =
    mode === "dark" ||
    (mode !== "light" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.dataset.theme = resolved ? "dark" : "light";
}

export default function App({
  client = tauriClient,
}: {
  client?: CommandClient;
}) {
  const [scan, setScan] = useState<ScanResult | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [activeScope, setActiveScope] = useState<Scope>("global");
  const [activeTab, setActiveTab] = useState<ArtifactType>("agent");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [recentsOpen, setRecentsOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [expandedEcosystem, setExpandedEcosystem] = useState<Ecosystem | null>(
    null,
  );
  const [activeEcosystem, setActiveEcosystem] = useState<Ecosystem | null>(null);
  const [plan, setPlan] = useState<ActionPlan | null>(null);
  const [actionBusy, setActionBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [trashOpen, setTrashOpen] = useState(false);
  const [trashEntries, setTrashEntries] = useState<TrashEntry[]>([]);
  const [trashBusyId, setTrashBusyId] = useState<string | null>(null);
  const [trashError, setTrashError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    Promise.all([client.scan({}), client.getSettings()])
      .then(([result, loaded]) => {
        if (cancelled) return;
        setScan(result);
        setSettings(loaded);
      })
      .catch((cause: unknown) => {
        if (!cancelled) setError(String(cause));
      });
    return () => {
      cancelled = true;
    };
  }, [client]);

  useEffect(() => {
    applyTheme(settings?.theme);
    if (settings?.theme !== "system") return;
    if (typeof window.matchMedia !== "function") return;
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const listener = () => applyTheme("system");
    query.addEventListener("change", listener);
    return () => query.removeEventListener("change", listener);
  }, [settings?.theme]);

  const chooseProject = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Choose a project folder",
      });
      if (typeof selected === "string") {
        await selectProject(selected);
      }
    } catch (cause) {
      setError(String(cause));
    }
  };

  const selectProject = async (path: string) => {
    try {
      setRecentsOpen(false);
      const result = await client.setProject(path);
      setScan(result);
      setSelectedId(null);
      const loaded = await client.getSettings();
      setSettings(loaded);
      setError(null);
    } catch (cause) {
      setError(String(cause));
    }
  };

  const refresh = async () => {
    try {
      const result = await client.scan({ projectRoot: scan?.projectRoot ?? null });
      setScan(result);
      setError(null);
    } catch (cause) {
      setError(String(cause));
    }
  };

  const saveSettings = async (next: AppSettings) => {
    const saved = await client.saveSettings(next);
    setSettings(saved);
  };

  const refreshRef = useRef<() => void>(() => {});
  refreshRef.current = refresh;

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    listen("agentdeck://changed", () => refreshRef.current())
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const applyScan = (result: ScanResult) => {
    setScan(result);
    setSelectedId((current) =>
      result.artifacts.some((artifact) => artifact.id === current)
        ? current
        : null,
    );
  };

  const openTrash = async () => {
    setTrashOpen(true);
    setTrashError(null);
    try {
      setTrashEntries(await client.listTrash());
    } catch (cause) {
      setTrashError(String(cause));
    }
  };

  const restoreEntry = async (id: string) => {
    setTrashBusyId(id);
    setTrashError(null);
    try {
      const result = await client.restore(id);
      applyScan(result.scan);
      setTrashEntries(await client.listTrash());
    } catch (cause) {
      setTrashError(String(cause));
    } finally {
      setTrashBusyId(null);
    }
  };

  const confirmPlan = async () => {
    if (!plan) return;
    setActionBusy(true);
    setActionError(null);
    try {
      const result = await client.applyPlan(plan);
      applyScan(result.scan);
      setPlan(null);
    } catch (cause) {
      setActionError(String(cause));
    } finally {
      setActionBusy(false);
    }
  };

  const artifacts = scan?.artifacts ?? [];
  const visible = artifacts.filter(
    (artifact) =>
      artifact.scope === activeScope &&
      (activeTab === "hook"
        ? isOtherType(artifact.artifactType)
        : artifact.artifactType === activeTab) &&
      (!activeEcosystem ||
        artifact.ecosystems.includes(activeEcosystem) ||
        artifact.ecosystem === activeEcosystem),
  );
  const selected =
    artifacts.find((artifact) => artifact.id === selectedId) ?? null;
  const detected = scan?.detections.filter((entry) => entry.detected) ?? [];
  const recents = settings?.recentProjects ?? [];

  return (
    <div className="app">
      <aside className="rail">
        <div className="brand">
          <span className="brand-mark">A</span>
          <span className="brand-name">AgentDeck</span>
        </div>
        <div className="rail-section-label">Ecosystems</div>
        {ECOSYSTEMS.map((ecosystem) => {
          const detection = scan?.detections.find(
            (entry) => entry.ecosystem === ecosystem.id,
          );
          const expanded = expandedEcosystem === ecosystem.id;
          return (
            <div className="rail-group" key={ecosystem.id}>
              <button
                className="rail-item"
                data-detected={detection?.detected ?? false}
                data-active={activeEcosystem === ecosystem.id}
                onClick={() => {
                  if (detection?.detected) {
                    setActiveEcosystem((current) =>
                      current === ecosystem.id ? null : ecosystem.id,
                    );
                  } else {
                    setExpandedEcosystem(expanded ? null : ecosystem.id);
                  }
                }}
              >
                <span className="dot" />
                <span>{ecosystem.label}</span>
                <span className="rail-meta">
                  {detection?.detected
                    ? detection.version ?? "detected"
                    : detection
                      ? "not found"
                      : ""}
                </span>
              </button>
              {expanded && detection && !detection.detected ? (
                <div className="rail-paths">
                  <p>{detection.reason}</p>
                  <ul>
                    {detection.checkedPaths.map((path) => (
                      <li key={path}>{path}</li>
                    ))}
                  </ul>
                </div>
              ) : null}
            </div>
          );
        })}
        <div className="rail-footer">
          {scan ? `${detected.length} of 4 detected` : "Scanning…"}
        </div>
      </aside>

      <main className="main">
        <header className="topbar">
          <div className="project-summary">
            <span className="project-label">Project</span>
            <span className="project-path">
              {scan?.projectRoot ?? "No project selected"}
            </span>
            {scan?.projectRoot && !scan.isGitRepository ? (
              <span className="badge">not a git repository</span>
            ) : null}
          </div>
          <div className="topbar-actions">
            {recents.length > 0 ? (
              <div className="recents">
                <button
                  className="button button-quiet"
                  onClick={() => setRecentsOpen((value) => !value)}
                >
                  Recent
                </button>
                {recentsOpen ? (
                  <div className="recents-menu">
                    {recents.map((path) => (
                      <button
                        key={path}
                        className="recents-item"
                        onClick={() => selectProject(path)}
                      >
                        {path}
                      </button>
                    ))}
                  </div>
                ) : null}
              </div>
            ) : null}
            <button className="button" onClick={chooseProject}>
              Choose project
            </button>
            <button className="button button-quiet" onClick={refresh}>
              Refresh
            </button>
            <button
              className="button button-quiet"
              onClick={() => setSettingsOpen(true)}
            >
              Settings
            </button>
            <button className="button button-quiet" onClick={openTrash}>
              Trash
            </button>
          </div>
        </header>

        <div className="scopebar">
          {SCOPES.map((scope) => (
            <button
              key={scope.id}
              className="scope"
              data-active={activeScope === scope.id}
              onClick={() => setActiveScope(scope.id)}
            >
              {scope.label}
            </button>
          ))}
        </div>

        <nav className="tabs">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className="tab"
              data-active={activeTab === tab.id}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </nav>

        <div className="workspace">
          <section className="list" aria-label="Artifacts">
            {error ? <div className="error">{error}</div> : null}
            {scan?.warnings.map((warning) => (
              <div className="scan-warning" key={warning}>
                {warning}
              </div>
            ))}
            {scan && visible.length === 0 ? (
              <div className="empty">
                <p className="empty-title">Nothing here yet</p>
                <p className="empty-body">
                  {activeScope === "global"
                    ? "No global artifacts of this type were found."
                    : "Select a project to see artifacts for this scope."}
                </p>
                <details className="checked-paths">
                  <summary>Paths checked</summary>
                  <ul>
                    {scan.detections.flatMap((detection) =>
                      detection.checkedPaths.map((path) => (
                        <li key={`${detection.ecosystem}-${path}`}>{path}</li>
                      )),
                    )}
                  </ul>
                </details>
              </div>
            ) : null}
            {visible.map((artifact) => (
              <button
                key={artifact.id}
                className="artifact-row"
                data-selected={selectedId === artifact.id}
                data-disabled={artifact.enabled === false}
                onClick={() => setSelectedId(artifact.id)}
              >
                <span className="artifact-name">{artifact.name}</span>
                <span className="artifact-desc">
                  {artifact.description ?? artifact.primaryPath}
                </span>
                <span className="artifact-badges">
                  {artifact.ecosystems.map((ecosystem) => (
                    <span className="badge" key={ecosystem}>
                      {ecosystem}
                    </span>
                  ))}
                  {artifact.shadowed.length > 0 ? (
                    <span className="badge badge-muted">
                      {artifact.shadowed.length} shadowed
                    </span>
                  ) : null}
                  {artifact.enabled === false ? (
                    <span className="badge badge-muted">disabled</span>
                  ) : null}
                </span>
              </button>
            ))}
          </section>

          <aside className="detail">
            {selected ? (
              <ArtifactDetail
                artifact={selected}
                client={client}
                detection={scan?.detections.find(
                  (entry) => entry.ecosystem === selected.ecosystem,
                )}
                onPlan={(next) => {
                  setActionError(null);
                  setPlan(next);
                }}
                onScan={applyScan}
              />
            ) : (
              <div className="empty">
                <p className="empty-title">Select an artifact</p>
                <p className="empty-body">
                  Details, source paths and actions appear here.
                </p>
              </div>
            )}
          </aside>
        </div>
      </main>

      {settingsOpen && settings ? (
        <SettingsPanel
          settings={settings}
          onSave={saveSettings}
          onClose={() => setSettingsOpen(false)}
        />
      ) : null}

      {plan ? (
        <ActionDialog
          plan={plan}
          busy={actionBusy}
          error={actionError}
          onConfirm={confirmPlan}
          onCancel={() => {
            if (!actionBusy) setPlan(null);
          }}
        />
      ) : null}

      {trashOpen ? (
        <TrashPanel
          entries={trashEntries}
          busyId={trashBusyId}
          error={trashError}
          onRestore={restoreEntry}
          onClose={() => setTrashOpen(false)}
        />
      ) : null}
    </div>
  );
}

function ArtifactDetail({
  artifact,
  client,
  detection,
  onPlan,
  onScan,
}: {
  artifact: Artifact;
  client: CommandClient;
  detection?: EcosystemDetection;
  onPlan: (plan: ActionPlan) => void;
  onScan: (scan: ScanResult) => void;
}) {
  const [showRaw, setShowRaw] = useState(false);
  const [raw, setRaw] = useState<Record<string, string>>({});
  const [rawError, setRawError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [history, setHistory] = useState<HistoryEntry[] | null>(null);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const [historyBusy, setHistoryBusy] = useState(false);
  const [diff, setDiff] = useState<SnapshotDiff | null>(null);

  const isDisabled = artifact.enabled === false;
  const capability =
    artifact.artifactType === "agent"
      ? "disable-agent"
      : artifact.artifactType === "skill"
        ? "disable-skill"
        : "disable-mcp";
  const quarantineAvailable = artifact.ecosystems.some(
    (ecosystem) => ecosystem === "codex" || ecosystem === "commandcode",
  );
  const gated = Boolean(
    artifact.managed &&
      detection?.detected &&
      detection.capabilities.length > 0 &&
      !detection.capabilities.includes(capability) &&
      !(quarantineAvailable && detection.capabilities.includes("disable-quarantine")),
  );

  useEffect(() => {
    setShowRaw(false);
    setRaw({});
    setRawError(null);
    setActionError(null);
    setHistoryOpen(false);
    setHistory(null);
    setHistoryError(null);
    setDiff(null);
  }, [artifact.id]);

  useEffect(() => {
    if (!showRaw) return;
    let cancelled = false;
    Promise.all(
      artifact.sourcePaths.map(async (path) => {
        try {
          return [path, await client.readFile(path)] as const;
        } catch (cause) {
          if (!cancelled) setRawError(String(cause));
          return [path, ""] as const;
        }
      }),
    ).then((entries) => {
      if (!cancelled) setRaw(Object.fromEntries(entries));
    });
    return () => {
      cancelled = true;
    };
  }, [artifact.id, artifact.sourcePaths, client, showRaw]);

  const requestToggle = async () => {
    setActionError(null);
    try {
      onPlan(await client.planToggle(artifact.id, isDisabled));
    } catch (cause) {
      setActionError(String(cause));
    }
  };

  const requestRemove = async () => {
    setActionError(null);
    try {
      onPlan(await client.planRemove(artifact.id));
    } catch (cause) {
      setActionError(String(cause));
    }
  };

  const loadHistory = async () => {
    setHistoryOpen(true);
    setHistoryBusy(true);
    setHistoryError(null);
    try {
      setHistory(await client.listHistory(artifact.primaryPath));
    } catch (cause) {
      setHistoryError(String(cause));
    } finally {
      setHistoryBusy(false);
    }
  };

  const rollbackEntry = async (snapshotPath: string) => {
    setHistoryBusy(true);
    setHistoryError(null);
    try {
      const result = await client.rollback(snapshotPath);
      onScan(result.scan);
      setHistory(await client.listHistory(artifact.primaryPath));
    } catch (cause) {
      setHistoryError(String(cause));
    } finally {
      setHistoryBusy(false);
    }
  };

  const pruneEntry = async (id: string) => {
    setHistoryBusy(true);
    setHistoryError(null);
    try {
      await client.pruneSnapshots([id]);
      setHistory(await client.listHistory(artifact.primaryPath));
    } catch (cause) {
      setHistoryError(String(cause));
    } finally {
      setHistoryBusy(false);
    }
  };

  const showDiff = async (snapshotPath: string) => {
    setHistoryError(null);
    try {
      setDiff(await client.diffSnapshot(snapshotPath));
    } catch (cause) {
      setHistoryError(String(cause));
    }
  };

  return (
    <div className="detail-body">
      <h2>{artifact.name}</h2>
      <p className="detail-path">{artifact.primaryPath}</p>
      {artifact.description ? <p>{artifact.description}</p> : null}
      <dl className="meta">
        <dt>Scope</dt>
        <dd>{artifact.scope}</dd>
        <dt>Type</dt>
        <dd>{artifact.artifactType}</dd>
        <dt>Status</dt>
        <dd>
          {artifact.enabled === false ? "disabled" : "enabled"}
          {artifact.disableMechanism ? ` · ${artifact.disableMechanism}` : ""}
        </dd>
        <dt>Reload</dt>
        <dd>{artifact.reload}</dd>
        {detection ? (
          <>
            <dt>Version</dt>
            <dd>{detection.version ?? "unknown"}</dd>
            <dt>Capabilities</dt>
            <dd>
              {detection.capabilities.length > 0
                ? detection.capabilities.join(", ")
                : "unavailable"}
            </dd>
          </>
        ) : null}
      </dl>

      {detection?.detected && detection.capabilities.length === 0 && artifact.managed ? (
        <p className="warning">
          The installed {artifact.ecosystem} version could not be read, so
          actions that depend on it are unavailable. Set the binary path in
          Settings.
        </p>
      ) : null}

      {artifact.warnings.map((warning) => (
        <p className="warning" key={warning}>
          {warning}
        </p>
      ))}

      {artifact.managed ? (
        <section className="actions">
          <button
            className="button button-primary"
            onClick={requestToggle}
            disabled={gated}
            title={
              gated
                ? "The installed version does not advertise this action."
                : undefined
            }
          >
            {isDisabled ? "Enable" : "Disable"}
          </button>
          <button className="button button-danger" onClick={requestRemove}>
            Remove
          </button>
        </section>
      ) : (
        <p className="settings-hint">Read-only in AgentDeck.</p>
      )}
      {actionError ? <div className="error">{actionError}</div> : null}

      {artifact.shadowed.length > 0 ? (
        <section className="shadowed">
          <h3>Shadowed definitions</h3>
          {artifact.shadowed.map((shadowed) => (
            <div className="shadowed-item" key={`${shadowed.path}-${shadowed.name}`}>
              <span className="detail-path">{shadowed.path}</span>
              <span className="shadowed-reason">{shadowed.reason}</span>
            </div>
          ))}
        </section>
      ) : null}

      <section className="raw">
        <div className="raw-header">
          <h3>Source</h3>
          <button
            className="button button-quiet"
            onClick={() => setShowRaw((value) => !value)}
          >
            {showRaw ? "Hide raw" : "Show raw"}
          </button>
        </div>
        {rawError ? <p className="error">{rawError}</p> : null}
        {showRaw
          ? artifact.sourcePaths.map((path) => (
              <div className="raw-file" key={path}>
                <div className="detail-path">{path}</div>
                <pre>{raw[path] ?? "Loading…"}</pre>
              </div>
            ))
          : null}
      </section>

      <section className="raw">
        <div className="raw-header">
          <h3>History</h3>
          <button className="button button-quiet" onClick={loadHistory}>
            {historyOpen ? "Refresh" : "Show history"}
          </button>
        </div>
        {historyError ? <p className="error">{historyError}</p> : null}
        {historyBusy ? <p className="settings-hint">Working…</p> : null}
        {historyOpen && history && history.length === 0 ? (
          <p className="settings-hint">No snapshots for this file yet.</p>
        ) : null}
        {historyOpen && history ? (
          <ul className="history-list">
            {history.map((entry) => (
              <li key={entry.id}>
                <div className="history-meta">
                  <span className="badge">{entry.action}</span>
                  <span className="settings-hint">{entry.timestamp}</span>
                </div>
                <div className="history-actions">
                  <button
                    className="button button-quiet"
                    onClick={() => showDiff(entry.snapshotPath)}
                  >
                    Diff
                  </button>
                  <button
                    className="button button-quiet"
                    onClick={() => rollbackEntry(entry.snapshotPath)}
                    disabled={historyBusy}
                  >
                    Roll back
                  </button>
                  <button
                    className="button button-quiet"
                    onClick={() => pruneEntry(entry.id)}
                    disabled={historyBusy}
                  >
                    Delete
                  </button>
                </div>
              </li>
            ))}
          </ul>
        ) : null}
      </section>

      {diff ? (
        <div className="modal-backdrop" onClick={() => setDiff(null)}>
          <div
            className="modal modal-wide"
            role="dialog"
            aria-label="Snapshot diff"
            onClick={(event) => event.stopPropagation()}
          >
            <header className="modal-header">
              <h2>Snapshot diff</h2>
              <button className="button button-quiet" onClick={() => setDiff(null)}>
                Close
              </button>
            </header>
            <p className="detail-path">{diff.path}</p>
            <pre className="diff">{diff.diff || "No differences."}</pre>
          </div>
        </div>
      ) : null}
    </div>
  );
}

export { createFakeClient };
