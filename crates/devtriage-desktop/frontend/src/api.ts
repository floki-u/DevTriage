import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { open } from "@tauri-apps/plugin-dialog";

export type OutputBudget = "compact" | "standard" | "detailed";
export type AnalysisDepth = "generic" | "structured" | "deep";

export interface AnalysisResponse {
  context: {
    analysis_depth: AnalysisDepth;
    evidence: Array<{ kind: string; value: string }>;
    output: { text: string };
  };
  candidates: ProjectCandidate[];
}

export interface ProjectCandidate {
  path: string;
  score: number;
  reason: "absolute_path" | "recent_relative_path" | "recent_project";
}

export interface DesktopApi {
  analyzeInput(input: string, budget: OutputBudget): Promise<AnalysisResponse>;
  confirmProject(path: string): Promise<void>;
  chooseDirectory(): Promise<string | null>;
  copyText(text: string): Promise<void>;
  showDetail(): Promise<void>;
}

export const desktopApi: DesktopApi = {
  analyzeInput: (input, budget) => invoke<AnalysisResponse>("analyze_input", { input, budget }),
  confirmProject: (path) => invoke<void>("confirm_project", { path }),
  chooseDirectory: async () => {
    const selected = await open({ directory: true, multiple: false });
    return typeof selected === "string" ? selected : null;
  },
  copyText: writeText,
  showDetail: () => invoke<void>("show_detail_window"),
};
