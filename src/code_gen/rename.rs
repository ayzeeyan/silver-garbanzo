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
            // Assign param names
            for p in &func.params {
                let name = get_name("arg");
                self.names.insert(*p, name);
            }

            for block in func.blocks.values() {
                for op in &block.ops {
                    let dest = match op {
                        IrOp::LoadConst(d, _) | IrOp::Move(d, _) | IrOp::Add(d, _, _) |
                        IrOp::Sub(d, _, _) | IrOp::Mul(d, _, _) | IrOp::Div(d, _, _) |
                        IrOp::Mod(d, _, _) | IrOp::Pow(d, _, _) | IrOp::Unm(d, _) |
                        IrOp::Not(d, _) | IrOp::Len(d, _) | IrOp::Concat(d, _) |
                        IrOp::GetUpvalue(d, _) | IrOp::GetGlobal(d, _) | IrOp::GetTable(d, _, _) |
                        IrOp::NewTable(d) | IrOp::Eq(d, _, _) | IrOp::Lt(d, _, _) |
                        IrOp::Le(d, _, _) | IrOp::Closure(d, _) => {
                            Some(*d)
                        }
                        IrOp::Call(rets, _, _) | IrOp::VarArg(rets) => {
                            if !rets.is_empty() {
                                Some(rets[0])
                            } else {
                                None
                            }
                        }
                        _ => None
                    };

                    if let Some(d) = dest {
                        let prefix = match op {
                            IrOp::LoadConst(_, crate::ir_engine::types::IrConst::String(_)) => "str",
                            IrOp::LoadConst(_, crate::ir_engine::types::IrConst::Number(_)) => "num",
                            IrOp::LoadConst(_, crate::ir_engine::types::IrConst::Bool(_)) => "bool",
                            IrOp::NewTable(_) => "tbl",
                            IrOp::Closure(_, _) => "fn",
                            _ => "var"
                        };
                        if !self.names.contains_key(&d) {
                            let n = get_name(prefix);
                            self.names.insert(d, n);
                        }
                    }

                    // Fallback for multi returns
                    if let IrOp::Call(rets, _, _) = op {
                        for r in rets.iter().skip(1) {
                            if !self.names.contains_key(r) {
                                let name = get_name("var");
                                self.names.insert(*r, name);
                            }
                        }
                    }
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
