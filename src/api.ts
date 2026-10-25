import { invoke } from "@tauri-apps/api/core";
import type {
  ActionResult,
  ActionPlan,
  AppSettings,
  Artifact,
  HistoryEntry,
  ScanRequest,
  ScanResult,
  SnapshotDiff,
  TrashEntry,
} from "./types";

export interface CommandClient {
  scan(request: ScanRequest): Promise<ScanResult>;
  planToggle(artifactId: string, enabled: boolean): Promise<ActionPlan>;
  planRemove(artifactId: string): Promise<ActionPlan>;
  applyPlan(plan: ActionPlan): Promise<ActionResult>;
  restore(trashId: string): Promise<ActionResult>;
  listTrash(): Promise<TrashEntry[]>;
  listHistory(path?: string): Promise<HistoryEntry[]>;
  readSnapshot(snapshotPath: string): Promise<string>;
  diffSnapshot(snapshotPath: string): Promise<SnapshotDiff>;
  rollback(snapshotPath: string): Promise<ActionResult>;
  pruneSnapshots(ids: string[]): Promise<number>;
  pruneTrash(ids: string[]): Promise<number>;
  getSettings(): Promise<AppSettings>;
  saveSettings(settings: AppSettings): Promise<AppSettings>;
  setProject(projectRoot: string | null): Promise<ScanResult>;
  readFile(path: string): Promise<string>;
  historyForArtifact(artifactId: string): Promise<HistoryEntry[]>;
}

export const tauriClient: CommandClient = {
  scan: (request) => invoke<ScanResult>("scan", { request }),
  planToggle: (artifactId, enabled) =>
    invoke<ActionPlan>("plan_toggle", { artifactId, enabled }),
  planRemove: (artifactId) => invoke<ActionPlan>("plan_remove", { artifactId }),
  applyPlan: (plan) => invoke<ActionResult>("apply_plan", { plan }),
  restore: (trashId) => invoke<ActionResult>("restore", { trashId }),
  listTrash: () => invoke<TrashEntry[]>("list_trash"),
  listHistory: (path) => invoke<HistoryEntry[]>("list_history", { path }),
  readSnapshot: (snapshotPath) =>
    invoke<string>("read_snapshot", { snapshotPath }),
  diffSnapshot: (snapshotPath) =>
    invoke<SnapshotDiff>("diff_snapshot", { snapshotPath }),
  rollback: (snapshotPath) =>
    invoke<ActionResult>("rollback", { snapshotPath }),
  pruneSnapshots: (ids) => invoke<number>("prune_snapshots", { ids }),
  pruneTrash: (ids) => invoke<number>("prune_trash", { ids }),
  getSettings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings) =>
    invoke<AppSettings>("save_settings", { settings }),
  setProject: (projectRoot) =>
    invoke<ScanResult>("set_project", { projectRoot }),
  readFile: (path) => invoke<string>("read_file", { path }),
  historyForArtifact: (artifactId) =>
    invoke<HistoryEntry[]>("history_for_artifact", { artifactId }),
};

export function createFakeClient(
  overrides: Partial<CommandClient> = {},
): CommandClient {
  const emptyScan: ScanResult = {
    artifacts: [],
    detections: [],
    warnings: [],
    projectRoot: null,
    isGitRepository: false,
    scannedAt: "2026-01-01T00:00:00Z",
  };
  const base: CommandClient = {
    scan: async () => emptyScan,
    planToggle: async () => {
      throw new Error("not implemented");
    },
    planRemove: async () => {
      throw new Error("not implemented");
    },
    applyPlan: async () => {
      throw new Error("not implemented");
    },
    restore: async () => {
      throw new Error("not implemented");
    },
    listTrash: async () => [],
    listHistory: async () => [],
    readSnapshot: async () => "",
    diffSnapshot: async () => ({
      path: "",
      snapshot: "",
      current: "",
      diff: "",
    }),
    rollback: async () => {
      throw new Error("not implemented");
    },
    pruneSnapshots: async () => 0,
    pruneTrash: async () => 0,
    getSettings: async () => ({
      theme: "system",
      checkForUpdates: true,
      recentProjects: [],
      binaryOverrides: {},
      window: { x: null, y: null, width: 1280, height: 820 },
    }),
    saveSettings: async (settings) => settings,
    setProject: async () => emptyScan,
    readFile: async () => "",
    historyForArtifact: async () => [],
  };
  return { ...base, ...overrides };
}

export function artifactById(
  artifacts: Artifact[],
  id: string | null,
): Artifact | null {
  if (!id) return null;
  return artifacts.find((artifact) => artifact.id === id) ?? null;
}
