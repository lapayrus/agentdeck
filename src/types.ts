export type Ecosystem = "claude" | "codex" | "opencode" | "commandcode";

export type ArtifactType =
  | "agent"
  | "skill"
  | "mcp"
  | "hook"
  | "command"
  | "plugin"
  | "memory"
  | "settings";

export type Scope = "global" | "project" | "projectLocal";

export type ReloadHint =
  | "live"
  | "nextTurn"
  | "nextSession"
  | "restart"
  | "unknown";

export interface ShadowedArtifact {
  name: string;
  path: string;
  scope: Scope;
  ecosystem: Ecosystem;
  reason: string;
}

export interface Artifact {
  id: string;
  name: string;
  description: string | null;
  ecosystem: Ecosystem;
  artifactType: ArtifactType;
  scope: Scope;
  primaryPath: string;
  sourcePaths: string[];
  enabled: boolean | null;
  ecosystems: Ecosystem[];
  shadowed: ShadowedArtifact[];
  warnings: string[];
  managed: boolean;
  canDisable: boolean;
  canRemove: boolean;
  disableMechanism: string | null;
  transport: string | null;
  command: string | null;
  url: string | null;
  keyPath: string | null;
  reload: ReloadHint;
}

export interface EcosystemDetection {
  ecosystem: Ecosystem;
  detected: boolean;
  version: string | null;
  checkedPaths: string[];
  capabilities: string[];
  reason: string | null;
}

export interface ScanResult {
  artifacts: Artifact[];
  detections: EcosystemDetection[];
  warnings: string[];
  projectRoot: string | null;
  isGitRepository: boolean;
  scannedAt: string;
}

export interface ScanRequest {
  projectRoot?: string | null;
}

export interface PlanFile {
  path: string;
  operation: "create" | "update" | "move" | "delete";
  description: string;
}

export interface ActionPlan {
  action: "disable" | "enable" | "remove" | "restore";
  artifactId: string;
  artifactName: string;
  summary: string;
  affectedEcosystems: Ecosystem[];
  files: PlanFile[];
  warnings: string[];
}

export interface ActionResult {
  plan: ActionPlan;
  scan: ScanResult;
}

export interface HistoryEntry {
  id: string;
  path: string;
  action: string;
  timestamp: string;
  ecosystem: Ecosystem | null;
  artifactName: string | null;
  snapshotPath: string;
}

export interface TrashEntry {
  id: string;
  artifactName: string;
  artifactType: ArtifactType;
  ecosystem: Ecosystem;
  scope: Scope;
  originPath: string;
  trashedPath: string;
  timestamp: string;
  kind: "file" | "quarantine" | "configKey";
  restored: boolean;
}

export interface AppSettings {
  theme: "light" | "dark" | "system";
  checkForUpdates: boolean;
  recentProjects: string[];
  binaryOverrides: Partial<Record<Ecosystem, string>>;
  window: { x: number | null; y: number | null; width: number; height: number };
}

export interface SnapshotDiff {
  path: string;
  snapshot: string;
  current: string;
  diff: string;
}
