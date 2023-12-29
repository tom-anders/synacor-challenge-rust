use itertools::Itertools;
use rand::seq::SliceRandom;

use crate::rpg::{Rpg, Direction, Exit, Room};
use crate::vm::Result;

#[derive(Debug, derive_more::Constructor)]
pub struct Maze<'a, 'b> {
    rpg: &'b mut Rpg<'a>,
}

impl Maze<'_, '_> {
    pub fn random_moves_until(&mut self, until: impl Fn(&Room) -> bool) -> Result<()> {
        let mut next = Direction::North;
        loop {
            let room = self.rpg.go(next.into())?;
            log::debug!("At {room:?}...");

            if until(&room) {
                return Ok(());
            }

            let dirs = room
                .exits
                .iter()
                .filter_map(|e| match e {
                    Exit::Dir(d) => Some(d),
                    _ => None,
                })
                .collect_vec();

            let next_dir = loop {
                let dir = **dirs.choose(&mut rand::thread_rng()).unwrap();
                if dir == Direction::East
                    && room.description.get(2).is_some_and(|line| {
                        line.starts_with("The passage to the east looks very dark")
                            || line.starts_with("The east passage appears very dark")
                    })
                {
                    log::debug!("Avoid grue...");
                    continue;
                }
                break dir;
            };

            next = next_dir;
        }
    }
}
