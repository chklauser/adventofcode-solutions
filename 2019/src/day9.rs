use std::iter::{once};

#[aoc_generator(day9)]
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
    // the boolean indicates whether input was consumed (true = consumed)
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

    fn execute_stream(&mut self, mut input: impl Iterator<Item=isize>, output: &mut Vec<isize>) {
        let mut current_input = None;
        loop {
            match self.execute(current_input) {
                Yield::Halt => return,
                Yield::WaitForInput => {
                    if let Some(value) = input.next() {
                        current_input = Some(value)
                    } else {
                        panic!("Insufficient input.");
                    }
                },
                Yield::OutputReady(value) => output.push(value),
                Yield::Continue(true) => current_input = None,
                Yield::Continue(false) => (),
            }
        }
    }
}

#[aoc(day9, part1)]
pub fn part1(image: &Vec<isize>) -> isize {
    let mut output = Vec::new();
    let mut computer = Computer::new(image.clone());
    computer.execute_stream(once(1), &mut output);
    eprintln!("STATS: cycles={}, yields={}", computer.instr_cycles, computer.instr_yields);
    output[0]
}

#[aoc(day9, part2)]
pub fn part2(image: &Vec<isize>) -> isize {
    let mut output = Vec::new();
    let mut computer = Computer::new(image.clone());
    computer.execute_stream(once(2), &mut output);
    eprintln!("STATS: cycles={}, yields={}", computer.instr_cycles, computer.instr_yields);
    output[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example1() {
        let program = vec![109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99];
        let input = vec![];
        let mut output = vec![];
        Computer::new(program.clone()).execute_stream(input.into_iter(), &mut output);
        assert_eq!(output, program);
    }

    #[test]
    fn part1_example2() {
        let program = vec![1102,34915192,34915192,7,4,7,99,0];
        let input = vec![];
        let mut output = vec![];
        Computer::new(program.clone()).execute_stream(input.into_iter(), &mut output);
        assert_eq!(output.len(), 1);
        assert!(output[0] >= 1_000_000_000_000_000, "Expected {} to be a 16 digit number.", output[0]);
    }

    #[test]
    fn part1_example3() {
        let program = vec![104,1125899906842624,99];
        let input = vec![];
        let mut output = vec![];
        Computer::new(program.clone()).execute_stream(input.into_iter(), &mut output);
        assert_eq!(output, vec![1125899906842624]);
    }
}