use crate::opcode::*;
use std::io::Write;

const NUM_ADDRESSES: u16 = 2 << 14;
const FIRST_REGISTER: u16 = NUM_ADDRESSES;
const NUM_REGISTERS: u16 = 8;

#[derive(Debug, Clone)]
pub struct Vm {
    memory: [u16; (NUM_ADDRESSES + NUM_REGISTERS) as usize],
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
    #[error("Invalid memory address: {0}")]
    InvalidAddress(u16),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Vm {
    pub fn new() -> Self {
        Self {
            memory: std::array::from_fn(|_| 0),
            stack: Vec::new(),
            ip: 0,
        }
    }

    fn lit_or_reg(&self, val: u16) -> Result<u16> {
        if val < FIRST_REGISTER {
            Ok(val)
        } else {
            self.memory
                .get(val as usize)
                .copied()
                .ok_or(Error::InvalidValue(val))
        }
    }

    fn mem_or_reg_mut(&mut self, addr: u16) -> Result<&mut u16> {
        self.memory
            .get_mut(addr as usize)
            .ok_or(Error::InvalidAddress(addr))
    }

    fn mem_or_reg(&self, addr: u16) -> Result<u16> {
        self.memory
            .get(addr as usize)
            .copied()
            .ok_or(Error::InvalidAddress(addr))
    }

    fn reg_mut(&mut self, val: u16) -> Result<&mut u16> {
        if val < FIRST_REGISTER {
            Err(Error::InvalidRegister(val))
        } else {
            self.memory
                .get_mut((val) as usize)
                .ok_or(Error::InvalidRegister(val))
        }
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

            log::trace!("{:4}: {opcode:?}", self.ip);

            self.ip += opcode.num_words();

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
                    let out = self.lit_or_reg(val)?;
                    output
                        .write_all(&[out.try_into().map_err(|_| Error::InvalidOutput(val))?])?;
                }
                Opcode::Set { reg, val } => {
                    *self.reg_mut(reg)? = self.lit_or_reg(val)?;
                }
                Opcode::Mod { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? =
                        (self.lit_or_reg(lhs)? % self.lit_or_reg(rhs)?) % NUM_ADDRESSES;
                }
                Opcode::Add { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? =
                        (self.lit_or_reg(lhs)? + self.lit_or_reg(rhs)?) % NUM_ADDRESSES;
                }
                Opcode::Mult { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? =
                        ((self.lit_or_reg(lhs)? as usize * self.lit_or_reg(rhs)? as usize)
                            % (NUM_ADDRESSES as usize)) as u16;
                }
                Opcode::Eq { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? = if self.lit_or_reg(lhs)? == self.lit_or_reg(rhs)? {
                        1
                    } else {
                        0
                    }
                }
                Opcode::Gt { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? = if self.lit_or_reg(lhs)? > self.lit_or_reg(rhs)? {
                        1
                    } else {
                        0
                    }
                }
                Opcode::And { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? =
                        (self.lit_or_reg(lhs)? & self.lit_or_reg(rhs)?) % NUM_ADDRESSES;
                }
                Opcode::Or { write_to, lhs, rhs } => {
                    *self.reg_mut(write_to)? =
                        (self.lit_or_reg(lhs)? | self.lit_or_reg(rhs)?) % NUM_ADDRESSES;
                }
                Opcode::Not { write_to, val } => {
                    *self.reg_mut(write_to)? = (!self.lit_or_reg(val)?) % NUM_ADDRESSES;
                }
                Opcode::Push { val } => {
                    self.stack.push(self.lit_or_reg(val)?);
                }
                Opcode::Pop { write_to } => {
                    *self.reg_mut(write_to)? = self.stack.pop().ok_or(Error::StackUnderflow)?;
                }
                Opcode::Call { addr } => {
                    self.stack.push(self.ip as u16);
                    self.ip = self.lit_or_reg(addr)? as usize;
                }
                Opcode::ReadMem { write_to, addr } => {
                    *self.reg_mut(write_to)? = self.memory[self.lit_or_reg(addr)? as usize];
                }
                Opcode::WriteMem { addr, val } => {
                    self.memory[self.lit_or_reg(addr)? as usize] = self.lit_or_reg(val)?;
                }
                Opcode::Ret => match self.stack.pop() {
                    None => return Ok(()),
                    Some(jump_to) => self.ip = jump_to as usize,
                },
                Opcode::Noop => (),
                _ => unimplemented!("Opcode {opcode:?} is not yet implemented!"),
            };
        }
    }
}
