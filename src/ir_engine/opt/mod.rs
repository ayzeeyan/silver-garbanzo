pub mod copy_prop;
pub mod dce;
pub mod fold;
pub mod opaque;
pub mod phi;
pub mod unreachable;

use crate::error::IrError;
use super::types::IrModule;

#[allow(missing_docs)]
pub struct PassStats {
    pub applied: usize,
}

#[allow(missing_docs)]
pub fn optimize_module(module: &mut IrModule) -> Result<usize, IrError> {
    let mut total_passes = 0;
    for _ in 0..10 {
        let mut total_applied = 0;

        total_applied += fold::run(module)?.applied;
        total_applied += copy_prop::run(module)?.applied;
        total_applied += dce::run(module)?.applied;
        total_applied += opaque::run(module)?.applied;
        total_applied += phi::run(module)?.applied;
        total_applied += unreachable::run(module)?.applied;

        total_passes += 1;

        if total_applied == 0 {
            break;
        }
    }
    Ok(total_passes)
}
