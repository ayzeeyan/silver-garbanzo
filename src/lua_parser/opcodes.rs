
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum InstructionFormat {
    IABC,
    IABx,
    IAsBx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Opcode {
    Move = 0,
    LoadK = 1,
    LoadBool = 2,
    LoadNil = 3,
    GetUpval = 4,
    GetGlobal = 5,
    GetTable = 6,
    SetGlobal = 7,
    SetUpval = 8,
    SetTable = 9,
    NewTable = 10,
    SelfOp = 11,
    Add = 12,
    Sub = 13,
    Mul = 14,
    Div = 15,
    Mod = 16,
    Pow = 17,
    Unm = 18,
    Not = 19,
    Len = 20,
    Concat = 21,
    Jmp = 22,
    Eq = 23,
    Lt = 24,
    Le = 25,
    Test = 26,
    TestSet = 27,
    Call = 28,
    TailCall = 29,
    Return = 30,
    ForLoop = 31,
    ForPrep = 32,
    TForLoop = 33,
    SetList = 34,
    Close = 35,
    Closure = 36,
    VarArg = 37,
}

impl Opcode {
    #[allow(missing_docs)]
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 37 {
            // SAFETY: Because we validated the bound, transmutation here is safe.
            // But we have #![forbid(unsafe_code)] so we map manually.
            Some(match val {
                0 => Self::Move, 1 => Self::LoadK, 2 => Self::LoadBool, 3 => Self::LoadNil,
                4 => Self::GetUpval, 5 => Self::GetGlobal, 6 => Self::GetTable, 7 => Self::SetGlobal,
                8 => Self::SetUpval, 9 => Self::SetTable, 10 => Self::NewTable, 11 => Self::SelfOp,
                12 => Self::Add, 13 => Self::Sub, 14 => Self::Mul, 15 => Self::Div, 16 => Self::Mod,
                17 => Self::Pow, 18 => Self::Unm, 19 => Self::Not, 20 => Self::Len, 21 => Self::Concat,
                22 => Self::Jmp, 23 => Self::Eq, 24 => Self::Lt, 25 => Self::Le, 26 => Self::Test,
                27 => Self::TestSet, 28 => Self::Call, 29 => Self::TailCall, 30 => Self::Return,
                31 => Self::ForLoop, 32 => Self::ForPrep, 33 => Self::TForLoop, 34 => Self::SetList,
                35 => Self::Close, 36 => Self::Closure, 37 => Self::VarArg,
                _ => unreachable!(),
            })
        } else {
            None
        }
    }

    #[allow(missing_docs)]
    pub fn format(&self) -> InstructionFormat {
        match self {
            Self::LoadK | Self::GetGlobal | Self::SetGlobal | Self::Closure => InstructionFormat::IABx,
            Self::Jmp | Self::ForLoop | Self::ForPrep => InstructionFormat::IAsBx,
            _ => InstructionFormat::IABC,
        }
    }

    #[allow(missing_docs)]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Move => "MOVE", Self::LoadK => "LOADK", Self::LoadBool => "LOADBOOL",
            Self::LoadNil => "LOADNIL", Self::GetUpval => "GETUPVAL", Self::GetGlobal => "GETGLOBAL",
            Self::GetTable => "GETTABLE", Self::SetGlobal => "SETGLOBAL", Self::SetUpval => "SETUPVAL",
            Self::SetTable => "SETTABLE", Self::NewTable => "NEWTABLE", Self::SelfOp => "SELF",
            Self::Add => "ADD", Self::Sub => "SUB", Self::Mul => "MUL", Self::Div => "DIV",
            Self::Mod => "MOD", Self::Pow => "POW", Self::Unm => "UNM", Self::Not => "NOT",
            Self::Len => "LEN", Self::Concat => "CONCAT", Self::Jmp => "JMP", Self::Eq => "EQ",
            Self::Lt => "LT", Self::Le => "LE", Self::Test => "TEST", Self::TestSet => "TESTSET",
            Self::Call => "CALL", Self::TailCall => "TAILCALL", Self::Return => "RETURN",
            Self::ForLoop => "FORLOOP", Self::ForPrep => "FORPREP", Self::TForLoop => "TFORLOOP",
            Self::SetList => "SETLIST", Self::Close => "CLOSE", Self::Closure => "CLOSURE",
            Self::VarArg => "VARARG",
        }
    }

    #[allow(missing_docs)]
    pub fn has_jump(&self) -> bool {
        matches!(self, Self::Jmp | Self::Eq | Self::Lt | Self::Le | Self::Test | Self::TestSet | Self::ForLoop | Self::ForPrep | Self::TForLoop)
    }
}
