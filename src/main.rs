use std::io::BufReader;

use itertools::Itertools;
use vm::Vm;

mod opcode;
mod vm;

fn main() -> vm::Result<()> {
    env_logger::init();

    let challenge = include_bytes!("../vm_challenge/challenge.bin")
        .chunks(2)
        .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
        .collect_vec();

    let mut vm = Vm::new();
    vm.load_program(&challenge)?;
    vm.run(
        &mut BufReader::new(&mut std::io::stdin()),
        &mut std::io::stdout(),
    )?;
    Ok(())
}
