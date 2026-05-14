use crate::error::EmuError;
use crate::ir_engine::IrModule;

#[allow(missing_docs)]
pub struct StringResolver;

#[allow(missing_docs)]
pub fn resolve_all(_module: &mut IrModule) -> Result<(), EmuError> {
    Ok(())
}
