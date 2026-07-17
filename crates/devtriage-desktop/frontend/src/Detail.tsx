import { useEffect, useState } from "react";
import {
  desktopApi,
  type DesktopApi,
  type IssueContext,
  type OutputBudget,
} from "./api";

interface DetailProps {
  api?: DesktopApi;
}

const budgets: Array<{ value: OutputBudget; label: string }> = [
  { value: "compact", label: "Compact" },
  { value: "standard", label: "Standard" },
  { value: "detailed", label: "Detailed" },
];

export function Detail({ api = desktopApi }: DetailProps) {
  const [budget, setBudget] = useState<OutputBudget>("standard");
  const [context, setContext] = useState<IssueContext | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let current = true;
    setIsLoading(true);
    setError(null);
    api.currentContext(budget)
      .then((nextContext) => {
        if (current) setContext(nextContext);
      })
      .catch(() => {
        if (current) {
          setContext(null);
          setError("Unable to load the current context.");
        }
      })
      .finally(() => {
        if (current) setIsLoading(false);
      });

    return () => {
      current = false;
    };
  }, [api, budget]);

  return (
    <main className="detail-view">
      <h1>Analysis detail</h1>
      <label htmlFor="output-budget">Output budget</label>
      <select
        id="output-budget"
        value={budget}
        onChange={(event) => setBudget(event.target.value as OutputBudget)}
      >
        {budgets.map(({ value, label }) => <option key={value} value={value}>{label}</option>)}
      </select>

      {isLoading && <p>Loading context…</p>}
      {error && <p role="alert">{error}</p>}
      {!isLoading && !error && context === null && <p>No analyzed context is available.</p>}

      {context && (
        <>
          <section aria-labelledby="evidence-heading">
            <h2 id="evidence-heading">Evidence</h2>
            {context.evidence.length === 0 ? <p>No evidence was produced.</p> : (
              <ul>
                {context.evidence.map((evidence, index) => (
                  <li key={`${evidence.kind}-${index}`}>
                    <strong>{evidence.kind}</strong>: {evidence.value}
                    {evidence.provenance && evidence.provenance.length > 0 && (
                      <ul aria-label={`Provenance for ${evidence.kind}`}>
                        {evidence.provenance.map((provenance, provenanceIndex) => (
                          <li key={`${provenance.source_id}-${provenanceIndex}`}>
                            {provenance.source_id} via {provenance.capability_id}
                            {provenance.range && ` (${provenance.range.start}–${provenance.range.end})`}
                          </li>
                        ))}
                      </ul>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </section>

          <section aria-labelledby="transformations-heading">
            <h2 id="transformations-heading">Transformations</h2>
            {context.transformations?.length ? (
              <ul>{context.transformations.map((item, index) => <li key={`${item.kind}-${index}`}>{item.kind}: {item.detail} ({item.count})</li>)}</ul>
            ) : <p>No transformations were applied.</p>}
          </section>

          <section aria-labelledby="fingerprint-heading">
            <h2 id="fingerprint-heading">Fingerprint</h2>
            <code>{context.fingerprint ?? "Unavailable"}</code>
          </section>

          <section aria-labelledby="output-heading">
            <h2 id="output-heading">Output preview</h2>
            <pre>{context.output.text}</pre>
            <p>Omitted evidence: {context.output.omitted_evidence ?? 0}</p>
          </section>
        </>
      )}
    </main>
  );
}
