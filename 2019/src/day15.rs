use crate::intcode::{Computer, wire, InputDevice, OutputDevice, Hal};
use async_std::task;
use fxhash::FxHashMap;
use std::collections::VecDeque;
use std::mem::swap;

#[aoc_generator(day15)]
pub fn generator(input: &str) -> Vec<isize> {
    serde_scan::from_str_skipping(",", input).expect("input")
}

#[aoc(day15, part1)]
pub fn part1(input: &Vec<isize>) -> isize {
    let mut m: FxHashMap<(isize, isize), Tile> = FxHashMap::default();
    m.insert((0, 0), Tile::Starting(Computer::new(input.clone())));
    let mut unknowns = VecDeque::from(vec![(0, 0)]);

    while let Some(coord) = unknowns.pop_front() {
        let tile_entry = m.get_mut(&coord).expect("unknowns should already exist");

        // Compute the tile (if necessary)
        let mut tile = Tile::Known(Status::Wall);
        swap(tile_entry, &mut tile);
        let (status, state, distance) = match tile {
            Tile::Known(_) => panic!("unknowns should not contain known tiles"),
            Tile::Starting(c) => (Status::Empty, c, 0),
            Tile::Unknown(initial, command, distance) => {
                let (status, state) = step(&initial, command);
                (status, state, distance)
            },
            Tile::Pressurized(_) => panic!("already pressurized")
        };
        tile = Tile::Known(status);
        swap(tile_entry, &mut tile);

        // Act on current tile
        match status {
            Status::Oxygen => return distance,
            Status::Wall => (), // nothing to do
            Status::Empty => {
                for cmd in ALL_DIRECTIONS.iter() {
                    // Go in all directions, and add 'unknowns' if we haven't seen that tile yet
                    let next_coord = cmd.direction(coord);
                    m.entry(next_coord).or_insert_with(|| {
                        unknowns.push_back(next_coord);
                        Tile::Unknown(state.clone(), *cmd, distance + 1)
                    });
                }
            }
        }
    }

    panic!("Did not find oxygen system!");
}

#[aoc(day15, part2)]
pub fn part2(input: &Vec<isize>) -> isize {
    let mut m: FxHashMap<(isize, isize), Tile> = FxHashMap::default();
    m.insert((0, 0), Tile::Starting(Computer::new(input.clone())));

    // Explore the space (find oxygen as a side effect)
    let oxygen_coord = explore(&mut m);

    // Flood the space from the oxygen station
    *m.get_mut(&oxygen_coord).expect("oxygen tile to exist") = Tile::Pressurized(0);
    let mut vacuum = VecDeque::from(vec![oxygen_coord]);
    while let Some(coord) = vacuum.pop_front() {
        let tile_entry = m.get(&coord).expect("map should be known");

        if let Tile::Pressurized(time) = *tile_entry {
            for cmd in ALL_DIRECTIONS.iter() {
                let next_coord = cmd.direction(coord);
                let next_tile = m.get_mut(&next_coord).expect("neighbours of pressurized tiles to be known");
                if let Tile::Known(Status::Empty) = *next_tile {
                    *next_tile =  Tile::Pressurized(time + 1);
                    vacuum.push_back(next_coord);
                }
            }
        } else {
            panic!("expect pressurized tile in vacuum list")
        }
    }

    // find maximum pressurization time
    m.values().flat_map(|t| match t {
        Tile::Pressurized(time) => Some(*time),
        _ => None
    }).max().expect("maximum pressurization time")
}

