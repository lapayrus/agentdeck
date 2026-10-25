import { useState } from "react";
import type { AppSettings, Ecosystem } from "../types";

const ECOSYSTEM_LABELS: { id: Ecosystem; label: string }[] = [
  { id: "claude", label: "Claude Code" },
  { id: "codex", label: "Codex" },
  { id: "opencode", label: "OpenCode" },
  { id: "commandcode", label: "Command Code" },
];

export default function SettingsPanel({
  settings,
  onSave,
  onClose,
}: {
  settings: AppSettings;
  onSave: (settings: AppSettings) => Promise<void>;
  onClose: () => void;
}) {
  const [draft, setDraft] = useState<AppSettings>(settings);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const update = (next: AppSettings) => {
    setDraft(next);
    setSaving(true);
    setError(null);
    onSave(next)
      .catch((cause: unknown) => setError(String(cause)))
      .finally(() => setSaving(false));
  };

  const setOverride = (ecosystem: Ecosystem, value: string) => {
    const overrides = { ...draft.binaryOverrides };
    const trimmed = value.trim();
    if (trimmed) overrides[ecosystem] = trimmed;
    else delete overrides[ecosystem];
    update({ ...draft, binaryOverrides: overrides });
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-label="Settings"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <h2>Settings</h2>
          <button className="button button-quiet" onClick={onClose}>
            Close
          </button>
        </header>

        {error ? <div className="error">{error}</div> : null}
        {saving ? <div className="saving">Saving…</div> : null}

        <section className="settings-section">
          <h3>Appearance</h3>
          <div className="segmented" role="group" aria-label="Theme">
            {(["light", "dark", "system"] as const).map((theme) => (
              <button
                key={theme}
                className="scope"
                data-active={draft.theme === theme}
                onClick={() => update({ ...draft, theme })}
              >
                {theme === "light"
                  ? "Light"
                  : theme === "dark"
                    ? "Dark"
                    : "System"}
              </button>
            ))}
          </div>
        </section>

        <section className="settings-section">
          <h3>Updates</h3>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={draft.checkForUpdates}
              onChange={(event) =>
                update({ ...draft, checkForUpdates: event.target.checked })
              }
            />
            Check for updates at launch
          </label>
          <p className="settings-hint">
            The update check is the only network request AgentDeck makes.
          </p>
        </section>

        <section className="settings-section">
          <h3>Projects</h3>
          {draft.recentProjects.length === 0 ? (
            <p className="settings-hint">No recent projects yet.</p>
          ) : (
            <ul className="settings-list">
              {draft.recentProjects.map((path) => (
                <li key={path}>
                  <span className="settings-path">{path}</span>
                  <button
                    className="button button-quiet"
                    onClick={() =>
                      update({
                        ...draft,
                        recentProjects: draft.recentProjects.filter(
                          (entry) => entry !== path,
                        ),
                      })
                    }
                  >
                    Remove
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="settings-section">
          <h3>Ecosystem binaries</h3>
          <p className="settings-hint">
            Set a path when a tool is installed outside PATH.
          </p>
          {ECOSYSTEM_LABELS.map((ecosystem) => (
            <div className="override-row" key={ecosystem.id}>
              <label htmlFor={`override-${ecosystem.id}`}>
                {ecosystem.label}
              </label>
              <input
                id={`override-${ecosystem.id}`}
                type="text"
                placeholder="Auto-detect"
                defaultValue={draft.binaryOverrides[ecosystem.id] ?? ""}
                onBlur={(event) => setOverride(ecosystem.id, event.target.value)}
              />
            </div>
          ))}
        </section>
      </div>
    </div>
  );
}
