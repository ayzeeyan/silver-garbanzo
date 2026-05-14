use crate::error::IrError;
use crate::ir_engine::types::{IrModule, IrOp};
use std::collections::HashSet;
use super::PassStats;

pub fn run(module: &mut IrModule) -> Result<PassStats, IrError> {
    let mut applied = 0;
    for func in &mut module.functions {
        let mut uses = HashSet::new();

        for block in func.blocks.values() {
            for op in &block.ops {
                match op {
                    IrOp::Move(_, s) => { uses.insert(*s); }
                    IrOp::Add(_, a, b) | IrOp::Sub(_, a, b) | IrOp::Mul(_, a, b) | IrOp::Div(_, a, b) | IrOp::Mod(_, a, b) | IrOp::Pow(_, a, b) => {
                        uses.insert(*a); uses.insert(*b);
                    }
                    IrOp::Unm(_, a) | IrOp::Not(_, a) | IrOp::Len(_, a) => {
                        uses.insert(*a);
                    }
                    IrOp::Call(_, f, args) => {
                        uses.insert(*f);
                        uses.extend(args);
                    }
                    IrOp::Return(rets) => {
                        uses.extend(rets);
                    }
                    IrOp::SetGlobal(_, a) => {
                        uses.insert(*a);
                    }
                    IrOp::SetUpvalue(_, a) => {
                        uses.insert(*a);
                    }
                    IrOp::SetTable(a, b, c) => {
                        uses.insert(*a); uses.insert(*b); uses.insert(*c);
                    }
                    IrOp::Eq(_, b, c) | IrOp::Lt(_, b, c) | IrOp::Le(_, b, c) => {
                        uses.insert(*b); uses.insert(*c);
                    }
                    IrOp::Concat(_, args) => {
                        uses.extend(args);
                    }
                    _ => {}
                }
            }
        }

        for block in func.blocks.values_mut() {
            let orig_len = block.ops.len();
            block.ops.retain(|op| {
                match op {
                    IrOp::LoadConst(d, _) | IrOp::Move(d, _) | IrOp::Add(d, _, _) | IrOp::Sub(d, _, _) | IrOp::Mul(d, _, _) | IrOp::Div(d, _, _) => {
                        uses.contains(d)
                    }
                    _ => true // Keep side-effect operations
                }
            });
            applied += orig_len - block.ops.len();
        }
    }
    Ok(PassStats { applied })
}
