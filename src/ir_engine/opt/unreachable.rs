use crate::error::IrError;
use crate::ir_engine::types::{IrModule, BlockId};
use std::collections::HashSet;
use super::PassStats;

pub fn run(module: &mut IrModule) -> Result<PassStats, IrError> {
    let mut applied = 0;
    for func in &mut module.functions {
        let mut visited = HashSet::new();
        let mut stack = vec![BlockId(0)];

        while let Some(node) = stack.pop() {
            if visited.insert(node) {
                if let Some(succs) = func.cfg.successors.get(&node) {
                    stack.extend(succs);
                }
            }
        }

        let to_remove: Vec<BlockId> = func.blocks.keys().filter(|&k| !visited.contains(k)).copied().collect();
        for k in to_remove {
            func.blocks.shift_remove(&k);
            applied += 1;
        }
    }
    Ok(PassStats { applied })
}
