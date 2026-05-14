use crate::error::DeobfuscationError;
use crate::ir_engine::IrModule;
use super::registry::{DetectionContext, ObfuscatorProfile};

#[allow(missing_docs)]
pub struct MoonSecProfile;

impl ObfuscatorProfile for MoonSecProfile {
    fn name(&self) -> &'static str {
        "MoonSec"
    }

    fn detect(&self, ctx: &DetectionContext<'_>) -> f64 {
        let mut score: f64 = 0.0;

        if let Some(src) = ctx.raw_source {
            // Typical Moonsec strings
            if src.contains("MoonSec") || src.contains("moonsec") {
                score += 0.5;
            }
        }

        score.clamp(0.0, 1.0)
    }

    fn pre_decompile_pass(&self, _ir: &mut IrModule) -> Result<(), DeobfuscationError> {
        Ok(())
    }

    fn post_decompile_pass(&self, _source: &mut String) -> Result<(), DeobfuscationError> {
        Ok(())
    }
}
