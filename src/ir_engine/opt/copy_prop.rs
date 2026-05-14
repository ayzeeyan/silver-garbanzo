use crate::error::IrError;
use crate::ir_engine::types::{IrModule, IrOp, IrValue};
use std::collections::HashMap;
use super::PassStats;

pub fn run(module: &mut IrModule) -> Result<PassStats, IrError> {
    let mut applied = 0;
    for func in &mut module.functions {
        let mut copies: HashMap<IrValue, IrValue> = HashMap::new();

        for block in func.blocks.values() {
            for op in &block.ops {
                if let IrOp::Move(d, s) = op {
                    copies.insert(*d, *s);
                }
            }
        }

        for block in func.blocks.values_mut() {
            for op in &mut block.ops {
                let mut replaced = false;
                match op {
                    IrOp::Move(_, s) => { if let Some(n) = copies.get(s) { *s = *n; replaced = true; } }
                    IrOp::Add(_, a, b) | IrOp::Sub(_, a, b) | IrOp::Mul(_, a, b) | IrOp::Div(_, a, b) | IrOp::Mod(_, a, b) | IrOp::Pow(_, a, b) => {
                        if let Some(n) = copies.get(a) { *a = *n; replaced = true; }
                        if let Some(n) = copies.get(b) { *b = *n; replaced = true; }
                    }
                    IrOp::Unm(_, a) | IrOp::Not(_, a) | IrOp::Len(_, a) => {
                        if let Some(n) = copies.get(a) { *a = *n; replaced = true; }
                    }
                    _ => {}
                }
                if replaced {
                    applied += 1;
                }
            }
        }
    }
    Ok(PassStats { applied })
}
