import type { ActionPlan } from "../types";

export default function ActionDialog({
  plan,
  busy,
  error,
  onConfirm,
  onCancel,
}: {
  plan: ActionPlan;
  busy: boolean;
  error: string | null;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <div className="modal-backdrop" onClick={busy ? undefined : onCancel}>
      <div
        className="modal"
        role="dialog"
        aria-label="Confirm action"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal-header">
          <h2>{plan.action === "remove" ? "Remove artifact" : "Change state"}</h2>
          <button className="button button-quiet" onClick={onCancel} disabled={busy}>
            Cancel
          </button>
        </header>

        <p>{plan.summary}</p>
        <p className="settings-hint">
          Affected ecosystems: {plan.affectedEcosystems.join(", ")}
        </p>

        {plan.warnings.map((warning) => (
          <p className="warning" key={warning}>
            {warning}
          </p>
        ))}
        {error ? <div className="error">{error}</div> : null}

        <section className="settings-section">
          <h3>Files</h3>
          <ul className="plan-files">
            {plan.files.map((file) => (
              <li key={`${file.path}-${file.operation}`}>
                <span className="badge">{file.operation}</span>
                <span className="settings-path">{file.path}</span>
                <span className="plan-description">{file.description}</span>
              </li>
            ))}
          </ul>
        </section>

        <footer className="modal-footer">
          <button className="button button-quiet" onClick={onCancel} disabled={busy}>
            Cancel
          </button>
          <button
            className={plan.action === "remove" ? "button button-danger" : "button button-primary"}
            onClick={onConfirm}
            disabled={busy}
          >
            {busy ? "Working…" : plan.action === "remove" ? "Remove" : "Apply"}
          </button>
        </footer>
      </div>
    </div>
  );
}
