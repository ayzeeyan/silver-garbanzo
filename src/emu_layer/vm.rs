use std::cell::RefCell;
use std::rc::Rc;
use crate::error::EmuError;
use super::table::LuaTable;
use super::value::{LuaTableRef, LuaValue};

const STEP_LIMIT: usize = 1_000_000;
const MEM_LIMIT: usize = 64 * 1024 * 1024; // 64 MB

#[allow(missing_docs)]
pub struct LuaVm {
    steps: usize,
    mem_allocated: usize,
    _registers: Vec<LuaValue>,
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
            _registers: vec![LuaValue::Nil; 256],
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
}
