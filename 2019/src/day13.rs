
use crate::intcode::{Computer, OutputDevice, wire, CombinedDevice, WireInput, InputDevice, WireOutput};
use fxhash::{FxHashMap, FxBuildHasher};
use async_std::task;
use std::collections::HashMap;
use std::convert::From;
use std::fmt::{Display, Write};
use serde::export::Formatter;
use serde::export::fmt::Error;
use itertools::Itertools;
use std::time::{Instant, Duration};
use std::sync::Arc;

#[aoc_generator(day13)]
pub fn generator(input: &str) -> Vec<isize> {
    serde_scan::from_str_skipping(",", input).expect("input")
}

static NO_INPUT : Vec<isize> = Vec::new();

#[aoc(day13, part1)]
pub fn part1(input: &Vec<isize>) -> usize {
    let computer = Computer::new(Vec::from(&input[..]));

    let (signal, output_wire) = wire(3);

    let shared_state = Arc::new(async_std::sync::RwLock::new(GameState::default()));
    let runtime = task::spawn(run_computer(computer,&NO_INPUT[..], output_wire));
    let drawing = task::spawn(run_drawing(signal, shared_state));

    task::block_on(runtime);
    let (screen, _) : (FxHashMap<(isize,isize),Tile> , isize) = task::block_on(drawing);

    screen.values().filter(|x| **x == Tile::Block).count()
}

#[derive(Eq,PartialEq,Clone,Debug)]
struct GameState {
    ball_pos: isize,
    paddle_pos: isize,
    quit: bool
}

impl Default for GameState {
    fn default() -> Self {
        GameState { ball_pos: 0, paddle_pos: 0, quit: false }
    }
}

#[aoc(day13, part2)]
pub fn part2(input: &Vec<isize>) -> isize {
    let mut memory = Vec::from(&input[..]);
    memory[0] = 2;
    let computer = Computer::new(memory);

    let shared_state = Arc::new(async_std::sync::RwLock::new(GameState::default()));
    let (signal, output_wire) = wire(3);
    let (input_wire, controller) = wire(1);


    let runtime = task::spawn(run_computer(computer, input_wire, output_wire));
    let drawing = task::spawn(run_drawing(signal, shared_state.clone()));
    let bot = task::spawn(run_bot(controller, shared_state));

    eprintln!("Running game...");
    task::block_on(runtime);
    eprintln!("Waiting for drawing to shut down...");
    let (_, score) = task::block_on(drawing);
    eprintln!("Waiting for bot to shut down...");
    task::block_on(bot);

    score
}

async fn run_computer(mut computer: Computer, input: impl InputDevice+Send, output: impl OutputDevice+Send) -> () {
    computer.execute(&mut CombinedDevice::new(input, output)).await
}

async fn run_drawing(signal: WireInput, shared_state : Arc<async_std::sync::RwLock<GameState>>) -> (FxHashMap<(isize,isize), Tile>, isize) {
    #[allow(dead_code)]
    fn render(screen: &Screen, score: isize) -> () {
        println!("{}", screen);
        println!("Score: {}", score);
    }

    let mut screen :  Screen = Screen(FxHashMap::default());
    let mut score = 0isize;
    let mut last_refresh = Instant::now();

    while let Some(x) = signal.recv().await {
        let y = signal.recv().await.expect("received X coordinate, expecting Y coordinate");
        let tile_id = signal.recv().await.expect("received coordinates, expecting tile ID");
        if x == -1 && y == 0 {
            score = tile_id;
        } else {
            let tile = Tile::from(tile_id);
            if tile == Tile::Ball {
                let mut guard = async_std::sync::RwLock::write(&*shared_state).await;
                guard.ball_pos = x
            } else if tile == Tile::Paddle {
                let mut guard = async_std::sync::RwLock::write(&*shared_state).await;
                guard.paddle_pos = x
            }
            screen.0.insert((x, y), tile);
        }

        if last_refresh.elapsed() > Duration::from_millis(250) {
            last_refresh = Instant::now();
            //render(&screen, score)
        }
    }

    // render at least once
    //render(&screen, score);
    {
        let mut guard = async_std::sync::RwLock::write(&*shared_state).await;
        guard.quit = true;
    }
    (screen.0, score)
}

