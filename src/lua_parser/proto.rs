use std::borrow::Cow;
use nom::{
    bytes::complete::take,
    number::complete::{le_u8, le_u32, le_u64, le_f64},
    IResult,
};

use super::constants::LuaConstant;
use super::header::{ChunkHeader, Endianness};
use super::instruction::Instruction;

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct FunctionProto<'input> {
    pub source_name: Option<&'input [u8]>,
    pub line_defined: u32,
    pub last_line_defined: u32,
    pub num_upvalues: u8,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<LuaConstant<'input>>,
    pub protos: Vec<FunctionProto<'input>>,
    pub line_info: Vec<u32>,
    pub local_vars: Vec<LocalVar<'input>>,
    pub upvalue_names: Vec<&'input [u8]>,
}

#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct LocalVar<'input> {
    pub name: &'input [u8],
    pub start_pc: u32,
    pub end_pc: u32,
}

fn parse_int<'a>(input: &'a [u8], header: &ChunkHeader) -> IResult<&'a [u8], u32> {
    if header.int_size == 4 {
        if header.endianness == Endianness::Little {
            le_u32(input)
        } else {
            nom::number::complete::be_u32(input)
        }
    } else {
        // Assume 8
        if header.endianness == Endianness::Little {
            let (i, v) = le_u64(input)?;
            Ok((i, v as u32))
        } else {
            let (i, v) = nom::number::complete::be_u64(input)?;
            Ok((i, v as u32))
        }
    }
}

fn parse_size_t<'a>(input: &'a [u8], header: &ChunkHeader) -> IResult<&'a [u8], usize> {
    if header.size_t_size == 4 {
        if header.endianness == Endianness::Little {
            let (i, v) = le_u32(input)?;
            Ok((i, v as usize))
        } else {
            let (i, v) = nom::number::complete::be_u32(input)?;
            Ok((i, v as usize))
        }
    } else {
        if header.endianness == Endianness::Little {
            let (i, v) = le_u64(input)?;
            Ok((i, v as usize))
        } else {
            let (i, v) = nom::number::complete::be_u64(input)?;
            Ok((i, v as usize))
        }
    }
}

fn parse_string<'a>(input: &'a [u8], header: &ChunkHeader) -> IResult<&'a [u8], Option<&'a [u8]>> {
    let (input, size) = parse_size_t(input, header)?;
    if size == 0 {
        return Ok((input, None));
    }
    let len = size - 1; // Lua strings include trailing \0 in size
    let (input, bytes) = take(len)(input)?;
    let (input, _) = take(1usize)(input)?; // consume \0
    Ok((input, Some(bytes)))
}

fn parse_instruction<'a>(input: &'a [u8], header: &ChunkHeader) -> IResult<&'a [u8], Instruction> {
    let (input, raw) = if header.endianness == Endianness::Little {
        le_u32(input)?
    } else {
        nom::number::complete::be_u32(input)?
    };

    let inst = Instruction::decode(raw).ok_or_else(|| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
    })?;

    Ok((input, inst))
}

fn parse_constant<'a>(input: &'a [u8], header: &ChunkHeader) -> IResult<&'a [u8], LuaConstant<'a>> {
    let (input, tag) = le_u8(input)?;
    match tag {
        0 => Ok((input, LuaConstant::Nil)),
        1 => {
            let (input, val) = le_u8(input)?;
            Ok((input, LuaConstant::Boolean(val != 0)))
        }
        3 => {
            let (input, val) = if header.number_size == 8 {
                if header.endianness == Endianness::Little {
                    le_f64(input)?
                } else {
                    nom::number::complete::be_f64(input)?
                }
            } else {
                // Ignore other sizes for now, fallback to f64 via parsing
                (input, 0.0)
            };
            Ok((input, LuaConstant::Number(val)))
        }
        4 => {
            let (input, s) = parse_string(input, header)?;
            let s = s.unwrap_or(b"");
            Ok((input, LuaConstant::LuaString(Cow::Borrowed(s))))
        }
        _ => Err(nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
    }
}

#[allow(missing_docs)]
pub fn parse_proto<'a>(input: &'a [u8], header: &ChunkHeader) -> IResult<&'a [u8], FunctionProto<'a>> {
    let (input, source_name) = parse_string(input, header)?;
    let (input, line_defined) = parse_int(input, header)?;
    let (input, last_line_defined) = parse_int(input, header)?;
    let (input, num_upvalues) = le_u8(input)?;
    let (input, num_params) = le_u8(input)?;
    let (input, is_vararg) = le_u8(input)?;
    let (input, max_stack_size) = le_u8(input)?;

    let (input, num_inst) = parse_int(input, header)?;
    let mut instructions = Vec::with_capacity(num_inst as usize);
    let mut current_input = input;
    for _ in 0..num_inst {
        let (next_input, inst) = parse_instruction(current_input, header)?;
        instructions.push(inst);
        current_input = next_input;
    }

    let (input, num_const) = parse_int(current_input, header)?;
    let mut constants = Vec::with_capacity(num_const as usize);
    let mut current_input = input;
    for _ in 0..num_const {
        let (next_input, constant) = parse_constant(current_input, header)?;
        constants.push(constant);
        current_input = next_input;
    }

    let (input, num_protos) = parse_int(current_input, header)?;
    let mut protos = Vec::with_capacity(num_protos as usize);
    let mut current_input = input;
    for _ in 0..num_protos {
        let (next_input, proto) = parse_proto(current_input, header)?;
        protos.push(proto);
        current_input = next_input;
    }

    let (input, num_line_info) = parse_int(current_input, header)?;
    let mut line_info = Vec::with_capacity(num_line_info as usize);
    let mut current_input = input;
    for _ in 0..num_line_info {
        let (next_input, line) = parse_int(current_input, header)?;
        line_info.push(line);
        current_input = next_input;
    }

    let (input, num_locals) = parse_int(current_input, header)?;
    let mut local_vars = Vec::with_capacity(num_locals as usize);
    let mut current_input = input;
    for _ in 0..num_locals {
        let (next_input, name) = parse_string(current_input, header)?;
        let name = name.unwrap_or(b"");
        let (next_input, start_pc) = parse_int(next_input, header)?;
        let (next_input, end_pc) = parse_int(next_input, header)?;
        local_vars.push(LocalVar { name, start_pc, end_pc });
        current_input = next_input;
    }

    let (input, num_upval_names) = parse_int(current_input, header)?;
    let mut upvalue_names = Vec::with_capacity(num_upval_names as usize);
    let mut current_input = input;
    for _ in 0..num_upval_names {
        let (next_input, name) = parse_string(current_input, header)?;
        upvalue_names.push(name.unwrap_or(b""));
        current_input = next_input;
    }

    Ok((current_input, FunctionProto {
        source_name,
        line_defined,
        last_line_defined,
        num_upvalues,
        num_params,
        is_vararg,
        max_stack_size,
        instructions,
        constants,
        protos,
        line_info,
        local_vars,
        upvalue_names,
    }))
}
