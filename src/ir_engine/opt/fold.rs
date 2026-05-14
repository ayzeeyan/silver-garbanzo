use crate::error::IrError;
use crate::ir_engine::types::{IrConst, IrModule, IrOp, IrValue};
use std::collections::HashMap;
use super::PassStats;

pub fn run(module: &mut IrModule) -> Result<PassStats, IrError> {
    let mut applied = 0;
    for func in &mut module.functions {
        let mut consts: HashMap<IrValue, IrConst> = HashMap::new();

        for block in func.blocks.values() {
            for op in &block.ops {
                if let IrOp::LoadConst(v, c) = op {
                    consts.insert(*v, c.clone());
                }
            }
        }

        for block in func.blocks.values_mut() {
            for op in &mut block.ops {
                let mut new_op = None;
                match op {
                    IrOp::Add(d, a, b) => {
                        if let (Some(IrConst::Number(na)), Some(IrConst::Number(nb))) = (consts.get(a), consts.get(b)) {
                            new_op = Some(IrOp::LoadConst(*d, IrConst::Number(*na + *nb)));
                        }
                    }
                    IrOp::Sub(d, a, b) => {
                        if let (Some(IrConst::Number(na)), Some(IrConst::Number(nb))) = (consts.get(a), consts.get(b)) {
                            new_op = Some(IrOp::LoadConst(*d, IrConst::Number(*na - *nb)));
                        }
                    }
                    IrOp::Mul(d, a, b) => {
                        if let (Some(IrConst::Number(na)), Some(IrConst::Number(nb))) = (consts.get(a), consts.get(b)) {
                            new_op = Some(IrOp::LoadConst(*d, IrConst::Number(*na * *nb)));
                        }
                    }
                    IrOp::Div(d, a, b) => {
                        if let (Some(IrConst::Number(na)), Some(IrConst::Number(nb))) = (consts.get(a), consts.get(b)) {
                            if *nb != 0.0 {
                                new_op = Some(IrOp::LoadConst(*d, IrConst::Number(*na / *nb)));
                            }
                        }
                    }
                    _ => {}
                }
                if let Some(nop) = new_op {
                    *op = nop;
                    applied += 1;
                }
            }
        }
    }
    Ok(PassStats { applied })
}
