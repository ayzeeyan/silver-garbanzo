//! Zero-copy `nom` parser for Lua 5.1 binary chunks and obfuscated source components.

#[allow(missing_docs)]
pub mod constants;
#[allow(missing_docs)]
pub mod header;
#[allow(missing_docs)]
pub mod instruction;
#[allow(missing_docs)]
pub mod opcodes;
#[allow(missing_docs)]
pub mod proto;
#[allow(missing_docs)]
pub mod ast;
#[allow(missing_docs)]
pub mod compiler;
#[allow(missing_docs)]
pub mod source;

pub use constants::LuaConstant;
pub use header::{ChunkHeader, Endianness};
pub use instruction::Instruction;
pub use opcodes::InstructionFormat;
pub use opcodes::Opcode;
pub use proto::FunctionProto;

/// A fully parsed Lua 5.1 binary chunk or reconstructed chunk from
/// obfuscated source. Borrows zero-copy from the input buffer.
#[derive(Debug, Clone)]
pub struct LuaChunk<'input> {
    /// 12-byte binary chunk header.
    pub header: ChunkHeader,
    /// Root function prototype; contains nested protos recursively.
    pub root_proto: FunctionProto<'input>,
}

/// Tries to parse the given byte buffer as a Lua 5.1 binary chunk.
pub fn parse_chunk(input: &[u8]) -> Result<LuaChunk<'_>, crate::error::ParserError> {
    let (rem, header) = header::parse_header(input)
        .map_err(|_| crate::error::ParserError::MalformedChunk("Failed to parse header".into()))?;

    let (_, root_proto) = proto::parse_proto(rem, &header)
        .map_err(|_| crate::error::ParserError::MalformedChunk("Failed to parse root proto".into()))?;

    Ok(LuaChunk { header, root_proto })
}

/// Tries to reconstruct a `LuaChunk` from obfuscated raw source.
pub fn parse_source(source: &str) -> Result<LuaChunk<'_>, crate::error::ParserError> {
    source::parse_obfuscated_source(source)
}
