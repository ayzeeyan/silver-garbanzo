use crate::error::IrError;
use crate::ir_engine::types::IrModule;
use super::PassStats;

pub fn run(module: &mut IrModule) -> Result<PassStats, IrError> {
    let mut applied = 0;
    for func in &mut module.functions {
        for block in func.blocks.values_mut() {
            let orig_len = block.phis.len();
            block.phis.retain(|phi| {
                if phi.incoming.is_empty() {
                    return false;
                }
                let first = phi.incoming[0].1;
                !phi.incoming.iter().all(|(_, v)| *v == first)
            });
            applied += orig_len - block.phis.len();
        }
    }
    Ok(PassStats { applied })
}
