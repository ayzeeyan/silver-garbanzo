//! IR Engine: SSA construction, Control Flow Graph extraction, and Structural Analysis.

#[allow(missing_docs)]
pub mod cfg;
#[allow(missing_docs)]
pub mod lift;
#[allow(missing_docs)]
pub mod opt;
#[allow(missing_docs)]
pub mod ssa;
#[allow(missing_docs)]
pub mod structural;
#[allow(missing_docs)]
pub mod types;

pub use types::{
    BasicBlock, BlockId, ControlFlowGraph, FunctionId, IrConst, IrFunction, IrModule, IrOp, IrType,
    IrValue, PhiNode, Terminator,
};

use crate::error::IrError;
use crate::lua_parser::LuaChunk;

/// Lifts a parsed `LuaChunk` into the `IrModule` intermediate representation.
pub fn lift_to_ir(chunk: &LuaChunk<'_>) -> Result<IrModule, IrError> {
    let mut module = IrModule {
        functions: Vec::new(),
        entry: FunctionId(0),
    };

    lift::lift_proto(&chunk.root_proto, &mut module, FunctionId(0))?;

    for func in module.functions.iter_mut() {
        cfg::build_cfg(func)?;
        ssa::construct_ssa(func)?;
    }

    Ok(module)
}
