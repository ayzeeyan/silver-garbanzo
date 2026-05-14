use crate::error::DeobfuscationError;
use crate::ir_engine::IrModule;
use super::registry::{DetectionContext, ObfuscatorProfile};

#[allow(missing_docs)]
pub struct GenericProfile;

impl ObfuscatorProfile for GenericProfile {
    fn name(&self) -> &'static str {
        "Generic"
    }

    fn detect(&self, _ctx: &DetectionContext<'_>) -> f64 {
        0.1 // Lowest priority sentinel fallback
    }

    fn pre_decompile_pass(&self, _ir: &mut IrModule) -> Result<(), DeobfuscationError> {
        Ok(()) // Pass-through
    }

    fn post_decompile_pass(&self, _source: &mut String) -> Result<(), DeobfuscationError> {
        Ok(()) // Pass-through
    }
}
