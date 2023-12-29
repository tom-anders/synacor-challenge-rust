use std::io::{BufReader, Read};

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

    let input: String = [
        "go doorway",
        "go north",
        "go north",
        "go bridge",
        "go continue",
        "go down",
        "go east",
        "take empty lantern",
        "go west",
    ]
    .into_iter()
    .interleave_shortest(std::iter::repeat("\n"))
    .collect();

    vm.run(
        &mut BufReader::new(input.as_bytes()).chain(&mut BufReader::new(&mut std::io::stdin())),
        &mut std::io::stdout(),
    )?;
    Ok(())
}
