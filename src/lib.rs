#![deny(warnings, missing_docs, unused)]
#![forbid(unsafe_code)]

//! `lunadec` is a production-grade Lua 5.1 decompiler and deobfuscator.
//!
//! It provides robust pipeline infrastructure to lift Lua 5.1 binary chunks and
//! deobfuscate specific Lua environments (such as Ironbrew 2, MoonSec, and Luraph).

pub mod error;
// PHASE 3: defined in phase 3
pub mod lua_parser;
// PHASE 4: defined in phase 4
pub mod ir_engine;
// PHASE 5: defined in phase 5
pub mod emu_layer;
// PHASE 6: defined in phase 6
pub mod profiles;
// PHASE 7: defined in phase 7
pub mod code_gen;

pub use error::DeobfuscationError;
