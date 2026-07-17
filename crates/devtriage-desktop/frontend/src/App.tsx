import { useState } from "react";
import { desktopApi, type AnalysisResponse, type DesktopApi } from "./api";

export type { DesktopApi } from "./api";

interface AppProps {
  api?: DesktopApi;
}

const candidateReason: Record<AnalysisResponse["candidates"][number]["reason"], string> = {
  absolute_path: "Path found in the supplied log",
  recent_relative_path: "Recent project matching a relative path",
  recent_project: "Previously confirmed project",
};

export function App({ api = desktopApi }: AppProps) {
  const [input, setInput] = useState("");
  const [response, setResponse] = useState<AnalysisResponse | null>(null);
  const [selectedCandidate, setSelectedCandidate] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);

  const analyze = async () => {
    setError(null);
    setIsAnalyzing(true);
    try {
      const nextResponse = await api.analyzeInput(input, "standard");
      setResponse(nextResponse);
      setSelectedCandidate(nextResponse.candidates[0]?.path ?? null);
    } catch {
      setResponse(null);
      setSelectedCandidate(null);
      setError("Unable to analyze input.");
    } finally {
      setIsAnalyzing(false);
    }
  };

  const runCommand = async (action: () => Promise<void>, failureMessage: string) => {
    setError(null);
    try {
      await action();
    } catch {
      setError(failureMessage);
    }
  };

  const chooseFolder = () =>
    runCommand(async () => {
      const path = await api.chooseDirectory();
      if (path !== null) {
        await api.confirmProject(path);
      }
    }, "Unable to confirm the selected folder.");

  const firstError = response?.context.evidence.find((evidence) => evidence.kind === "error")?.value;

  return (
    <main className="quick-panel">
      <h1>DevTriage</h1>
      <label htmlFor="log-input">Log input</label>
      <textarea
        id="log-input"
        value={input}
        onChange={(event) => setInput(event.target.value)}
        placeholder="Paste a log or error message"
        rows={10}
      />
      <button type="button" onClick={analyze} disabled={isAnalyzing}>
        {isAnalyzing ? "Analyzing…" : "Analyze"}
      </button>

      {error && <p role="alert">{error}</p>}

      {response && (
        <section aria-label="Analysis result">
          <p>Analysis depth: {response.context.analysis_depth}</p>
          {firstError && <p>First error: {firstError}</p>}
          <pre aria-label="AI context">{response.context.output.text}</pre>

          <fieldset>
            <legend>Project candidates</legend>
            {response.candidates.length === 0 ? (
              <p>No project candidates found.</p>
            ) : (
              response.candidates.map((candidate) => (
                <label className="candidate" key={candidate.path}>
                  <input
                    type="radio"
                    name="project-candidate"
                    value={candidate.path}
                    checked={selectedCandidate === candidate.path}
                    onChange={() => setSelectedCandidate(candidate.path)}
                  />
                  <span>{candidate.path}</span>
                  <small>{candidateReason[candidate.reason]}</small>
                </label>
              ))
            )}
          </fieldset>

          <button
            type="button"
            disabled={selectedCandidate === null}
            onClick={() => selectedCandidate && runCommand(() => api.confirmProject(selectedCandidate), "Unable to confirm the selected project.")}
          >
            Confirm project
          </button>
        </section>
      )}

      <div className="actions">
        <button type="button" onClick={chooseFolder}>Choose folder</button>
        <button
          type="button"
          disabled={response === null}
          onClick={() => response && runCommand(() => api.copyText(response.context.output.text), "Unable to copy AI context.")}
        >
          Copy AI context
        </button>
        <button
          type="button"
          disabled={response === null}
          onClick={() => runCommand(api.showDetail, "Unable to open detail view.")}
        >
          Open detail
        </button>
      </div>
    </main>
  );
}
