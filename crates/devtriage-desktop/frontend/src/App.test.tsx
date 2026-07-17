import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { afterEach } from "vitest";
import { App, type DesktopApi } from "./App";

function fakeApi(overrides: Partial<DesktopApi> = {}): DesktopApi {
  return {
    analyzeInput: vi.fn().mockResolvedValue({
      context: {
        analysis_depth: "structured",
        evidence: [{ kind: "error", value: "Fatal error" }],
        output: { text: "## Facts\n[REDACTED:CREDENTIAL]" },
      },
      candidates: [
        { path: "/work/app", score: 300, reason: "absolute_path" },
      ],
    }),
    confirmProject: vi.fn().mockResolvedValue(undefined),
    chooseDirectory: vi.fn().mockResolvedValue(null),
    copyText: vi.fn().mockResolvedValue(undefined),
    showDetail: vi.fn().mockResolvedValue(undefined),
    currentContext: vi.fn().mockResolvedValue(null),
    ...overrides,
  };
}

describe("App", () => {
  afterEach(cleanup);

  it("shows redacted summary and copies only on click", async () => {
    const api = fakeApi({
      analyzeInput: vi.fn().mockResolvedValue({
        context: {
          analysis_depth: "structured",
          evidence: [{ kind: "error", value: "Fatal error" }],
          output: { text: "## Facts\n[REDACTED:CREDENTIAL]" },
        },
        candidates: [],
      }),
    });
    render(<App api={api} />);

    fireEvent.change(screen.getByLabelText("Log input"), { target: { value: "fatal token=secret" } });
    fireEvent.click(screen.getByRole("button", { name: "Analyze" }));

    expect((await screen.findByLabelText("AI context")).textContent).toContain("[REDACTED:CREDENTIAL]");
    expect(api.copyText).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Copy AI context" }));
    expect(api.copyText).toHaveBeenCalledWith(expect.not.stringContaining("secret"));
  });

  it("confirms the selected candidate and manually chosen folder", async () => {
    const api = fakeApi({ chooseDirectory: vi.fn().mockResolvedValue("/work/manual") });
    render(<App api={api} />);

    fireEvent.click(screen.getByRole("button", { name: "Analyze" }));
    await screen.findByRole("radio", { name: /\/work\/app/ });
    fireEvent.click(screen.getByRole("button", { name: "Confirm project" }));
    fireEvent.click(screen.getByRole("button", { name: "Choose folder" }));

    await waitFor(() => expect(api.confirmProject).toHaveBeenNthCalledWith(1, "/work/app"));
    await waitFor(() => expect(api.confirmProject).toHaveBeenNthCalledWith(2, "/work/manual"));
  });

  it("shows a safe command failure without echoing raw input", async () => {
    const api = fakeApi({ analyzeInput: vi.fn().mockRejectedValue(new Error("backend unavailable")) });
    render(<App api={api} />);

    fireEvent.change(screen.getByLabelText("Log input"), { target: { value: "token=secret" } });
    fireEvent.click(screen.getByRole("button", { name: "Analyze" }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toBe("Unable to analyze input.");
    expect(alert.textContent).not.toContain("token=secret");
  });
});
