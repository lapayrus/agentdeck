import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { createFakeClient, tauriClient, type CommandClient } from "./api";
import type {
  AppSettings,
  Artifact,
  ArtifactType,
  Ecosystem,
  ScanResult,
  Scope,
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

  const artifacts = scan?.artifacts ?? [];
  const visible = artifacts.filter(
    (artifact) =>
      artifact.scope === activeScope &&
      (activeTab === "hook"
        ? isOtherType(artifact.artifactType)
        : artifact.artifactType === activeTab),
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
          return (
            <button
              key={ecosystem.id}
              className="rail-item"
              data-detected={detection?.detected ?? false}
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
              <ArtifactDetail artifact={selected} client={client} />
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
    </div>
  );
}

function ArtifactDetail({
  artifact,
  client,
}: {
  artifact: Artifact;
  client: CommandClient;
}) {
  const [showRaw, setShowRaw] = useState(false);
  const [raw, setRaw] = useState<Record<string, string>>({});
  const [rawError, setRawError] = useState<string | null>(null);

  useEffect(() => {
    setShowRaw(false);
    setRaw({});
    setRawError(null);
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
      </dl>

      {artifact.warnings.map((warning) => (
        <p className="warning" key={warning}>
          {warning}
        </p>
      ))}

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
    </div>
  );
}

export { createFakeClient };
