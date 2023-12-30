use std::io::stdout;

mod maze;
mod opcode;
mod rpg;
mod vm;

use itertools::Itertools;
use log::info;
use vm::Vm;

use crate::{maze::Maze, rpg::{Rpg, Command, Direction}, vm::ExitReason};

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
                "go ladder",
            ],
            &mut stdout()
        )?,
        ExitReason::NoMoreInput
    );

    info!("Start exploring maze...");

    let mut rpg = Rpg::new(&mut vm);

    {
        let mut maze = Maze::new(&mut rpg);
        maze.random_moves_until(|room| room.items.first().is_some_and(|item| item == "can"))?;
    }

    rpg.command(Command::Take("can"))?;
    rpg.command(Command::Look("can"))?;
    rpg.command(Command::Use("can"))?;

    rpg.command(Command::Look("lantern"))?;

    rpg.command(Command::Use("lantern"))?;

    rpg.go(Direction::West)?;
    rpg.go("ladder")?;
    rpg.go("darkness")?;
    rpg.go("continue")?;
    rpg.go(Direction::West)?;
    rpg.go(Direction::West)?;
    rpg.go(Direction::West)?;
    rpg.go(Direction::West)?;

    vm.run_interactive()?;

    Ok(())
}
