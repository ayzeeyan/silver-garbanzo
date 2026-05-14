use crate::error::IrError;
use crate::ir_engine::types::{IrConst, IrModule, IrOp, Terminator};
use super::PassStats;

pub fn run(module: &mut IrModule) -> Result<PassStats, IrError> {
    let mut applied = 0;
    for func in &mut module.functions {
        for block in func.blocks.values_mut() {
            let mut resolved = None;
            if let Terminator::CondBranch(c, tb, fb) = &block.terminator {
                for op in &block.ops {
                    if let IrOp::LoadConst(d, IrConst::Bool(val)) = op {
                        if d == c {
                            resolved = Some(if *val { *tb } else { *fb });
                        }
                    }
                }
            }
            if let Some(target) = resolved {
                block.terminator = Terminator::Branch(target);
                applied += 1;
            }
        }
    }
    Ok(PassStats { applied })
}
