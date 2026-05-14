#[allow(missing_docs)]
pub mod copy_prop;
#[allow(missing_docs)]
pub mod dce;
#[allow(missing_docs)]
pub mod fold;
#[allow(missing_docs)]
pub mod opaque;
#[allow(missing_docs)]
pub mod phi;
#[allow(missing_docs)]
pub mod unreachable;

use crate::error::IrError;
use super::types::IrModule;

pub struct PassStats {
    pub applied: usize,
}

pub fn optimize_module(module: &mut IrModule) -> Result<(), IrError> {
    for _ in 0..10 {
        let mut total_applied = 0;

        total_applied += fold::run(module)?.applied;
        total_applied += copy_prop::run(module)?.applied;
        total_applied += dce::run(module)?.applied;
        total_applied += opaque::run(module)?.applied;
        total_applied += phi::run(module)?.applied;
        total_applied += unreachable::run(module)?.applied;

        if total_applied == 0 {
            break;
        }
    }
    Ok(())
}
