import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";
import { createFakeClient } from "./api";
import type { ActionPlan, AppSettings, Artifact, ScanResult } from "./types";

describe("App shell", () => {
  it("renders the four ecosystems and the artifact tabs", async () => {
    render(<App client={createFakeClient()} />);
    expect(screen.getByText("AgentDeck")).toBeInTheDocument();
    expect(screen.getByText("Claude Code")).toBeInTheDocument();
    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.getByText("OpenCode")).toBeInTheDocument();
    expect(screen.getByText("Command Code")).toBeInTheDocument();
    expect(screen.getByText("Agents")).toBeInTheDocument();
    expect(screen.getByText("Skills")).toBeInTheDocument();
    expect(screen.getByText("MCP")).toBeInTheDocument();
    expect(screen.getByText("Other")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText("0 of 4 detected")).toBeInTheDocument());
  });

  it("shows the project badge when the folder is not a git repository", async () => {
    render(
      <App
        client={createFakeClient({
          scan: async () => ({
            artifacts: [],
            detections: [],
            warnings: [],
            projectRoot: "C:\\scratch",
            isGitRepository: false,
            scannedAt: "2026-01-01T00:00:00Z",
          }),
        })}
      />,
    );
    expect(await screen.findByText("not a git repository")).toBeInTheDocument();
  });

  it("persists a theme change and applies it immediately", async () => {
    const saved: AppSettings[] = [];
    const settings: AppSettings = {
      theme: "dark",
      checkForUpdates: true,
      recentProjects: [],
      binaryOverrides: {},
      window: { x: null, y: null, width: 1280, height: 820 },
    };
    render(
      <App
        client={createFakeClient({
          getSettings: async () => settings,
          saveSettings: vi.fn(async (next: AppSettings) => {
            saved.push(next);
            return next;
          }),
        })}
      />,
    );

    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe("dark"),
    );

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    fireEvent.click(screen.getByRole("button", { name: "Light" }));

    await waitFor(() => expect(saved.at(-1)?.theme).toBe("light"));
    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe("light"),
    );
  });

  it("names every affected ecosystem before disabling a shared artifact", async () => {
    const artifact: Artifact = {
      id: "agent:global:reviewer",
      name: "reviewer",
      description: "Reviews code",
      ecosystem: "claude",
      artifactType: "agent",
      scope: "global",
      primaryPath: "C:\\Users\\test\\.claude\\agents\\reviewer.md",
      sourcePaths: ["C:\\Users\\test\\.claude\\agents\\reviewer.md"],
      enabled: true,
      ecosystems: ["claude", "codex"],
      shadowed: [],
      warnings: [],
      managed: true,
      canDisable: true,
      canRemove: true,
      disableMechanism: "permissions.deny",
      transport: null,
      command: null,
      url: null,
      keyPath: null,
      reload: "live",
    };
    const scan: ScanResult = {
      artifacts: [artifact],
      detections: [
        {
          ecosystem: "claude",
          detected: true,
          version: "2.1.215",
          checkedPaths: [],
          capabilities: ["disable-agent"],
          reason: null,
        },
      ],
      warnings: [],
      projectRoot: null,
      isGitRepository: false,
      scannedAt: "2026-01-01T00:00:00Z",
    };
    const plan: ActionPlan = {
      action: "disable",
      artifactId: artifact.id,
      artifactName: artifact.name,
      summary: 'Disable "reviewer" for Claude Code, Codex.',
      affectedEcosystems: ["claude", "codex"],
      files: [
        {
          path: "C:\\Users\\test\\.claude\\settings.json",
          operation: "update",
          description: "Add the Agent deny rule",
        },
      ],
      warnings: [],
    };

    render(
      <App
        client={createFakeClient({
          scan: async () => scan,
          planToggle: async () => plan,
          applyPlan: async (value) => ({
            plan: value,
            scan: {
              ...scan,
              artifacts: [{ ...artifact, enabled: false }],
            },
          }),
          listHistory: async () => [],
        })}
      />,
    );

    const [row] = await screen.findAllByText("reviewer");
    fireEvent.click(row);
    fireEvent.click(await screen.findByRole("button", { name: "Disable" }));
    expect(
      await screen.findByText("Affected ecosystems: claude, codex"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    await waitFor(() =>
      expect(screen.getAllByText("disabled").length).toBeGreaterThan(0),
    );
  });
});