async fn run_bot(controller: WireOutput, shared_state : Arc<async_std::sync::RwLock<GameState>>) -> () {
    loop {
        let state : GameState = { async_std::sync::RwLock::read(&*shared_state).await.clone() };
        if state.quit {
            break
        }

        if !controller.is_full() {
            let next_input = (state.ball_pos - state.paddle_pos).signum();
            // eprintln!("Paddle: {}, Ball: {}, Joystick: {}", state.paddle_pos, state.ball_pos, next_input);
            controller.send(next_input).await;
        } else {
            task::sleep(Duration::from_millis(1)).await
        }
    }
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball
}
impl Tile {
    #[allow(dead_code)]
    fn id(&self) -> isize {
        isize::from(*self)
    }
}

impl From<isize> for Tile {
    fn from(x: isize) -> Self {
        match x {
            0 => Tile::Empty,
            1 => Tile::Wall,
            2 => Tile::Block,
            3 => Tile::Paddle,
            4 => Tile::Ball,
            _ => panic!("Unexpected tile ID {}", x)
        }
    }
}
impl From<Tile> for isize {
    fn from(x: Tile) -> Self {
        match x {
            Tile::Empty => 0,
            Tile::Wall => 1,
            Tile::Block => 2,
            Tile::Paddle => 3,
            Tile::Ball => 4,
        }
    }
}

struct Screen(HashMap<(isize,isize),Tile, FxBuildHasher>);
impl Display for Screen {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let (min_x,max_x) = self.0.keys()
            .map(|k| k.0)
            .minmax()
            .into_option().unwrap_or((0,0));
        let (min_y,max_y) = self.0.keys()
            .map(|k| k.1)
            .minmax()
            .into_option().unwrap_or((0,0));
        for y in (min_y-1) ..= (max_y+1) {
            for x in (min_x-1) ..= (max_x+1) {
                self.0.get(&(x,y)).unwrap_or(&Tile::Empty).fmt(f)?
            }
            f.write_str("\n")?
        }
        Ok(())
    }
}
impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Tile::Empty => f.write_char(' '),
            Tile::Wall => f.write_char('X'),
            Tile::Block => f.write_char('\u{2588}'),
            Tile::Paddle => f.write_char('='),
            Tile::Ball => f.write_char('\u{25CF}'),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_zero_blocks() {
        assert_eq!(part1(&vec![104, 1, 104, 2, 104, Tile::Empty.id(), 99]), 0);
        assert_eq!(part1(&vec![104, 1, 104, 2, 104, Tile::Wall.id(), 99]), 0);
        assert_eq!(part1(&vec![104, 1, 104, 2, 104, Tile::Paddle.id(), 99]), 0);
        assert_eq!(part1(&vec![104, 1, 104, 2, 104, Tile::Ball.id(), 99]), 0);
        assert_eq!(part1(&vec![99]), 0);
    }

    #[test]
    fn part1_one_block() {
        assert_eq!(part1(&vec![104, 1, 104, 2, 104, Tile::Block.id(), 99]), 1);
    }

    #[test]
    fn part1_block_overwritten() {
        assert_eq!(part1(&vec![
            104, 1, 104, 2, 104, Tile::Block.id(),
            104, 1, 104, 2, 104, Tile::Wall.id(),
            104, 2, 104, 2, 104, Tile::Block.id(),
            99]), 1);
    }

    #[test]
    fn part1_block_dimensions() {
        assert_eq!(part1(&vec![
            104, 1, 104, 1, 104, Tile::Block.id(),
            104, 1, 104, 2, 104, Tile::Block.id(),
            104, 2, 104, 1, 104, Tile::Block.id(),
            104, 2, 104, 2, 104, Tile::Block.id(),
            99]), 4);
    }

    #[test]
    fn part1_one_block_negative() {
        assert_eq!(part1(&vec![104, -1, 104, -2, 104, Tile::Block.id(), 99]), 1);
    }
}