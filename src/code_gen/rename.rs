use std::collections::HashMap;
use crate::error::CodeGenError;
use crate::ir_engine::{IrModule, IrValue, IrOp};

#[allow(missing_docs)]
pub struct VariableRenamer {
    pub names: HashMap<IrValue, String>,
}

impl Default for VariableRenamer {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableRenamer {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    #[allow(missing_docs)]
    #[allow(clippy::map_entry)]
    pub fn rename_all(&mut self, module: &IrModule) -> Result<(), CodeGenError> {
        let mut counters = HashMap::new();

        let mut get_name = |prefix: &str| -> String {
            let cnt = counters.entry(prefix.to_string()).or_insert(0);
            let name = format!("{}_{}", prefix, cnt);
            *cnt += 1;
            name
        };

        for func in &module.functions {
            for p in &func.params {
                let name = get_name("arg");
                self.names.insert(*p, name);
            }

            for block in func.blocks.values() {
                for op in &block.ops {
                    // Collect all values generated or read from ops to make sure nothing is missing
                    let mut vals = Vec::new();

                    match op {
                        IrOp::LoadConst(d, _) => { vals.push(*d); }
                        IrOp::Move(d, s) => { vals.push(*d); vals.push(*s); }
                        IrOp::Add(d, a, b) | IrOp::Sub(d, a, b) | IrOp::Mul(d, a, b) |
                        IrOp::Div(d, a, b) | IrOp::Mod(d, a, b) | IrOp::Pow(d, a, b) => {
                            vals.push(*d); vals.push(*a); vals.push(*b);
                        }
                        IrOp::Unm(d, s) | IrOp::Not(d, s) | IrOp::Len(d, s) => {
                            vals.push(*d); vals.push(*s);
                        }
                        IrOp::Concat(d, args) => {
                            vals.push(*d); vals.extend(args);
                        }
                        IrOp::GetUpvalue(d, _) | IrOp::GetGlobal(d, _) | IrOp::NewTable(d) |
                        IrOp::Closure(d, _) | IrOp::Close(d) => {
                            vals.push(*d);
                        }
                        IrOp::GetTable(d, t, k) => {
                            vals.push(*d); vals.push(*t); vals.push(*k);
                        }
                        IrOp::Eq(d, a, b) | IrOp::Lt(d, a, b) | IrOp::Le(d, a, b) => {
                            vals.push(*d); vals.push(*a); vals.push(*b);
                        }
                        IrOp::Test(d, s) => {
                            vals.push(*d); vals.push(*s);
                        }
                        IrOp::TestSet(d, a, b) => {
                            vals.push(*d); vals.push(*a); vals.push(*b);
                        }
                        IrOp::SelfOp(d, a, b) | IrOp::TForLoop(d, a, b) => {
                            vals.push(*d); vals.push(*a); vals.push(*b);
                        }
                        IrOp::ForLoop(d, s) | IrOp::ForPrep(d, s) => {
                            vals.push(*d); vals.push(*s);
                        }
                        IrOp::SetList(d, _, _) => {
                            vals.push(*d);
                        }
                        IrOp::Call(rets, f, args) => {
                            vals.extend(rets); vals.push(*f); vals.extend(args);
                        }
                        IrOp::VarArg(rets) => {
                            vals.extend(rets);
                        }
                        IrOp::SetGlobal(_, s) | IrOp::SetUpvalue(_, s) => {
                            vals.push(*s);
                        }
                        IrOp::SetTable(t, k, v) => {
                            vals.push(*t); vals.push(*k); vals.push(*v);
                        }
                        IrOp::TailCall(f, args) => {
                            vals.push(*f); vals.extend(args);
                        }
                        IrOp::Return(rets) => {
                            vals.extend(rets);
                        }
                    };

                    for d in vals {
                        if !self.names.contains_key(&d) {
                            let prefix = match op {
                                IrOp::LoadConst(_, crate::ir_engine::types::IrConst::String(_)) => "str",
                                IrOp::LoadConst(_, crate::ir_engine::types::IrConst::Number(_)) => "num",
                                IrOp::LoadConst(_, crate::ir_engine::types::IrConst::Bool(_)) => "bool",
                                IrOp::NewTable(_) => "tbl",
                                IrOp::Closure(_, _) => "fn",
                                _ => "var"
                            };
                            let n = get_name(prefix);
                            self.names.insert(d, n);
                        }
                    }
                }
                for phi in &block.phis {
                    if !self.names.contains_key(&phi.dest) {
                        let name = get_name("var");
                        self.names.insert(phi.dest, name);
                    }
                }
                match &block.terminator {
                    crate::ir_engine::Terminator::CondBranch(v, _, _) => {
                        if !self.names.contains_key(v) {
                            let name = get_name("var");
                            self.names.insert(*v, name);
                        }
                    },
                    crate::ir_engine::Terminator::Return(rets) => {
                        for r in rets {
                            if !self.names.contains_key(r) {
                                let name = get_name("var");
                                self.names.insert(*r, name);
                            }
                        }
                    },
                    crate::ir_engine::Terminator::TailCall(f, args) => {
                        if !self.names.contains_key(f) {
                            let name = get_name("var");
                            self.names.insert(*f, name);
                        }
                        for r in args {
                            if !self.names.contains_key(r) {
                                let name = get_name("var");
                                self.names.insert(*r, name);
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    #[allow(missing_docs)]
    pub fn _get(&self, val: IrValue) -> Result<&str, CodeGenError> {
        self.names.get(&val).map(|s| s.as_str()).ok_or_else(|| {
            CodeGenError::MissingBinding(format!("{:?}", val))
        })
    }
}
