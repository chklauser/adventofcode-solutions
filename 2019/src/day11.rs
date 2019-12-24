use fxhash::FxHashMap;
use std::fmt::Display;
use serde::export::Formatter;
use serde::export::fmt::Error;
use itertools::Itertools;
use crate::intcode::{InputDevice, OutputDevice};

#[aoc_generator(day11)]
pub fn generator(input: &str) -> Vec<isize> {
    serde_scan::from_str_skipping(",", input).expect("input")
}


#[derive(Debug, Eq, PartialEq)]
enum Param {
    Location(usize),
    Immediate(isize),
    Relative(isize)
}

impl Param {
    fn access<'a>(&'a self, memory: &'a [isize], relative_base: usize) -> &'a isize {
        fn protect(memory: &[isize], address: usize) -> &isize {
            if address >= memory.len() {
                &0
            } else {
                &memory[address]
            }
        }
        match *self {
            Param::Location(idx) => protect(&memory, idx),
            Param::Immediate(ref val) => val,
            Param::Relative(offset) => protect(&memory, (relative_base as isize + offset) as usize)
        }
    }
    fn access_mut<'a>(&self, memory: &'a mut Vec<isize>, relative_base: usize) -> &'a mut isize {
        fn protect(memory: &mut Vec<isize>, address: usize) -> &mut isize {
            if address >= memory.len() {
                memory.resize(address + 1, 0);
            }
            &mut memory[address]
        }
        match *self {
            Param::Location(idx) => protect(memory, idx),
            Param::Immediate(val) => panic!("Cannot write to immediate {}", val),
            Param::Relative(offset) => protect(memory, (relative_base as isize + offset) as usize)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Op {
    Add { lhs: Param, rhs: Param, dest: Param },
    Mul { lhs: Param, rhs: Param, dest: Param },
    Input { dest: Param },
    Output { val: Param },
    JumpTrue { cond: Param, dest: Param },
    JumpFalse { cond: Param, dest: Param },
    LessThan { lhs: Param, rhs: Param, dest: Param },
    Equals { lhs: Param, rhs: Param, dest: Param },
    RelBase { delta: Param },
    Halt,
}

impl Op {
    fn decode(memory: &[isize], instruction_pointer: &mut usize) -> Op {
        let raw = memory[*instruction_pointer];
        let mut rem = raw / 100;
        let mut next_param = |idx: &mut usize| {
            let mode = rem % 10;
            rem = rem / 10;
            let value = memory[*idx];
            let param = match mode {
                0 => Param::Location(value as usize),
                1 => Param::Immediate(value),
                2 => Param::Relative(value),
                _ => panic!("Invalid parameter mode {} in op code {}", mode, raw)
            };
            *idx += 1;
            param
        };
        *instruction_pointer += 1;
        let op = match raw % 100 {
            1 => Op::Add { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            2 => Op::Mul { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            3 => Op::Input { dest: next_param(instruction_pointer) },
            4 => Op::Output { val: next_param(instruction_pointer) },
            5 => Op::JumpTrue { cond: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            6 => Op::JumpFalse { cond: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            7 => Op::LessThan { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            8 => Op::Equals { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            9 => Op::RelBase { delta: next_param(instruction_pointer) },
            99 => Op::Halt,
            _ => panic!("Invalid opcode {}", raw)
        };
        op
    }
}

struct Computer {
    memory: Vec<isize>,
    instruction_pointer: usize,
    relative_base: usize,
    instr_cycles: usize,
    instr_yields: usize
}

#[derive(Eq, PartialEq, Debug)]
enum Yield {
    Halt,
    WaitForInput,
    OutputReady(isize),
    /// the boolean indicates whether the computer has consumed input (true = consumed)
    Continue(bool),
}

impl Yield {
    #[allow(unused)]
    fn unwrap_output(self) -> isize {
        match self {
            Yield::OutputReady(v) => v,
            x => panic!("Expected output, got {:?}", x)
        }
    }
    #[allow(unused)]
    fn expect_wait_for_input(self, msg: &str) {
        match self {
            Yield::WaitForInput => (),
            x => panic!("Expected to wait for input. {}. Got {:?} instead.", msg, x)
        }
    }
}

impl Computer {
    fn new(memory: Vec<isize>) -> Computer {
        Computer { memory, instruction_pointer: 0, relative_base: 0, instr_cycles: 0, instr_yields: 0 }
    }

    fn step<'a>(&mut self, op: &Op, input: Option<isize>) -> Yield {
        match op {
            Op::Add { lhs, rhs, dest } => self.store(dest, self.load(lhs) + self.load(rhs)),
            Op::Mul { lhs, rhs, dest } => self.store(dest, self.load(lhs) * self.load(rhs)),
            Op::Input { dest } => {
                if let Some(input) = input {
                    self.store(dest, input);
                    return Yield::Continue(true);
                } else {
                    return Yield::WaitForInput;
                }
            }
            Op::Output { val } => return Yield::OutputReady(self.load(val)),
            Op::JumpTrue { cond, dest } => if self.load(cond) != 0 { self.instruction_pointer = self.load(dest) as usize },
            Op::JumpFalse { cond, dest } => if self.load(cond) == 0 { self.instruction_pointer = self.load(dest) as usize },
            Op::LessThan { lhs, rhs, dest } => self.store(dest, if self.load(lhs) < self.load(rhs) { 1 } else { 0 }),
            Op::Equals { lhs, rhs, dest } => self.store(dest, if self.load(lhs) == self.load(rhs) { 1 } else { 0 }),
            Op::RelBase { delta: pos } => self.relative_base = (self.relative_base as isize + self.load(pos)) as usize,
            Op::Halt => return Yield::Halt
        };
        Yield::Continue(false)
    }

    fn load(&self, location: &Param) -> isize {
        *location.access(&self.memory[..], self.relative_base)
    }

    fn store(&mut self, location: &Param, value: isize) {
        *location.access_mut(&mut self.memory, self.relative_base) = value;
    }

    fn execute(&mut self, mut input: Option<isize>) -> Yield {
        self.instr_yields += 1;
        loop {
            //eprintln!("MEM: {:?}", memory);

            // decode + advance instruction pointer
            let current_instruction_pointer = self.instruction_pointer;
            let op = Op::decode(&self.memory, &mut self.instruction_pointer);
            self.instr_cycles += 1;
            //eprintln!(" OP: {:?}", op);
            if op == Op::Halt {
                return Yield::Halt;
            }

            // execute operation
            match self.step(&op, input) {
                y @ Yield::WaitForInput => {
                    // rewind to before the input operation so that we can re-try it afterwards.
                    self.instruction_pointer = current_instruction_pointer;
                    return y;
                }
                y @ Yield::Halt | y @ Yield::OutputReady(_) => return y,
                Yield::Continue(true) => input = None,
                Yield::Continue(false) => ()
            }
        }
    }
}

type Vector2 = (isize,isize);
fn add(lhs: &Vector2, rhs: &Vector2) -> Vector2 {
    (lhs.0 + rhs.0, lhs.1 + rhs.1)
}

#[derive(Copy,Eq,PartialEq,Debug,Clone)]
enum Color {
    Black,
    White
}
impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(match self {
            &Color::Black => "\u{00B7}",
            &Color::White => "\u{2588}"
        })
    }
}
impl From<Color> for isize {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => 0,
            Color::White => 1,
        }
    }
}
impl From<isize> for Color {
    fn from(v: isize) -> Self {
        match v {
            0 => Color::Black,
            1 => Color::White,
            x => panic!("invalid color {}", x)
        }
    }
}
fn rotate_clockwise(v: Vector2) -> Vector2 {
    //                                v.0
    //                                v.1
    // ccw rotation matrix:  0  -1    (0*v.0 - v.1)
    //                       1   0    (v.0 + 0*v.1)
    (-v.1, v.0)
}

fn rotate_counter_clockwise(v: Vector2) -> Vector2 {
    //                                v.0
    //                                v.1
    //  cw rotation matrix:  0   1    (0*v.0 + v.1)
    //                      -1   0    (-v.0 + 0*v.1)
    (v.1, -v.0)
}

#[derive(Clone,Eq,PartialEq,Debug)]
struct Hull {
    tiles: FxHashMap<Vector2, Color>,
    position: Vector2,
    speed: Vector2,
    state: RobotState
}
impl Hull {
    fn color_at(&self, position: &Vector2) -> Color {
        *self.tiles.get(position).unwrap_or(&Color::Black)
    }
}
impl Display for Hull {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let (min_x,max_x) = self.tiles.keys()
            .map(|k| k.0)
            .minmax()
            .into_option().unwrap_or((0,0));
        let (min_y,max_y) = self.tiles.keys()
            .map(|k| k.1)
            .minmax()
            .into_option().unwrap_or((0,0));
        for y in (min_y-1) ..= (max_y+1) {
            for x in (min_x-1) ..= (max_x+1) {
                if (x,y) == self.position {
                    f.write_str(match self.speed {
                        (0, -1) => "^",
                        (1, 0) => ">",
                        (0,1) => "v",
                        (-1,0) => "<",
                        _ => panic!("wtf is direction ({},{})",x,y)
                    })?
                } else {
                    self.color_at(&(x, y)).fmt(f)?
                }
            }
            f.write_str("\n")?
        }
        Ok(())
    }
}
impl Default for Hull {
    fn default() -> Self {
        Hull { tiles: FxHashMap::default(), position: (0,0), speed: (0, -1), state: RobotState::WaitingForColor }
    }
}
impl Iterator for &Hull {
    type Item = isize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(isize::from(self.color_at(&self.position)))
    }
}
#[derive(Eq,PartialEq,Debug,Copy,Clone)]
enum RobotState {
    WaitingForColor,
    WaitingForDirection
}
fn paint(program: &[isize]) -> Hull {
    let mut computer = Computer::new(Vec::from(program));
    let mut hull = Hull::default();
    paint_with(&mut computer, &mut hull);
    hull
}
fn paint_with(computer: &mut Computer, hull: &mut Hull) {
    loop {
        match (computer.execute((&*hull).next()), hull.state) {
            (Yield::Halt, _) => return,
            (Yield::WaitForInput, _) => panic!("Input is always available."),
            (Yield::OutputReady(1), RobotState::WaitingForColor) => {
                hull.tiles.insert(hull.position, Color::White);
                hull.state = RobotState::WaitingForDirection;
                //eprintln!("painted white: \n{}", hull);
            },
            (Yield::OutputReady(0), RobotState::WaitingForColor) => {
                hull.tiles.insert(hull.position, Color::Black);
                hull.state = RobotState::WaitingForDirection;
                //eprintln!("painted black: \n{}", hull);
            },
            (Yield::OutputReady(1), RobotState::WaitingForDirection) => {
                hull.speed = rotate_clockwise(hull.speed);
                hull.position = add(&hull.position, &hull.speed);
                hull.state = RobotState::WaitingForColor;
                //eprintln!("turned + moved right: \n{}", hull);
            },
            (Yield::OutputReady(0), RobotState::WaitingForDirection) => {
                hull.speed = rotate_counter_clockwise(hull.speed);
                hull.position = add(&hull.position, &hull.speed);
                hull.state = RobotState::WaitingForColor;
                //eprintln!("turned + moved left: \n{}", hull);
            },
            (Yield::Continue(_),_) => {},
            (y,s) => panic!("unexpected machine state. Robot: {:?}, Computer Yield: {:?}", y, s)
        }
    }
}
async fn paint_async(program: &[isize], initial_color: Color) -> Hull {
    let mut computer = crate::intcode::Computer::new(Vec::from(program));
    let mut hull = Hull::default();
    if initial_color != Color::Black {
        hull.tiles.insert((0,0), initial_color);
    }

    computer.execute(&mut hull).await;

    hull
}

