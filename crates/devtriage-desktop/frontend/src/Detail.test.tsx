import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Detail } from "./Detail";
import type { DesktopApi } from "./api";

function fakeApi(): DesktopApi {
  return {
    analyzeInput: vi.fn(),
    confirmProject: vi.fn(),
    chooseDirectory: vi.fn(),
    copyText: vi.fn(),
    showDetail: vi.fn(),
    currentContext: vi.fn().mockResolvedValue({
      analysis_depth: "structured",
      evidence: [
        {
          kind: "error",
          value: "Fatal error",
          provenance: [{ source_id: "clipboard", capability_id: "universal", range: { start: 0, end: 11 } }],
        },
      ],
      transformations: [{ kind: "credential_redacted", detail: "credential", count: 1 }],
      fingerprint: "abc123",
      output: { text: "## Facts\n[REDACTED:CREDENTIAL]", omitted_evidence: 2 },
    }),
  };
}

describe("Detail", () => {
  afterEach(cleanup);

  it("loads the same context and switches output budgets", async () => {
    const api = fakeApi();
    render(<Detail api={api} />);

    expect(await screen.findByText("Evidence")).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Output budget"), { target: { value: "detailed" } });
    expect(api.currentContext).toHaveBeenLastCalledWith("detailed");
  });

  it("shows an empty state when there is no current context", async () => {
    const api = fakeApi();
    vi.mocked(api.currentContext).mockResolvedValue(null);
    render(<Detail api={api} />);

    expect(await screen.findByText("No analyzed context is available.")).toBeTruthy();
  });
});
