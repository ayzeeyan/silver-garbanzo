use std::collections::{HashMap, HashSet};

use crate::error::IrError;
use crate::lua_parser::{FunctionProto, LuaConstant, Opcode};
use super::types::*;

pub fn lift_proto(
    proto: &FunctionProto<'_>,
    module: &mut IrModule,
    id: FunctionId,
) -> Result<(), IrError> {
    let mut leaders = HashSet::new();
    leaders.insert(0); // first instruction

    for (pc, inst) in proto.instructions.iter().enumerate() {
        if inst.opcode.has_jump() {
            if inst.opcode == Opcode::Jmp || inst.opcode == Opcode::ForLoop || inst.opcode == Opcode::ForPrep {
                let target = (pc as i32 + 1 + inst.sbx) as usize;
                leaders.insert(target);
                leaders.insert(pc + 1);
            } else {
                leaders.insert(pc + 1);
                leaders.insert(pc + 2); // target of conditional jump implicitly
            }
        }
        if inst.opcode == Opcode::Return || inst.opcode == Opcode::TailCall {
            leaders.insert(pc + 1);
        }
    }

    let mut leader_list: Vec<usize> = leaders.into_iter().filter(|&x| x < proto.instructions.len()).collect();
    leader_list.sort_unstable();

    let mut pc_to_block = HashMap::new();
    let mut r_id = 0;

    let mut b_id = 0;
    let mut alloc_block = || {
        let bid = BlockId(b_id);
        b_id += 1;
        bid
    };

    let mut blocks_map = indexmap::IndexMap::new();

    for &pc in &leader_list {
        let bid = alloc_block();
        pc_to_block.insert(pc, bid);
        blocks_map.insert(bid, BasicBlock {
            id: bid,
            phis: Vec::new(),
            ops: Vec::new(),
            terminator: Terminator::Unreachable,
        });
    }

    let mut reg_map = HashMap::new();

    let mut current_block_id = BlockId(0);
    let mut i = 0;

    // Process instructions
    while i < proto.instructions.len() {
        if let Some(&bid) = pc_to_block.get(&i) {
            current_block_id = bid;
        }

        let inst = proto.instructions[i];
        let mut ops = Vec::new();
        let mut terminator = None;

        // Inlined to avoid closure borrow issues
        macro_rules! get_reg {
            ($idx:expr) => {{
                if let Some(&v) = reg_map.get(&$idx) {
                    v
                } else {
                    let v = IrValue(r_id);
                    r_id += 1;
                    reg_map.insert($idx, v);
                    v
                }
            }};
        }

        macro_rules! alloc_reg {
            () => {{
                let v = IrValue(r_id);
                r_id += 1;
                v
            }};
        }

        macro_rules! resolve_rk {
            ($rk:expr, $ops:expr) => {{
                if $rk >= 256 {
                    let const_idx = $rk - 256;
                    let const_val = match &proto.constants[const_idx as usize] {
                        LuaConstant::Nil => IrConst::Nil,
                        LuaConstant::Boolean(b) => IrConst::Bool(*b),
                        LuaConstant::Number(n) => IrConst::Number(*n),
                        LuaConstant::LuaString(s) => IrConst::String(s.to_vec()),
                    };
                    let v = alloc_reg!();
                    $ops.push(IrOp::LoadConst(v, const_val));
                    v
                } else {
                    get_reg!($rk as u8)
                }
            }};
        }

        match inst.opcode {
            Opcode::Move => {
                let r1 = get_reg!(inst.a);
                let r2 = get_reg!(inst.b as u8);
                ops.push(IrOp::Move(r1, r2));
            }
            Opcode::LoadK => {
                let const_val = match &proto.constants[inst.bx as usize] {
                    LuaConstant::Nil => IrConst::Nil,
                    LuaConstant::Boolean(b) => IrConst::Bool(*b),
                    LuaConstant::Number(n) => IrConst::Number(*n),
                    LuaConstant::LuaString(s) => IrConst::String(s.to_vec()),
                };
                let v = alloc_reg!();
                ops.push(IrOp::LoadConst(v, const_val));
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Move(ra, v));
            }
            Opcode::LoadBool => {
                let v = alloc_reg!();
                ops.push(IrOp::LoadConst(v, IrConst::Bool(inst.b != 0)));
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Move(ra, v));
                if inst.c != 0 {
                    terminator = Some(Terminator::Branch(*pc_to_block.get(&(i + 2)).unwrap_or(&BlockId(0))));
                }
            }
            Opcode::LoadNil => {
                for r in inst.a..=inst.b as u8 {
                    let v = alloc_reg!();
                    ops.push(IrOp::LoadConst(v, IrConst::Nil));
                    let rr = get_reg!(r);
                    ops.push(IrOp::Move(rr, v));
                }
            }
            Opcode::GetUpval => {
                let ra = get_reg!(inst.a);
                ops.push(IrOp::GetUpvalue(ra, inst.b as u8));
            }
            Opcode::GetGlobal => {
                if let LuaConstant::LuaString(s) = &proto.constants[inst.bx as usize] {
                    let glob = String::from_utf8_lossy(s).into_owned();
                    let ra = get_reg!(inst.a);
                    ops.push(IrOp::GetGlobal(ra, glob));
                }
            }
            Opcode::GetTable => {
                let rb = get_reg!(inst.b as u8);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::GetTable(ra, rb, rc));
            }
            Opcode::SetGlobal => {
                if let LuaConstant::LuaString(s) = &proto.constants[inst.bx as usize] {
                    let glob = String::from_utf8_lossy(s).into_owned();
                    let ra = get_reg!(inst.a);
                    ops.push(IrOp::SetGlobal(glob, ra));
                }
            }
            Opcode::SetUpval => {
                let ra = get_reg!(inst.a);
                ops.push(IrOp::SetUpvalue(inst.b as u8, ra));
            }
            Opcode::SetTable => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::SetTable(ra, rb, rc));
            }
            Opcode::NewTable => {
                let ra = get_reg!(inst.a);
                ops.push(IrOp::NewTable(ra));
            }
            Opcode::SelfOp => {
                let rb = get_reg!(inst.b as u8);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra1 = get_reg!(inst.a + 1);
                ops.push(IrOp::Move(ra1, rb));
                let ra = get_reg!(inst.a);
                ops.push(IrOp::SelfOp(ra, rb, rc));
            }
            Opcode::Add => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Add(ra, rb, rc));
            }
            Opcode::Sub => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Sub(ra, rb, rc));
            }
            Opcode::Mul => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Mul(ra, rb, rc));
            }
            Opcode::Div => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Div(ra, rb, rc));
            }
            Opcode::Mod => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Mod(ra, rb, rc));
            }
            Opcode::Pow => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Pow(ra, rb, rc));
            }
            Opcode::Unm => {
                let ra = get_reg!(inst.a);
                let rb = get_reg!(inst.b as u8);
                ops.push(IrOp::Unm(ra, rb));
            }
            Opcode::Not => {
                let ra = get_reg!(inst.a);
                let rb = get_reg!(inst.b as u8);
                ops.push(IrOp::Not(ra, rb));
            }
            Opcode::Len => {
                let ra = get_reg!(inst.a);
                let rb = get_reg!(inst.b as u8);
                ops.push(IrOp::Len(ra, rb));
            }
            Opcode::Concat => {
                let mut operands = Vec::new();
                for r in inst.b..=inst.c {
                    operands.push(get_reg!(r as u8));
                }
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Concat(ra, operands));
            }
            Opcode::Jmp => {
                let target = (i as i32 + 1 + inst.sbx) as usize;
                terminator = Some(Terminator::Branch(*pc_to_block.get(&target).unwrap_or(&BlockId(0))));
            }
            Opcode::Eq | Opcode::Lt | Opcode::Le => {
                let rb = resolve_rk!(inst.b, &mut ops);
                let rc = resolve_rk!(inst.c, &mut ops);
                let dest = alloc_reg!();
                match inst.opcode {
                    Opcode::Eq => ops.push(IrOp::Eq(dest, rb, rc)),
                    Opcode::Lt => ops.push(IrOp::Lt(dest, rb, rc)),
                    Opcode::Le => ops.push(IrOp::Le(dest, rb, rc)),
                    _ => unreachable!()
                }
                if i + 1 < proto.instructions.len() && proto.instructions[i + 1].opcode == Opcode::Jmp {
                    let jmp_inst = proto.instructions[i + 1];
                    let target_taken = (i as i32 + 2 + jmp_inst.sbx) as usize;
                    let target_not_taken = i + 2;
                    let block_taken = *pc_to_block.get(&target_taken).unwrap_or(&BlockId(0));
                    let block_not_taken = *pc_to_block.get(&target_not_taken).unwrap_or(&BlockId(0));

                    if inst.a == 0 {
                        terminator = Some(Terminator::CondBranch(dest, block_not_taken, block_taken));
                    } else {
                        terminator = Some(Terminator::CondBranch(dest, block_taken, block_not_taken));
                    }
                    i += 1; // skip the JMP
                }
            }
            Opcode::Test => {
                let cond = get_reg!(inst.a);
                if i + 1 < proto.instructions.len() && proto.instructions[i + 1].opcode == Opcode::Jmp {
                    let jmp_inst = proto.instructions[i + 1];
                    let target_taken = (i as i32 + 2 + jmp_inst.sbx) as usize;
                    let target_not_taken = i + 2;
                    let block_taken = *pc_to_block.get(&target_taken).unwrap_or(&BlockId(0));
                    let block_not_taken = *pc_to_block.get(&target_not_taken).unwrap_or(&BlockId(0));

                    if inst.c == 0 {
                        terminator = Some(Terminator::CondBranch(cond, block_not_taken, block_taken));
                    } else {
                        terminator = Some(Terminator::CondBranch(cond, block_taken, block_not_taken));
                    }
                    i += 1;
                }
            }
            Opcode::TestSet => {
                let cond = get_reg!(inst.b as u8);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Move(ra, cond));
                if i + 1 < proto.instructions.len() && proto.instructions[i + 1].opcode == Opcode::Jmp {
                    let jmp_inst = proto.instructions[i + 1];
                    let target_taken = (i as i32 + 2 + jmp_inst.sbx) as usize;
                    let target_not_taken = i + 2;
                    let block_taken = *pc_to_block.get(&target_taken).unwrap_or(&BlockId(0));
                    let block_not_taken = *pc_to_block.get(&target_not_taken).unwrap_or(&BlockId(0));

                    if inst.c == 0 {
                        terminator = Some(Terminator::CondBranch(cond, block_not_taken, block_taken));
                    } else {
                        terminator = Some(Terminator::CondBranch(cond, block_taken, block_not_taken));
                    }
                    i += 1;
                }
            }
            Opcode::Call => {
                let mut args = Vec::new();
                if inst.b != 0 {
                    for r in 1..inst.b {
                        args.push(get_reg!(inst.a + r as u8));
                    }
                }
                let mut rets = Vec::new();
                if inst.c != 0 {
                    for r in 0..inst.c - 1 {
                        rets.push(get_reg!(inst.a + r as u8));
                    }
                }
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Call(rets, ra, args));
            }
            Opcode::TailCall => {
                let mut args = Vec::new();
                if inst.b != 0 {
                    for r in 1..inst.b {
                        args.push(get_reg!(inst.a + r as u8));
                    }
                }
                let ra = get_reg!(inst.a);
                terminator = Some(Terminator::TailCall(ra, args));
            }
            Opcode::Return => {
                let mut rets = Vec::new();
                if inst.b != 0 {
                    for r in 0..inst.b - 1 {
                        rets.push(get_reg!(inst.a + r as u8));
                    }
                }
                terminator = Some(Terminator::Return(rets));
            }
            Opcode::ForLoop => {
                let target = (i as i32 + 1 + inst.sbx) as usize;
                terminator = Some(Terminator::Branch(*pc_to_block.get(&target).unwrap_or(&BlockId(0))));
                let ra = get_reg!(inst.a);
                let r3 = get_reg!(inst.a + 3);
                ops.push(IrOp::ForLoop(ra, r3));
            }
            Opcode::ForPrep => {
                let target = (i as i32 + 1 + inst.sbx) as usize;
                terminator = Some(Terminator::Branch(*pc_to_block.get(&target).unwrap_or(&BlockId(0))));
                let ra = get_reg!(inst.a);
                let r3 = get_reg!(inst.a + 3);
                ops.push(IrOp::ForPrep(ra, r3));
            }
            Opcode::TForLoop => {
                let ra = get_reg!(inst.a);
                let r2 = get_reg!(inst.a + 2);
                let r3 = get_reg!(inst.a + 3);
                ops.push(IrOp::TForLoop(ra, r2, r3));
            }
            Opcode::SetList => {
                let ra = get_reg!(inst.a);
                ops.push(IrOp::SetList(ra, inst.b as u8, inst.c as u8));
            }
            Opcode::Close => {
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Close(ra));
            }
            Opcode::Closure => {
                let f_id = FunctionId(id.0 + 1 + inst.bx);
                let ra = get_reg!(inst.a);
                ops.push(IrOp::Closure(ra, f_id));
            }
            Opcode::VarArg => {
                let mut rets = Vec::new();
                if inst.b != 0 {
                    for r in 0..inst.b - 1 {
                        rets.push(get_reg!(inst.a + r as u8));
                    }
                }
                ops.push(IrOp::VarArg(rets));
            }
        }

        if let Some(block) = blocks_map.get_mut(&current_block_id) {
            block.ops.extend(ops);
            if let Some(term) = terminator {
                block.terminator = term;
            }
        } else {
            return Err(IrError::Malformed("Block missing".into()));
        }

        i += 1;
    }

    // Ensure all blocks have terminators
    for i in 0..leader_list.len() {
        let pc = leader_list[i];
        let bid = *pc_to_block.get(&pc).unwrap();
        let next_pc = leader_list.get(i + 1);

        let block = blocks_map.get_mut(&bid).unwrap();
        if let Terminator::Unreachable = block.terminator {
            if let Some(&npc) = next_pc {
                if let Some(&nbid) = pc_to_block.get(&npc) {
                    block.terminator = Terminator::Branch(nbid);
                }
            } else {
                block.terminator = Terminator::Return(Vec::new());
            }
        }
    }

    let mut params = Vec::new();
    for p in 0..proto.num_params {
        let v = if let Some(&v) = reg_map.get(&p) {
            v
        } else {
            let v = IrValue(r_id);
            r_id += 1;
            reg_map.insert(p, v);
            v
        };
        params.push(v);
    }

    module.functions.push(IrFunction {
        id,
        blocks: blocks_map,
        cfg: ControlFlowGraph::default(),
        params,
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
