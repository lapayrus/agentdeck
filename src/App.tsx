import { useEffect, useState } from "react";
import { createFakeClient, tauriClient, type CommandClient } from "./api";
import type {
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

export default function App({
  client = tauriClient,
}: {
  client?: CommandClient;
}) {
  const [scan, setScan] = useState<ScanResult | null>(null);
  const [activeScope, setActiveScope] = useState<Scope>("global");
  const [activeTab, setActiveTab] = useState<ArtifactType>("agent");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    client
      .scan({})
      .then((result) => {
        if (!cancelled) setScan(result);
      })
      .catch((cause: unknown) => {
        if (!cancelled) setError(String(cause));
      });
    return () => {
      cancelled = true;
    };
  }, [client]);

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
                {detection?.version ?? (detection ? "not found" : "")}
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
            <button className="button" disabled>
              Choose project
            </button>
            <button className="button button-quiet" disabled>
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
            {scan && visible.length === 0 ? (
              <div className="empty">
                <p className="empty-title">Nothing here yet</p>
                <p className="empty-body">
                  {activeScope === "global"
                    ? "No global artifacts of this type were found."
                    : "Select a project to see artifacts for this scope."}
                </p>
              </div>
            ) : null}
            {visible.map((artifact) => (
              <button
                key={artifact.id}
                className="artifact-row"
                data-selected={selectedId === artifact.id}
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
                </span>
              </button>
            ))}
          </section>

          <aside className="detail">
            {selected ? (
              <ArtifactDetail artifact={selected} />
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

function ArtifactDetail({ artifact }: { artifact: Artifact }) {
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
        <dd>{artifact.enabled === false ? "disabled" : "enabled"}</dd>
        <dt>Reload</dt>
        <dd>{artifact.reload}</dd>
      </dl>
      {artifact.warnings.map((warning) => (
        <p className="warning" key={warning}>
          {warning}
        </p>
      ))}
    </div>
  );
}

export { createFakeClient };
