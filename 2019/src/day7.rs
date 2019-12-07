use itertools::Itertools;

#[aoc_generator(day7)]
pub fn generator(input: &str) -> Vec<isize> {
    serde_scan::from_str_skipping(",", input).expect("input")
}

#[derive(Debug, Eq, PartialEq)]
enum Param {
    Location(usize),
    Immediate(isize),
}

impl Param {
    fn load(&self, memory: &[isize]) -> isize {
        match *self {
            Param::Location(idx) => memory[idx],
            Param::Immediate(val) => val
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Op {
    Add { lhs: Param, rhs: Param, dest: usize },
    Mul { lhs: Param, rhs: Param, dest: usize },
    Input { dest: usize },
    Output { val: Param },
    JumpTrue { cond: Param, dest: Param },
    JumpFalse { cond: Param, dest: Param },
    LessThan { lhs: Param, rhs: Param, dest: usize },
    Equals { lhs: Param, rhs: Param, dest: usize },
    Halt,
}

impl Op {
    fn decode(memory: &[isize], instruction_pointer: &mut usize) -> Op {
        let raw = memory[*instruction_pointer];
        let mut rem = raw / 100;
        let mut next_param = |idx: &mut usize| {
            let mode = rem % 10;
            rem = rem / 10;
            let param = match mode {
                0 => Param::Location(memory[*idx] as usize),
                1 => Param::Immediate(memory[*idx]),
                _ => panic!("Invalid parameter mode {} in op code {}", mode, raw)
            };
            *idx += 1;
            param
        };
        let next_dest = |idx: &mut usize| {
            let param = memory[*idx] as usize;
            *idx += 1;
            param
        };
        *instruction_pointer += 1;
        let op = match raw % 100 {
            1 => Op::Add { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_dest(instruction_pointer) },
            2 => Op::Mul { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_dest(instruction_pointer) },
            3 => Op::Input { dest: next_dest(instruction_pointer) },
            4 => Op::Output { val: next_param(instruction_pointer) },
            5 => Op::JumpTrue { cond: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            6 => Op::JumpFalse { cond: next_param(instruction_pointer), dest: next_param(instruction_pointer) },
            7 => Op::LessThan { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_dest(instruction_pointer) },
            8 => Op::Equals { lhs: next_param(instruction_pointer), rhs: next_param(instruction_pointer), dest: next_dest(instruction_pointer) },
            99 => Op::Halt,
            _ => panic!("Invalid opcode {}", raw)
        };
        op
    }
}

struct Amp {
    memory: Vec<isize>,
    parameter: isize,
    instruction_pointer: usize,
}

impl Amp {
    fn new(memory: Vec<isize>, parameter: isize) -> Amp {
        Amp { memory, parameter, instruction_pointer: 0 }
    }
    fn step<'a>(&mut self, op: &Op, input: &mut impl Iterator<Item=&'a isize>) -> Option<isize> {
        match op {
            Op::Add { lhs, rhs, dest } => self.memory[*dest] = lhs.load(&self.memory) + rhs.load(&self.memory),
            Op::Mul { lhs, rhs, dest } => self.memory[*dest] = lhs.load(&self.memory) * rhs.load(&self.memory),
            Op::Input { dest } => self.memory[*dest] = *input.next().expect("Program requires more input"),
            Op::Output { val } => return Some(val.load(&self.memory)),
            Op::JumpTrue { cond, dest } => if cond.load(&self.memory) != 0 { self.instruction_pointer = dest.load(&self.memory) as usize },
            Op::JumpFalse { cond, dest } => if cond.load(&self.memory) == 0 { self.instruction_pointer = dest.load(&self.memory) as usize },
            Op::LessThan { lhs, rhs, dest } => self.memory[*dest] = if lhs.load(&self.memory) < rhs.load(&self.memory) { 1 } else { 0 },
            Op::Equals { lhs, rhs, dest } => self.memory[*dest] = if lhs.load(&self.memory) == rhs.load(&self.memory) { 1 } else { 0 },
            Op::Halt => ()
        };
        None
    }
    fn execute<'a>(&mut self, input: &mut impl Iterator<Item=&'a isize>, output: &mut Vec<isize>) {
        loop {
            //eprintln!("MEM: {:?}", memory);

            // decode + advance instruction pointer
            let op = Op::decode(&self.memory, &mut self.instruction_pointer);
            //eprintln!(" OP: {:?}", op);
            if op == Op::Halt {
                return;
            }

            // execute operation
            if let Some(value) = self.step(&op, input) {
                output.push(value);
            }
        }
    }
    fn run(&mut self, input: isize) -> isize {
        let mut output = Vec::new();
        let input_seq = [self.parameter, input];
        self.execute(&mut input_seq.iter(), &mut output);
        *output.get(0).expect("amp to output a value")
    }
}

fn run_amp(program: &[isize], parameter: isize, input: isize) -> isize {
    let mut amp = Amp::new(Vec::from(program), parameter);
    amp.run(input)
}

#[aoc(day7, part1, seq)]
fn part1_seq(program: &Vec<isize>) -> isize {
    let parameters = (0isize..=4isize).permutations(5);
    parameters.map(|params| eval_params(&program, &params)).max().expect("should have one result")
}

fn eval_params(program: &[isize], params: &[isize]) -> isize {
    let a0 = run_amp(program, params[0], 0);
    let a1 = run_amp(program, params[1], a0);
    let a2 = run_amp(program, params[2], a1);
    let a3 = run_amp(program, params[3], a2);
    let a4 = run_amp(program, params[4], a3);
    a4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example1() {
        let program = vec![3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0];
        assert_eq!(part1_seq(&program), 43210);
    }
    #[test]
    fn part1_example1_eval_solution() {
        let program = vec![3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0];
        let params = [4,3,2,1,0];
        assert_eq!(eval_params(&program[..], &params), 43210);
    }


    #[test]
    fn part1_example2() {
        let program = vec![3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23, 99, 0, 0];
        assert_eq!(part1_seq(&program), 54321);
    }
    #[test]
    fn part1_example2_eval_solution() {
        let program = vec![3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23, 99, 0, 0];
        let params = [0,1,2,3,4];
        assert_eq!(eval_params(&program[..], &params), 54321);
    }


    #[test]
    fn part1_example3() {
        let program = vec![3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0];
        assert_eq!(part1_seq(&program), 65210);
    }
    #[test]
    fn part1_example3_eval_solution() {
        let program = vec![3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0];
        let params = [1,0,4,3,2];
        assert_eq!(eval_params(&program[..], &params), 65210);
    }
}