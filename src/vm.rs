use crate::opcode::*;
use std::io::{BufRead, Write};

#[derive(Debug, Clone)]
pub struct Vm {
    memory: [u16; 2_usize.pow(15)],
    registers: [u16; 8],
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
}

pub type Result<T> = std::result::Result<T, Error>;

impl Vm {
    pub fn new() -> Self {
        Self {
            memory: [0; 2_usize.pow(15)],
            registers: [0; 8],
            stack: Vec::new(),
            ip: 0,
        }
    }

    pub fn run(&mut self, program: &[u16], output: &mut impl Write) -> Result<()> {
        loop {
            let opcode = Opcode::try_from(&program[self.ip..])?;
            self.ip += opcode.num_words();

            match opcode {
                Opcode::Halt => return Ok(()),
                Opcode::Out { val } => {
                    output
                        .write_all(&[val.try_into().map_err(|_| Error::InvalidOutput(val))?])?;
                }

                Opcode::Noop => (),
                _ => unimplemented!("Opcode {opcode:?} is not yet implemented!"),
            };
        }
    }
}
