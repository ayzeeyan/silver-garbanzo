use std::collections::HashMap;

use crate::error::IrError;
use crate::lua_parser::{FunctionProto, LuaConstant, Opcode};
use super::types::*;

pub fn lift_proto(
    proto: &FunctionProto<'_>,
    module: &mut IrModule,
    id: FunctionId,
) -> Result<(), IrError> {
    // A simplified linear lifter into a single block to get started. CFG breaking comes in cfg.rs.
    let mut block = BasicBlock {
        id: BlockId(0),
        phis: Vec::new(),
        ops: Vec::new(),
        terminator: Terminator::Unreachable,
    };

    let mut r_id = 0;
    let mut alloc_reg = || {
        let v = IrValue(r_id);
        r_id += 1;
        v
    };

    let mut reg_map = HashMap::new();
    let mut get_reg = |idx: u8| -> IrValue {
        *reg_map.entry(idx).or_insert_with(&mut alloc_reg)
    };

    for inst in proto.instructions.iter() {
        match inst.opcode {
            Opcode::Move => {
                block.ops.push(IrOp::Move(get_reg(inst.a), get_reg(inst.b as u8)));
            }
            Opcode::LoadK => {
                let const_val = match &proto.constants[inst.bx as usize] {
                    LuaConstant::Nil => IrConst::Nil,
                    LuaConstant::Boolean(b) => IrConst::Bool(*b),
                    LuaConstant::Number(n) => IrConst::Number(*n),
                    LuaConstant::LuaString(s) => IrConst::String(s.to_vec()),
                };
                block.ops.push(IrOp::LoadConst(get_reg(inst.a), const_val));
            }
            Opcode::Return => {
                // Simplified return representation for Phase 4
                block.terminator = Terminator::Return(vec![get_reg(inst.a)]);
                break;
            }
            _ => {
                // Pass unsupported for now to ensure pipeline builds linearly
                // Actual full expansion is 30+ variants handled the exact same way.
            }
        }
    }

    if let Terminator::Unreachable = block.terminator {
        block.terminator = Terminator::Return(Vec::new());
    }

    let mut blocks = indexmap::IndexMap::new();
    blocks.insert(block.id, block);

    module.functions.push(IrFunction {
        id,
        blocks,
        cfg: ControlFlowGraph::default(),
        params: Vec::new(),
        upvalues: Vec::new(),
        is_vararg: proto.is_vararg != 0,
        max_stack_size: proto.max_stack_size,
    });

    for sub in &proto.protos {
        let sub_id = FunctionId(module.functions.len() as u32);
        lift_proto(sub, module, sub_id)?;
    }

    Ok(())
}
