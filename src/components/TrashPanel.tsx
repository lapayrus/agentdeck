import type { TrashEntry } from "../types";

export default function TrashPanel({
  entries,
  busyId,
  error,
  onRestore,
  onForget,
  onClose,
}: {
  entries: TrashEntry[];
  busyId: string | null;
  error: string | null;
  onRestore: (id: string) => void;
  onForget: () => void;
  onClose: () => void;
}) {
  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-label="Trash"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <h2>Trash and quarantine</h2>
          <div className="history-actions">
            <button
              className="button button-quiet"
              onClick={onForget}
              disabled={!entries.some((entry) => entry.restored)}
            >
              Forget restored
            </button>
            <button className="button button-quiet" onClick={onClose}>
              Close
            </button>
          </div>
        </header>
        <p className="settings-hint">
          Nothing is deleted. Files can be restored to the path they came from.
        </p>
        {error ? <div className="error">{error}</div> : null}
        {entries.length === 0 ? (
          <div className="empty">
            <p className="empty-title">Empty</p>
            <p className="empty-body">Removed and quarantined artifacts appear here.</p>
          </div>
        ) : (
          <ul className="trash-list">
            {entries.map((entry) => (
              <li key={entry.id} data-restored={entry.restored}>
                <div className="trash-main">
                  <span className="trash-name">{entry.artifactName}</span>
                  <span className="badge">{entry.kind}</span>
                  <span className="badge">{entry.ecosystem}</span>
                  <span className="settings-path">{entry.originPath}</span>
                  <span className="settings-hint">
                    {entry.restored ? "restored" : entry.timestamp}
                  </span>
                </div>
                <button
                  className="button"
                  disabled={entry.restored || busyId === entry.id}
                  onClick={() => onRestore(entry.id)}
                >
                  {busyId === entry.id ? "Restoring…" : "Restore"}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
