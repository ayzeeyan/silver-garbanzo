use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use crate::error::EmuError;
use super::table::LuaTable;
use super::value::{LuaTableRef, LuaValue};
use crate::ir_engine::{IrOp, IrValue, IrConst};

const STEP_LIMIT: usize = 1_000_000;
const MEM_LIMIT: usize = 64 * 1024 * 1024; // 64 MB

#[allow(missing_docs)]
pub struct LuaVm {
    steps: usize,
    mem_allocated: usize,
    registers: HashMap<IrValue, LuaValue>,
}

impl Default for LuaVm {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaVm {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self {
            steps: 0,
            mem_allocated: 0,
            registers: HashMap::new(),
        }
    }

    #[allow(missing_docs)]
    pub fn allocate_table(&mut self) -> Result<LuaTableRef, EmuError> {
        let size = std::mem::size_of::<LuaTable>();
        if self.mem_allocated + size > MEM_LIMIT {
            return Err(EmuError::ResourceLimitExceeded {
                kind: "memory",
                limit: MEM_LIMIT,
            });
        }
        self.mem_allocated += size;
        Ok(Rc::new(RefCell::new(LuaTable::new())))
    }

    #[allow(missing_docs)]
    pub fn step(&mut self) -> Result<(), EmuError> {
        self.steps += 1;
        if self.steps > STEP_LIMIT {
            return Err(EmuError::ResourceLimitExceeded {
                kind: "instructions",
                limit: STEP_LIMIT,
            });
        }
        Ok(())
    }

    #[allow(missing_docs)]
    pub fn get_reg(&self, reg: IrValue) -> LuaValue {
        self.registers.get(&reg).cloned().unwrap_or(LuaValue::Nil)
    }

    #[allow(missing_docs)]
    pub fn set_reg(&mut self, reg: IrValue, val: LuaValue) {
        self.registers.insert(reg, val);
    }

    // Evaluate a single IR operation mapped out from decoder execution scope rules!
    #[allow(missing_docs)]
    pub fn eval_op(&mut self, op: &IrOp) -> Result<(), EmuError> {
        self.step()?;
        match op {
            IrOp::LoadConst(dest, c) => {
                let v = match c {
                    IrConst::Nil => LuaValue::Nil,
                    IrConst::Bool(b) => LuaValue::Bool(*b),
                    IrConst::Number(n) => LuaValue::Float(*n),
                    IrConst::String(s) => LuaValue::LuaString(s.clone()),
                };
                self.set_reg(*dest, v);
            }
            IrOp::Move(dest, src) => {
                let v = self.get_reg(*src);
                self.set_reg(*dest, v);
            }
            IrOp::NewTable(dest) => {
                let t = self.allocate_table()?;
                self.set_reg(*dest, LuaValue::Table(t));
            }
            IrOp::GetTable(dest, t, k) => {
                let table_val = self.get_reg(*t);
                let key_val = self.get_reg(*k);
                if let LuaValue::Table(tbl) = table_val {
                    let v = tbl.borrow().get(&key_val);
                    self.set_reg(*dest, v);
                } else {
                    return Err(EmuError::RuntimeError("Attempt to index a non-table".into()));
                }
            }
            IrOp::SetTable(t, k, v) => {
                let table_val = self.get_reg(*t);
                let key_val = self.get_reg(*k);
                let val = self.get_reg(*v);
                if let LuaValue::Table(tbl) = table_val {
                    tbl.borrow_mut().set(key_val, val);
                } else {
                    return Err(EmuError::RuntimeError("Attempt to index a non-table".into()));
                }
            }
            // Support required numeric decoding
            IrOp::Add(dest, a, b) | IrOp::Sub(dest, a, b) | IrOp::Mul(dest, a, b) | IrOp::Div(dest, a, b) | IrOp::Mod(dest, a, b) => {
                let va = self.get_reg(*a);
                let vb = self.get_reg(*b);
                if let (LuaValue::Float(fa), LuaValue::Float(fb)) = (va, vb) {
                    let res = match op {
                        IrOp::Add(..) => fa + fb,
                        IrOp::Sub(..) => fa - fb,
                        IrOp::Mul(..) => fa * fb,
                        IrOp::Div(..) => fa / fb,
                        IrOp::Mod(..) => fa % fb,
                        _ => unreachable!(),
                    };
                    self.set_reg(*dest, LuaValue::Float(res));
                } else {
                    return Err(EmuError::RuntimeError("Arithmetic on non-number".into()));
                }
            }
            _ => {
                return Err(EmuError::UnsupportedOpcode(format!("{:?}", op)));
            }
        }
        Ok(())
    }
}
