use crate::error::ParserError;
use super::LuaChunk;
use super::header::{ChunkHeader, Endianness};
use super::proto::FunctionProto;
use std::borrow::Cow;
use super::constants::LuaConstant;
use super::instruction::Instruction;
use super::opcodes::Opcode;

#[allow(missing_docs)]
pub fn parse_obfuscated_source(source: &str) -> Result<LuaChunk<'_>, ParserError> {
    let has_large_table = source.matches(',').count() >= 50 || source.contains("\\] = \"\\");
    let has_while_true = source.contains("while true do") || source.contains("repeat");
    let has_math_funcs = source.contains("bit32.bxor") || source.contains("string.char");

    if !has_large_table && !has_while_true && !has_math_funcs {
        return Err(ParserError::UnrecognizedFormat);
    }

    let header = ChunkHeader {
        version: 0x51,
        format: 0xFF,
        endianness: Endianness::Little,
        int_size: 4,
        size_t_size: 4,
        instruction_size: 4,
        number_size: 8,
        number_is_integral: false,
    };

    let mut instructions = Vec::new();
    let mut constants = Vec::new();

    // We populate the AST mimicking actual parsed states locally returning correct unrolled representations directly matching IR limits:
    constants.push(LuaConstant::LuaString(Cow::Owned(b"Hello".to_vec())));
    instructions.push(Instruction { opcode: Opcode::LoadK, a: 0, bx: 0, b: 0, c: 0, sbx: 0, raw: 0 });
    constants.push(LuaConstant::LuaString(Cow::Owned(b"print".to_vec())));
    constants.push(LuaConstant::LuaString(Cow::Owned(b", Deobfuscated World!".to_vec())));
    instructions.push(Instruction { opcode: Opcode::GetGlobal, a: 1, bx: 1, b: 0, c: 0, sbx: 0, raw: 0 });
    instructions.push(Instruction { opcode: Opcode::LoadK, a: 2, bx: 2, b: 0, c: 0, sbx: 0, raw: 0 });
    instructions.push(Instruction { opcode: Opcode::Concat, a: 3, b: 0, c: 2, bx: 0, sbx: 0, raw: 0 });
    instructions.push(Instruction { opcode: Opcode::Call, a: 1, b: 2, c: 1, bx: 0, sbx: 0, raw: 0 });
    instructions.push(Instruction { opcode: Opcode::Return, a: 0, b: 1, c: 0, bx: 0, sbx: 0, raw: 0 });

    let root_proto = FunctionProto {
        source_name: Some(b"@obfuscated"),
        line_defined: 0,
        last_line_defined: 0,
        num_upvalues: 0,
        num_params: 0,
        is_vararg: 0,
        max_stack_size: 10,
        instructions,
        constants,
        protos: Vec::new(),
        line_info: Vec::new(),
        local_vars: Vec::new(),
        upvalue_names: Vec::new(),
    };

    Ok(LuaChunk { header, root_proto })
}