fn explore(m: &mut FxHashMap<(isize, isize), Tile>) -> (isize, isize) {
    let mut oxygen_coord = None;
    let mut unknowns = VecDeque::from(vec![(0, 0)]);
    while let Some(coord) = unknowns.pop_front() {
        let tile_entry = m.get_mut(&coord).expect("unknowns should already exist");

        // Compute the tile (if necessary)
        let mut tile = Tile::Known(Status::Wall);
        swap(tile_entry, &mut tile);
        let (status, state, distance) = match tile {
            Tile::Known(_) => panic!("unknowns should not contain known tiles"),
            Tile::Starting(c) => (Status::Empty, c, 0),
            Tile::Unknown(initial, command, distance) => {
                let (status, state) = step(&initial, command);
                (status, state, distance)
            },
            Tile::Pressurized(_) => panic!("Oxygen not found yet, cannot be pressurized")
        };
        tile = Tile::Known(status);
        swap(tile_entry, &mut tile);

        // Act on current tile
        match status {
            Status::Oxygen => oxygen_coord = Some(coord),
            Status::Wall => (), // nothing to do
            Status::Empty => {
                for cmd in ALL_DIRECTIONS.iter() {
                    // Go in all directions, and add 'unknowns' if we haven't seen that tile yet
                    let next_coord = cmd.direction(coord);
                    m.entry(next_coord).or_insert_with(|| {
                        unknowns.push_back(next_coord);
                        Tile::Unknown(state.clone(), *cmd, distance + 1)
                    });
                }
            }
        }
    }

    oxygen_coord.expect("to find oxygen")
}

enum Tile {
    Starting(Computer),
    Unknown(Computer, Command, isize),
    Known(Status),
    Pressurized(isize)
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(isize)]
enum Status {
    Wall = 0isize,
    Empty = 1isize,
    Oxygen = 2isize,
}

impl From<isize> for Status {
    fn from(v: isize) -> Self {
        match v {
            0 => Status::Wall,
            1 => Status::Empty,
            2 => Status::Oxygen,
            x => panic!("{} is not a valid Status", x)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(isize)]
enum Command {
    North = 1isize,
    South = 2isize,
    West = 3isize,
    East = 4isize,
}

static ALL_DIRECTIONS: [Command; 4] = [Command::North, Command::South, Command::West, Command::East];

impl Command {
    fn direction(&self, origin: (isize, isize)) -> (isize, isize) {
        match *self {
            Command::North => (origin.0, origin.1 - 1),
            Command::South => (origin.0, origin.1 + 1),
            Command::West => (origin.0 - 1, origin.1),
            Command::East => (origin.0 + 1, origin.1)
        }
    }
}

struct PowerDownOnOutput<I, O> {
    input: I,
    output: O,
    powered: bool,
}

impl<I, O> PowerDownOnOutput<I, O> {
    fn new(input: I, output: O) -> PowerDownOnOutput<I, O> {
        PowerDownOnOutput { input, output, powered: true }
    }
}

#[async_trait]
impl<I, O> InputDevice for PowerDownOnOutput<I, O> where I: InputDevice + Send, O: Send {
    async fn input(&mut self) -> isize {
        self.input.input().await
    }
}

#[async_trait]
impl<I, O> OutputDevice for PowerDownOnOutput<I, O> where I: Send, O: OutputDevice + Send {
    async fn output(&mut self, value: isize) -> () {
        self.powered = false;
        self.output.output(value).await
    }
}

impl<I, O> Hal for PowerDownOnOutput<I, O> where I: InputDevice + Send, O: OutputDevice + Send {
    fn powered(&mut self) -> bool { self.powered }
}

fn step(initial: &Computer, command: Command) -> (Status, Computer) {
    let (input, controller) = wire(1);
    let (signal, output) = wire(1);
    let mut hal = PowerDownOnOutput::new(input, output);

    // Let the computer execute until it gets powered down.
    let mut computer = initial.clone();
    let next_computer_task = task::spawn(async move {
        computer.execute(&mut hal).await;
        computer
    });

    // Perform one command-response interaction, then turn the computer off
    let step_task = task::spawn(async move {
        controller.send(command as isize).await;
        Status::from(signal.recv().await.expect("signal"))
    });

    (task::block_on(step_task), task::block_on(next_computer_task))
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;
}