use std::collections::HashMap;
use crate::error::CodeGenError;
use crate::ir_engine::{IrModule, IrValue};

#[allow(missing_docs)]
pub struct VariableRenamer {
    names: HashMap<IrValue, String>,
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
    pub fn rename_all(&mut self, _module: &IrModule) -> Result<(), CodeGenError> {
        Ok(())
    }

    pub fn _get(&self, val: IrValue) -> Result<&str, CodeGenError> {
        self.names.get(&val).map(|s| s.as_str()).ok_or_else(|| {
            CodeGenError::MissingBinding(format!("{:?}", val))
        })
    }
}
