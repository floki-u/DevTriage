use crate::compiler::{OutputBudget, compile};
use crate::fingerprint::fingerprint;
use crate::graph::merge;
use crate::model::{AnalysisDepth, IssueContext, SCHEMA_VERSION, Transformation};
use crate::normalize::normalize;
use crate::pack::CapabilityRegistry;
use crate::redact::redact_evidence;
use crate::universal::UniversalPack;

pub struct Pipeline {
    registry: CapabilityRegistry,
}

impl Default for Pipeline {
    fn default() -> Self {
        let mut registry = CapabilityRegistry::default();
        registry.register(UniversalPack);
        Self { registry }
    }
}

impl Pipeline {
    pub fn analyze(&self, raw: &str, budget: OutputBudget) -> IssueContext {
        let normalized = normalize(raw);
        let registry_output = self.registry.run(&normalized);
        let depth = registry_output
            .outputs
            .iter()
            .map(|(descriptor, _)| descriptor.depth)
            .max()
            .unwrap_or(AnalysisDepth::Generic);

        let mut transformations = normalized.transformations;
        let mut drafts = Vec::new();
        for (_, output) in registry_output.outputs {
            drafts.extend(output.evidence);
            transformations.extend(output.transformations);
        }
        transformations.extend(registry_output.failures.into_iter().map(|failure| {
            Transformation {
                kind: "capability_failed".into(),
                detail: format!("{}: {}", failure.capability_id, failure.message),
                count: 1,
            }
        }));

        let mut evidence = merge(drafts);
        transformations.extend(redact_evidence(&mut evidence));
        let fingerprint = fingerprint(&evidence);
        let output = compile(&evidence, &transformations, budget);

        IssueContext {
            schema_version: SCHEMA_VERSION,
            analysis_depth: depth,
            evidence,
            transformations,
            fingerprint,
            output,
        }
    }
}
