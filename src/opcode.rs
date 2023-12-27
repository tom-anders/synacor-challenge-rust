use std::io::{BufRead, Cursor, Read};

#[derive(Debug, Clone, Copy, strum::EnumDiscriminants)]
#[strum_discriminants(repr(u16), derive(num_enum::TryFromPrimitive))]
pub enum Opcode {
    Halt,
    Set { reg: u16, val: u16 },
    Push { val: u16 },
    Pop { write_to: u16 },
    Eq { write_to: u16, lhs: u16, rhs: u16 },
    Gt { write_to: u16, lhs: u16, rhs: u16 },
    Jmp { to: u16 },
    JmpIfTrue { cond: u16, to: u16 },
    JmpIfFalse { cond: u16, to: u16 },
    Add { write_to: u16, lhs: u16, rhs: u16 },
    Mult { write_to: u16, lhs: u16, rhs: u16 },
    Mod { write_to: u16, lhs: u16, rhs: u16 },
    And { write_to: u16, lhs: u16, rhs: u16 },
    Or { write_to: u16, lhs: u16, rhs: u16 },
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

        use OpcodeDiscriminants::*;
        Ok(match opcode {
            Out => Opcode::Out { val: read_word()? },
            Halt => Opcode::Halt,
            Noop => Opcode::Noop,
            Jmp => Opcode::Jmp { to: read_word()? },
            _ => unimplemented!("Opcode {opcode:?} is not yet implemented!"),
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
