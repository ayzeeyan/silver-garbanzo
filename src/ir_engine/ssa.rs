use indexmap::IndexMap;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::error::IrError;
use super::types::*;

pub fn construct_ssa(func: &mut IrFunction) -> Result<(), IrError> {
    compute_dominators(func)?;
    compute_dominance_frontiers(func)?;
    insert_phis(func)?;
    rename_variables(func)?;
    Ok(())
}

fn compute_dominators(func: &mut IrFunction) -> Result<(), IrError> {
    let mut doms = IndexMap::new();
    let root = BlockId(0);

    let blocks: Vec<BlockId> = func.blocks.keys().copied().collect();
    if blocks.is_empty() {
        return Ok(());
    }

    let all_blocks: HashSet<BlockId> = blocks.iter().copied().collect();

    for &b in &blocks {
        if b == root {
            let mut s = HashSet::new();
            s.insert(b);
            doms.insert(b, s);
        } else {
            doms.insert(b, all_blocks.clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for &b in &blocks {
            if b == root { continue; }

            let preds = func.cfg.predecessors.get(&b).ok_or_else(|| IrError::Malformed("Missing predecessors".into()))?;
            if preds.is_empty() { continue; }

            let mut new_dom: HashSet<BlockId> = doms.get(&preds[0]).ok_or_else(|| IrError::Malformed("Missing dominators".into()))?.clone();
            for p in preds.iter().skip(1) {
                if let Some(p_doms) = doms.get(p) {
                    new_dom = new_dom.intersection(p_doms).copied().collect();
                }
            }
            new_dom.insert(b);

            if new_dom != *doms.get(&b).ok_or_else(|| IrError::Malformed("Missing dominators".into()))? {
                doms.insert(b, new_dom);
                changed = true;
            }
        }
    }

    let mut idoms = IndexMap::new();
    idoms.insert(root, root);

    for &b in &blocks {
        if b == root { continue; }
        let my_doms = doms.get(&b).ok_or_else(|| IrError::Malformed("Missing dominators".into()))?;
        let mut strict_doms: HashSet<BlockId> = my_doms.clone();
        strict_doms.remove(&b);

        let mut idom = None;
        for &d in &strict_doms {
            let is_idom = strict_doms.iter().all(|&other_d| {
                other_d == d || doms.get(&other_d).unwrap_or(&HashSet::new()).contains(&d)
            });
            if is_idom {
                idom = Some(d);
                break;
            }
        }
        if let Some(idom) = idom {
            idoms.insert(b, idom);
        } else {
            idoms.insert(b, root);
        }
    }

    func.cfg.dominators = idoms;
    Ok(())
}

fn compute_dominance_frontiers(func: &mut IrFunction) -> Result<(), IrError> {
    let mut dfs = IndexMap::new();
    for &b in func.blocks.keys() {
        dfs.insert(b, HashSet::new());
    }

    for (&b, _block) in &func.blocks {
        let preds = func.cfg.predecessors.get(&b).ok_or_else(|| IrError::Malformed("Missing preds".into()))?;
        if preds.len() >= 2 {
            for &p in preds {
                let mut runner = p;
                while runner != *func.cfg.dominators.get(&b).unwrap_or(&BlockId(0)) {
                    if let Some(dfs_set) = dfs.get_mut(&runner) {
                        dfs_set.insert(b);
                    }
                    runner = *func.cfg.dominators.get(&runner).unwrap_or(&BlockId(0));
                }
            }
        }
    }

    let mut dfs_vec = IndexMap::new();
    for (k, v) in dfs {
        dfs_vec.insert(k, v.into_iter().collect());
    }
    func.cfg.dominance_frontiers = dfs_vec;
    Ok(())
}

fn insert_phis(func: &mut IrFunction) -> Result<(), IrError> {
    let mut blocks_defining_var: HashMap<IrValue, HashSet<BlockId>> = HashMap::new();

    for (&bid, block) in &func.blocks {
        let mut var_kill = HashSet::new();
        for op in &block.ops {
            let defs = get_defs(op);
            for d in defs {
                var_kill.insert(d);
                blocks_defining_var.entry(d).or_default().insert(bid);
            }
        }
    }

    for (var, def_blocks) in blocks_defining_var {
        let mut worklist: VecDeque<BlockId> = def_blocks.into_iter().collect();
        let mut has_phi = HashSet::new();

        while let Some(x) = worklist.pop_front() {
            if let Some(df) = func.cfg.dominance_frontiers.get(&x) {
                for &y in df {
                    if !has_phi.contains(&y) {
                        if let Some(block) = func.blocks.get_mut(&y) {
                            block.phis.push(PhiNode {
                                dest: var,
                                incoming: Vec::new(),
                            });
                        }
                        has_phi.insert(y);
                        worklist.push_back(y);
                    }
                }
            }
        }
    }
    Ok(())
}

fn rename_variables(_func: &mut IrFunction) -> Result<(), IrError> {
    Ok(())
}

fn get_defs(op: &IrOp) -> Vec<IrValue> {
    match op {
        IrOp::LoadConst(v, _) => vec![*v],
        IrOp::Move(d, _) => vec![*d],
        IrOp::GetUpvalue(d, _) => vec![*d],
        IrOp::GetGlobal(d, _) => vec![*d],
        IrOp::GetTable(d, _, _) => vec![*d],
        IrOp::NewTable(d) => vec![*d],
        IrOp::SelfOp(d, _, _) => vec![*d],
        IrOp::Add(d, _, _) => vec![*d],
        IrOp::Sub(d, _, _) => vec![*d],
        IrOp::Mul(d, _, _) => vec![*d],
        IrOp::Div(d, _, _) => vec![*d],
        IrOp::Mod(d, _, _) => vec![*d],
        IrOp::Pow(d, _, _) => vec![*d],
        IrOp::Unm(d, _) => vec![*d],
        IrOp::Not(d, _) => vec![*d],
        IrOp::Len(d, _) => vec![*d],
        IrOp::Concat(d, _) => vec![*d],
        IrOp::Call(rets, _, _) => rets.clone(),
        IrOp::VarArg(rets) => rets.clone(),
        IrOp::Eq(d, _, _) | IrOp::Lt(d, _, _) | IrOp::Le(d, _, _) => vec![*d],
        IrOp::ForPrep(d, _) | IrOp::ForLoop(d, _) => vec![*d],
        IrOp::Closure(d, _) => vec![*d],
        _ => vec![],
    }
}
