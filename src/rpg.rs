use std::{fmt::Display, str::FromStr};

use crate::vm::{Result, Vm};
use itertools::Itertools;
use parse_display::{Display, FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromStr, Display)]
#[display(style = "snake_case")]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromStr, derive_more::From)]
#[display("{0}")]
pub enum Exit {
    Dir(Direction),
    Other(String),
}

impl Display for Exit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Exit::Dir(dir) => dir.to_string(),
                Exit::Other(other) => other.to_string(),
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Room {
    pub description: Vec<String>,
    pub items: Vec<String>,
    pub exits: Vec<Exit>,
}

#[derive(Debug, derive_more::Constructor)]
pub struct Rpg<'a> {
    vm: &'a mut Vm,
}

impl Rpg<'_> {
    pub fn use_(&mut self, item: impl ToString) -> Result<()> {
        let item = item.to_string();
        self.vm.run_commands(
            [
                format!("use {item}").as_str(),
            ],
            &mut std::io::stdout(),
        )?;
        Ok(())
    }

    pub fn take(&mut self, item: impl ToString) -> Result<()> {
        let item = item.to_string();
        self.vm.run_commands(
            [
                format!("take {item}").as_str(),
                format!("look {item}").as_str(),
            ],
            &mut std::io::stdout(),
        )?;
        Ok(())
    }

    pub fn go(&mut self, exit: Exit) -> Result<Room> {
        let mut room = Vec::new();
        self.vm
            .run_commands([format!("go {exit}").as_str()], &mut room)?;

        let room_str: String = room.iter().map(|c| *c as char).collect();

        let room_lines = room_str
            .lines()
            .filter_map(|line| (!line.is_empty()).then_some(line.to_string()))
            .collect_vec();

        let room = room_lines
            .split(|line| {
                line == "Things of interest here:"
                    || lazy_regex::regex_is_match!(r"There (are|is) \d+ exit", line)
            })
            .collect_vec();

        let description = room[0].to_vec();

        let has_items = room.len() == 3;

        let items = if has_items {
            room[1]
                .iter()
                .filter_map(|s| s.starts_with("- ").then_some(s[2..].to_string()))
                .collect()
        } else {
            vec![]
        };

        let exits = room[if has_items { 2 } else { 1 }]
            .iter()
            .filter_map(|s| {
                s.starts_with("- ")
                    .then_some(Exit::from_str(&s[2..]).unwrap())
            })
            .collect();

        Ok(Room {
            description,
            items,
            exits,
        })
    }
}
