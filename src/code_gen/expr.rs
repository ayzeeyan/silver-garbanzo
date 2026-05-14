use crate::error::CodeGenError;
use crate::ir_engine::IrOp;
use super::rename::VariableRenamer;

pub fn _format_expr(_op: &IrOp, _ren: &VariableRenamer) -> Result<String, CodeGenError> {
    Ok("".to_string())
}
