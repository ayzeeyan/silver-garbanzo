//! Output emission logic transforming optimized `IrModule` into valid Lua 5.1 code.

#[allow(missing_docs)]
pub mod emitter;
#[allow(missing_docs)]
pub mod expr;
#[allow(missing_docs)]
pub mod rename;
#[allow(missing_docs)]
pub mod stmt;
#[allow(missing_docs)]
pub mod sugar;

use crate::error::CodeGenError;
use crate::ir_engine::IrModule;

#[allow(missing_docs)]
pub fn generate(ir: &IrModule) -> Result<String, CodeGenError> {
    let mut ren = rename::VariableRenamer::new();
    ren.rename_all(ir)?;

    let mut emit = emitter::Emitter::new();
    emit.emit_module(ir, &ren)?;

    let mut output = emit.into_string();
    sugar::apply_all(&mut output);

    Ok(output)
}
