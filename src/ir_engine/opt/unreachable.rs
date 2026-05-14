use crate::error::IrError;
use crate::ir_engine::types::IrModule;
use super::PassStats;

pub fn run(_module: &mut IrModule) -> Result<PassStats, IrError> {
    Ok(PassStats { applied: 0 })
}
