use std::io::stdout;

use itertools::Itertools;
use vm::Vm;

use crate::vm::ExitReason;

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

    assert_eq!(
        vm.run_commands(
            [
                "go doorway",
                "go north",
                "go north",
                "go bridge",
                "go continue",
                "go down",
                "go east",
                "take empty lantern",
                "go west",
                "go west",
                "go passage",
            ],
            &mut stdout()
        )?,
        ExitReason::NoMoreInput
    );

    vm.run_interactive()?;

    Ok(())
}
