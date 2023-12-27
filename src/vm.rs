use crate::opcode::*;
use std::io::{BufRead, Write};

const NUM_ADDRESSES: usize = 2 << 14;
const NUM_REGISTERS: usize = 8;

#[derive(Debug, Clone)]
pub struct Vm {
    memory: [u16; NUM_ADDRESSES],
    registers: [u16; NUM_REGISTERS],
    stack: Vec<u16>,
    ip: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ReadOpcode(#[from] ReadOpcodeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("`{0}` is valid ascii!")]
    InvalidOutput(u16),
    #[error("Invalid value: {0}")]
    InvalidValue(u16),
    #[error("Invalid register address: {0}")]
    InvalidRegister(u16),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Vm {
    pub fn new() -> Self {
        Self {
            memory: [0; NUM_ADDRESSES],
            registers: [0; NUM_REGISTERS],
            stack: Vec::new(),
            ip: 0,
        }
    }

    fn lit_or_reg(&self, val: u16) -> Result<u16> {
        if (val as usize) < NUM_ADDRESSES {
            Ok(val)
        } else if (val as usize) < NUM_ADDRESSES + NUM_REGISTERS {
            Ok(self.registers[val as usize - NUM_ADDRESSES])
        } else {
            Err(Error::InvalidValue(val))
        }
    }

    fn reg_mut(&mut self, val: u16) -> Result<&mut u16> {
        self.registers.get_mut(val as usize - NUM_ADDRESSES)
            .ok_or(Error::InvalidRegister(val))
    }

    pub fn run(&mut self, program: &[u16], output: &mut impl Write) -> Result<()> {
        loop {
            let opcode = Opcode::try_from(&program[self.ip..])?;
            self.ip += opcode.num_words();

            log::trace!("Next opode: {opcode:?}, ip: {}", self.ip);

            match opcode {
                Opcode::Halt => return Ok(()),
                Opcode::Jmp { to } => self.ip = to as usize,
                Opcode::JmpIfTrue { cond, to } => {
                    if self.lit_or_reg(cond)? != 0 {
                        self.ip = to as usize;
                    }
                }
                Opcode::JmpIfFalse { cond, to } => {
                    if self.lit_or_reg(cond)? == 0 {
                        self.ip = to as usize;
                    }
                }
                Opcode::Out { val } => {
                    output
                        .write_all(&[val.try_into().map_err(|_| Error::InvalidOutput(val))?])?;
                }
                Opcode::Set { reg, val } => {
                    *self.reg_mut(reg)? = self.lit_or_reg(val)?;
                }
                Opcode::Noop => (),
                _ => unimplemented!("Opcode {opcode:?} is not yet implemented!"),
            };
        }
    }
}
