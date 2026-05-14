use crate::error::CodeGenError;
use crate::ir_engine::BasicBlock;
use super::emitter::Emitter;
use super::rename::VariableRenamer;

pub fn _emit_block(_block: &BasicBlock, _emit: &mut Emitter, _ren: &VariableRenamer) -> Result<(), CodeGenError> {
    Ok(())
}
