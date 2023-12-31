use std::io::stdout;

mod maze;
mod opcode;
mod rpg;
mod vm;

use itertools::Itertools;
use log::info;
use vm::Vm;

use crate::{
    maze::Maze,
    rpg::{Command, Direction, Rpg},
    vm::ExitReason,
};

#[derive(Debug, Clone, Copy, parse_display::Display)]
#[display(style = "snake_case")]
enum Coin {
    Red = 2,
    Corroded = 3,
    Shiny = 5,
    Concave = 7,
    Blue = 9,
}

fn solve_coin_puzzle() -> Vec<Coin> {
    use Coin::*;
    [Red, Corroded, Shiny, Concave, Blue]
        .into_iter()
        .permutations(5)
        .find(|perm| {
            let (a, b, c, d, e) = perm.iter().map(|c| *c as u64).collect_tuple().unwrap();
            a + b * c.pow(2) + d.pow(3) - e == 399
        })
        .unwrap()
}

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

    use Command::*;
    use Direction::*;
    rpg.command(Take("can"))?;
    rpg.command(Look("can"))?;
    rpg.command(Use("can"))?;

    rpg.command(Look("lantern"))?;

    rpg.command(Use("lantern"))?;

    rpg.go(West)?;
    rpg.go("ladder")?;
    rpg.go("darkness")?;
    rpg.go("continue")?;
    rpg.go(West)?;
    rpg.go(West)?;
    rpg.go(West)?;
    rpg.go(West)?;
    rpg.go(North)?;

    rpg.command(Take("red coin"))?;
    rpg.go(North)?;
    rpg.go(East)?;
    rpg.command(Take("concave coin"))?;
    rpg.go("down")?;
    rpg.command(Take("corroded coin"))?;
    rpg.go("up")?;
    rpg.go(West)?;
    rpg.go(West)?;
    rpg.command(Take("blue coin"))?;
    rpg.go("up")?;
    rpg.command(Take("shiny coin"))?;
    rpg.go("down")?;
    rpg.go(East)?;

    for coin in solve_coin_puzzle() {
        rpg.command(Use(&format!("{coin} coin")))?;
    }

    rpg.go(North)?;
    rpg.command(Take("teleporter"))?;
    rpg.command(Use("teleporter"))?;

    vm.run_interactive()?;

    Ok(())
}