#[async_trait]
impl InputDevice for Hull {
    async fn input(&mut self) -> isize {
        isize::from(self.color_at(&self.position))
    }
}
#[async_trait]
impl OutputDevice for Hull {
    async fn output(&mut self, value: isize) -> () {
        match self.state {
            RobotState::WaitingForColor => {
                self.tiles.insert(self.position, Color::from(value));
                self.state = RobotState::WaitingForDirection;
            },
            RobotState::WaitingForDirection => {
                if value == 1 {
                    self.speed = rotate_clockwise(self.speed);
                } else {
                    self.speed = rotate_counter_clockwise(self.speed);
                }
                self.position = add(&self.position, &self.speed);
                self.state = RobotState::WaitingForColor;
            }
        }
    }
}


#[aoc(day11, part1)]
pub fn part1(input: &Vec<isize>) -> usize {
    let hull = paint(&input[..]);
    hull.tiles.len()
}

#[aoc(day11, part1, async)]
pub fn part1_async(input: &Vec<isize>) -> usize {
    let hull = async_std::task::block_on(paint_async(&input[..], Color::Black));
    hull.tiles.len()
}

#[aoc(day11, part2)]
pub fn part2(input: &Vec<isize>) -> usize {
    let mut computer = Computer::new(input.clone());
    let mut hull = Hull::default();
    hull.tiles.insert((0,0), Color::White);
    paint_with(&mut computer, &mut hull);
    //println!("{}", hull);
    hull.tiles.len()
}

#[aoc(day11, part2, async)]
pub fn part2_async(input: &Vec<isize>) -> usize {
    let hull = async_std::task::block_on(paint_async(&input[..], Color::White));
    //println!("{}", hull);
    hull.tiles.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example1() {
        let program = vec![
            104, 1,
            104, 0,

            104, 0,
            104, 0,

            104, 1,
            104, 0,

            104, 1,
            104, 0,

            104, 0,
            104, 1,

            104, 1,
            104, 0,

            104, 1,
            104, 0,

            099
        ];
        let hull = paint(&program);
        println!("Hull:\n{}", hull);
        println!("Tiles: {:?}", hull.tiles);
        assert_eq!(hull.tiles.len(), 6);
    }
}