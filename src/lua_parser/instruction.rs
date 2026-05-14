use super::opcodes::{InstructionFormat, Opcode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Instruction {
    pub opcode: Opcode,
    pub a: u8,
    pub c: u16,
    pub b: u16,
    pub bx: u32,
    pub sbx: i32,
    pub raw: u32,
}

impl Instruction {
    #[allow(missing_docs)]
    pub fn decode(raw: u32) -> Option<Self> {
        let op_val = (raw & 0x3F) as u8;
        let opcode = Opcode::from_u8(op_val)?;

        let a = ((raw >> 6) & 0xFF) as u8;
        let c = ((raw >> 14) & 0x1FF) as u16;
        let b = ((raw >> 23) & 0x1FF) as u16;
        let bx = (raw >> 14) & 0x3FFFF;

        // sBx is biased by 131071 (max 18-bit unsigned / 2 - 1)
        let sbx = (bx as i32) - 131071;

        Some(Self {
            opcode,
            a,
            c,
            b,
            bx,
            sbx,
            raw,
        })
    }

    #[allow(missing_docs)]
    pub fn encode(&self) -> u32 {
        let op = (self.opcode as u32) & 0x3F;
        let a = (self.a as u32) << 6;

        match self.opcode.format() {
            InstructionFormat::IABC => {
                let c = (self.c as u32) << 14;
                let b = (self.b as u32) << 23;
                op | a | c | b
            }
            InstructionFormat::IABx => {
                let bx = self.bx << 14;
                op | a | bx
            }
            InstructionFormat::IAsBx => {
                let bx = ((self.sbx + 131071) as u32) << 14;
                op | a | bx
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_decode_encode() {
        let raw = 0x00000000; // MOVE 0 0 0
        let inst = Instruction::decode(raw).unwrap();
        assert_eq!(inst.opcode, Opcode::Move);
        assert_eq!(inst.encode(), raw);

        let raw = 0x00004041; // LOADK 1 1 (op=1, a=1, bx=1)
        let inst = Instruction::decode(raw).unwrap();
        assert_eq!(inst.opcode, Opcode::LoadK);
        assert_eq!(inst.a, 1);
        assert_eq!(inst.bx, 1);
        assert_eq!(inst.encode(), raw);
    }
}
