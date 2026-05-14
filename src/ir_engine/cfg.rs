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
                successors.get_mut(id).unwrap().push(*target);
                predecessors.get_mut(target).unwrap().push(*id);
            }
            Terminator::CondBranch(_, true_target, false_target) => {
                successors.get_mut(id).unwrap().push(*true_target);
                predecessors.get_mut(true_target).unwrap().push(*id);

                successors.get_mut(id).unwrap().push(*false_target);
                predecessors.get_mut(false_target).unwrap().push(*id);
            }
            Terminator::Return(_) | Terminator::TailCall(_, _) | Terminator::Unreachable => {}
        }
    }

    func.cfg.successors = successors;
    func.cfg.predecessors = predecessors;

    Ok(())
}
