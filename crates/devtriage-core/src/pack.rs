use crate::model::{AnalysisDepth, EvidenceDraft, Transformation};
use crate::normalize::NormalizedInput;
use std::panic::{AssertUnwindSafe, catch_unwind};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackDescriptor {
    pub id: &'static str,
    pub depth: AnalysisDepth,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackOutput {
    pub evidence: Vec<EvidenceDraft>,
    pub transformations: Vec<Transformation>,
}

#[derive(Debug, Error)]
pub enum PackError {
    #[error("analysis failed: {0}")]
    Analysis(String),
}

pub trait CapabilityPack: Send + Sync {
    fn descriptor(&self) -> PackDescriptor;
    fn detect(&self, input: &NormalizedInput) -> u8;
    fn analyze(&self, input: &NormalizedInput) -> Result<PackOutput, PackError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFailure {
    pub capability_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegistryOutput {
    pub outputs: Vec<(PackDescriptor, PackOutput)>,
    pub failures: Vec<PackFailure>,
}

#[derive(Default)]
pub struct CapabilityRegistry {
    packs: Vec<Box<dyn CapabilityPack>>,
}

impl CapabilityRegistry {
    pub fn register(&mut self, pack: impl CapabilityPack + 'static) {
        self.packs.push(Box::new(pack));
    }

    pub fn run(&self, input: &NormalizedInput) -> RegistryOutput {
        let mut result = RegistryOutput::default();

        for pack in &self.packs {
            if pack.detect(input) == 0 {
                continue;
            }
            let descriptor = pack.descriptor();
            match catch_unwind(AssertUnwindSafe(|| pack.analyze(input))) {
                Ok(Ok(output)) => result.outputs.push((descriptor, output)),
                Ok(Err(error)) => result.failures.push(PackFailure {
                    capability_id: descriptor.id.into(),
                    message: error.to_string(),
                }),
                Err(_) => result.failures.push(PackFailure {
                    capability_id: descriptor.id.into(),
                    message: "capability panicked".into(),
                }),
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct GoodPack;
    struct PanicPack;

    impl CapabilityPack for GoodPack {
        fn descriptor(&self) -> PackDescriptor {
            PackDescriptor {
                id: "good",
                depth: AnalysisDepth::Structured,
            }
        }
        fn detect(&self, _: &NormalizedInput) -> u8 {
            100
        }
        fn analyze(&self, _: &NormalizedInput) -> Result<PackOutput, PackError> {
            Ok(PackOutput::default())
        }
    }

    impl CapabilityPack for PanicPack {
        fn descriptor(&self) -> PackDescriptor {
            PackDescriptor {
                id: "panic",
                depth: AnalysisDepth::Deep,
            }
        }
        fn detect(&self, _: &NormalizedInput) -> u8 {
            100
        }
        fn analyze(&self, _: &NormalizedInput) -> Result<PackOutput, PackError> {
            panic!("broken pack")
        }
    }

    #[test]
    fn keeps_successful_output_when_another_pack_panics() {
        let mut registry = CapabilityRegistry::default();
        registry.register(GoodPack);
        registry.register(PanicPack);
        let input = NormalizedInput {
            text: "error".into(),
            transformations: vec![],
        };
        let output = registry.run(&input);
        assert_eq!(output.outputs.len(), 1);
        assert_eq!(output.failures[0].capability_id, "panic");
    }
}
