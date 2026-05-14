use indexmap::IndexMap;
use crate::error::IrError;
use super::types::*;

pub fn build_cfg(func: &mut IrFunction) -> Result<(), IrError> {
    let mut successors: IndexMap<BlockId, Vec<BlockId>> = IndexMap::new();
    let mut predecessors: IndexMap<BlockId, Vec<BlockId>> = IndexMap::new();

    // Initialize all blocks in map
    for &id in func.blocks.keys() {
        successors.insert(id, Vec::new());
        predecessors.insert(id, Vec::new());
    }

    // Connect edges based on terminators
    for (id, block) in &func.blocks {
        match &block.terminator {
            Terminator::Branch(target) => {
                successors.get_mut(id).expect("Invariant violation").push(*target);
                if let Some(preds) = predecessors.get_mut(target) {
                    preds.push(*id);
                }
            }
            Terminator::CondBranch(_, true_target, false_target) => {
                successors.get_mut(id).expect("Invariant violation").push(*true_target);
                if let Some(preds) = predecessors.get_mut(true_target) {
                    preds.push(*id);
                }

                successors.get_mut(id).expect("Invariant violation").push(*false_target);
                if let Some(preds) = predecessors.get_mut(false_target) {
                    preds.push(*id);
                }
            }
            Terminator::Return(_) | Terminator::TailCall(_, _) | Terminator::Unreachable => {}
        }
    }

    func.cfg.successors = successors;
    func.cfg.predecessors = predecessors;

    Ok(())
}
