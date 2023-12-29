#[derive(Debug, Clone, Copy)]
pub struct BinOp {
    pub write_to: u16,
    pub lhs: u16,
    pub rhs: u16,
}

#[derive(Debug, Clone, Copy, strum::EnumDiscriminants)]
#[strum_discriminants(repr(u16), derive(num_enum::TryFromPrimitive))]
pub enum Opcode {
    Halt,
    Set { reg: u16, val: u16 },
    Push { val: u16 },
    Pop { write_to: u16 },
    Eq(BinOp),
    Gt(BinOp),
    Jmp { to: u16 },
    JmpIfTrue { cond: u16, to: u16 },
    JmpIfFalse { cond: u16, to: u16 },
    Add(BinOp),
    Mult(BinOp),
    Mod(BinOp),
    And(BinOp),
    Or(BinOp),
    Not { write_to: u16, val: u16 },
    ReadMem { write_to: u16, addr: u16 },
    WriteMem { addr: u16, val: u16 },
    Call { addr: u16 },
    Ret,
    Out { val: u16 },
    In { write_to: u16 },
    Noop,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadOpcodeError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    InvalidOpcode(#[from] num_enum::TryFromPrimitiveError<OpcodeDiscriminants>),
    #[error("Reached end of buffer!")]
    EndOfBuffer,
}

impl TryFrom<&[u16]> for Opcode {
    type Error = ReadOpcodeError;

    fn try_from(value: &[u16]) -> Result<Self, Self::Error> {
        let mut words = value.iter();

        let mut read_word = || {
            words
                .next()
                .ok_or(ReadOpcodeError::EndOfBuffer)
                .map(|&word| word)
        };

        let opcode = OpcodeDiscriminants::try_from(read_word()?)?;

        macro_rules! unpack_opcode {
            ($code:ident, $($fields:ident),*) => {
                Opcode::$code { $( $fields: read_word()? ),* }
            };
        }

        macro_rules! unpack_binop {
            ($code:ident) => {
                Opcode::$code(BinOp { write_to: read_word()?, lhs: read_word()?, rhs: read_word()? })
            };
        }

        use OpcodeDiscriminants::*;
        Ok(match opcode {
            Halt => Opcode::Halt,
            Set => unpack_opcode!(Set, reg, val),
            Push => unpack_opcode!(Push, val),
            Pop => unpack_opcode!(Pop, write_to),
            Eq => unpack_binop!(Eq),
            Gt => unpack_binop!(Gt),
            Jmp => unpack_opcode!(Jmp, to),
            JmpIfTrue => unpack_opcode!(JmpIfTrue, cond, to),
            JmpIfFalse => unpack_opcode!(JmpIfFalse, cond, to),
            Add => unpack_binop!(Add),
            Mult => unpack_binop!(Mult),
            Mod => unpack_binop!(Mod),
            And => unpack_binop!(And),
            Or => unpack_binop!(Or),
            Not => unpack_opcode!(Not, write_to, val),
            ReadMem => unpack_opcode!(ReadMem, write_to, addr),
            WriteMem => unpack_opcode!(WriteMem, addr, val),
            Call => unpack_opcode!(Call, addr),
            Ret => Opcode::Ret,
            Out => unpack_opcode!(Out, val),
            In => unpack_opcode!(In, write_to),
            Noop => Opcode::Noop,
        })
    }
}

impl Opcode {
    pub fn num_words(&self) -> usize {
        use OpcodeDiscriminants::*;
        match OpcodeDiscriminants::from(self) {
            Halt | Ret | Noop => 1,
            Push | Pop | Jmp | Call | Out | In => 2,
            Set | JmpIfTrue | JmpIfFalse | Not | ReadMem | WriteMem => 3,
            Eq | Gt | Add | Mult | Mod | And | Or => 4,
        }
    }
}
