//! Bounded Lua execution layer and string resolution.

#[allow(missing_docs)]
pub mod resolver;
#[allow(missing_docs)]
pub mod table;
#[allow(missing_docs)]
pub mod value;
#[allow(missing_docs)]
pub mod vm;

pub use resolver::StringResolver;
pub use table::LuaTable;
pub use value::{ClosureRef, LuaValue};
pub use vm::LuaVm;

use crate::error::EmuError;
use crate::ir_engine::IrModule;

/// Locates and resolves encrypted strings in the IR by running the bounded VM.
pub fn resolve_strings(module: &mut IrModule) -> Result<(), EmuError> {
    resolver::resolve_all(module)
}
