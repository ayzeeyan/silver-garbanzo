use nom::{
    bytes::complete::take,

    error::Error as NomError,
    IResult,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct ChunkHeader {
    pub version: u8,
    pub format: u8,
    pub endianness: Endianness,
    pub int_size: u8,
    pub size_t_size: u8,
    pub instruction_size: u8,
    pub number_size: u8,
    pub number_is_integral: bool,
}

/// Parses the 12-byte Lua 5.1 binary chunk header, applying heuristic recovery.
pub fn parse_header(input: &[u8]) -> IResult<&[u8], ChunkHeader> {
    if input.len() < 12 {
        return Err(nom::Err::Error(NomError::new(input, nom::error::ErrorKind::Eof)));
    }

    let (input, magic) = take(4usize)(input)?;

    // Heuristics Check 1: Exact match or partial match
    let mut attempted = Vec::new();
    let mut is_valid = magic == b"\x1bLua";

    if !is_valid && magic.len() == 4 && &magic[1..4] == b"Lua" {
        is_valid = true;
        attempted.push("Fixed mangled byte 0 to 0x1B");
    }

    if !is_valid {
        attempted.push("Invalid magic signature");
        return Err(nom::Err::Failure(NomError::new(input, nom::error::ErrorKind::Tag)));
    }

    let (input, version) = take(1usize)(input)?;
    let version = version[0];

    let (input, format) = take(1usize)(input)?;
    let format = format[0];

    let (input, endian_byte) = take(1usize)(input)?;
    let endianness = match endian_byte[0] {
        0 => Endianness::Big,
        1 => Endianness::Little,
        _ => return Err(nom::Err::Failure(NomError::new(input, nom::error::ErrorKind::Verify))),
    };

    let (input, int_size) = take(1usize)(input)?;
    let (input, size_t_size) = take(1usize)(input)?;
    let (input, instruction_size) = take(1usize)(input)?;
    let (input, number_size) = take(1usize)(input)?;
    let (input, integral_byte) = take(1usize)(input)?;

    let header = ChunkHeader {
        version,
        format,
        endianness,
        int_size: int_size[0],
        size_t_size: size_t_size[0],
        instruction_size: instruction_size[0],
        number_size: number_size[0],
        number_is_integral: integral_byte[0] != 0,
    };

    Ok((input, header))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_header() {
        let data = b"\x1bLua\x51\x00\x01\x04\x08\x04\x08\x00";
        let (rem, header) = parse_header(data).unwrap();
        assert!(rem.is_empty());
        assert_eq!(header.version, 0x51);
        assert_eq!(header.endianness, Endianness::Little);
        assert_eq!(header.int_size, 4);
        assert_eq!(header.size_t_size, 8);
        assert_eq!(header.instruction_size, 4);
        assert_eq!(header.number_size, 8);
        assert_eq!(header.number_is_integral, false);
    }

    #[test]
    fn test_mangled_magic_recovery() {
        let data = b"\x00Lua\x51\x00\x01\x04\x08\x04\x08\x00";
        let (rem, header) = parse_header(data).unwrap();
        assert!(rem.is_empty());
        assert_eq!(header.version, 0x51);
    }
}
