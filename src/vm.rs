use crate::opcode::*;
use std::io::{BufRead, Write};

const NUM_ADDRESSES: u16 = 2 << 14;
const NUM_REGISTERS: u16 = 8;

#[derive(Debug, Clone)]
pub struct Vm {
    memory: [u16; NUM_ADDRESSES as usize],
    registers: [u16; NUM_REGISTERS as usize],
    stack: Vec<u16>,
    ip: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Program is too big to fit into memory")]
    ProgramTooBig,
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
    #[error("Stack underflow!")]
    StackUnderflow,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Vm {
    pub fn new() -> Self {
        Self {
            memory: [0; NUM_ADDRESSES as usize],
            registers: [0; NUM_REGISTERS as usize],
            stack: Vec::new(),
            ip: 0,
        }
    }

    fn lit_or_reg(&self, val: u16) -> Result<u16> {
        if val < NUM_ADDRESSES {
            Ok(val)
        } else if val < NUM_ADDRESSES + NUM_REGISTERS {
            Ok(self.registers[(val - NUM_ADDRESSES) as usize])
        } else {
            Err(Error::InvalidValue(val))
        }
    }

    fn reg_mut(&mut self, val: u16) -> Result<&mut u16> {
        self.registers
            .get_mut((val - NUM_ADDRESSES) as usize)
            .ok_or(Error::InvalidRegister(val))
    }

    pub fn load_program(&mut self, program: &[u16]) -> Result<()> {
        if program.len() > self.memory.len() {
            return Err(Error::ProgramTooBig);
        }
        self.memory[..program.len()].copy_from_slice(program);
        Ok(())
    }

    pub fn run(&mut self, output: &mut impl Write) -> Result<()> {
        loop {
            let opcode = Opcode::try_from(&self.memory[self.ip..])?;
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
                Opcode::Add { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? =
                        (self.lit_or_reg(lhs)? + self.lit_or_reg(rhs)?) % NUM_ADDRESSES;
                }
                Opcode::Eq { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? = if self.lit_or_reg(lhs)? == self.lit_or_reg(rhs)? {
                        1
                    } else {
                        0
                    }
                }
                Opcode::Push { val } => {
                    self.stack.push(self.lit_or_reg(val)?);
                }
                Opcode::Pop { write_to } => {
                    *self.reg_mut(write_to)? = self.stack.pop().ok_or(Error::StackUnderflow)?;
                }
                Opcode::Noop => (),
                _ => unimplemented!("Opcode {opcode:?} is not yet implemented!"),
            };
        }
    }
}
