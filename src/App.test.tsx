import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";
import { createFakeClient } from "./api";

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
});
