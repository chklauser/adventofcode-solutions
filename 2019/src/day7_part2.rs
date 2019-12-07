use itertools::Itertools;

// generator: see day7.rs
pub fn generator(input: &str) -> Vec<isize> {
    crate::day7::generator(input)
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
    instruction_pointer: usize,
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
    fn unwrap_output(self) -> isize {
        match self {
            Yield::OutputReady(v) => v,
            x => panic!("Expected output, got {:?}", x)
        }
    }
    fn expect_wait_for_input(self, msg: &str) {
        match self {
            Yield::WaitForInput => (),
            x => panic!("Expected to wait for input. {}. Got {:?} instead.", msg, x)
        }
    }
}

impl Amp {
    fn new(memory: Vec<isize>, parameter: isize) -> Amp {
        let mut amp = Amp { memory, instruction_pointer: 0 };
        amp.execute(Some(parameter)).expect_wait_for_input("Should consume amp parameter.");
        amp
    }
    fn step<'a>(&mut self, op: &Op, input: Option<isize>) -> Yield {
        match op {
            Op::Add { lhs, rhs, dest } => self.memory[*dest] = lhs.load(&self.memory) + rhs.load(&self.memory),
            Op::Mul { lhs, rhs, dest } => self.memory[*dest] = lhs.load(&self.memory) * rhs.load(&self.memory),
            Op::Input { dest } => {
                if let Some(input) = input {
                    self.memory[*dest] = input;
                    return Yield::Continue(true);
                } else {
                    return Yield::WaitForInput;
                }
            }
            Op::Output { val } => return Yield::OutputReady(val.load(&self.memory)),
            Op::JumpTrue { cond, dest } => if cond.load(&self.memory) != 0 { self.instruction_pointer = dest.load(&self.memory) as usize },
            Op::JumpFalse { cond, dest } => if cond.load(&self.memory) == 0 { self.instruction_pointer = dest.load(&self.memory) as usize },
            Op::LessThan { lhs, rhs, dest } => self.memory[*dest] = if lhs.load(&self.memory) < rhs.load(&self.memory) { 1 } else { 0 },
            Op::Equals { lhs, rhs, dest } => self.memory[*dest] = if lhs.load(&self.memory) == rhs.load(&self.memory) { 1 } else { 0 },
            Op::Halt => return Yield::Halt
        };
        Yield::Continue(false)
    }
    fn execute<'a>(&mut self, mut input: Option<isize>) -> Yield {
        loop {
            //eprintln!("MEM: {:?}", memory);

            // decode + advance instruction pointer
            let current_instruction_pointer = self.instruction_pointer;
            let op = Op::decode(&self.memory, &mut self.instruction_pointer);
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

#[aoc(day7, part2, seq)]
fn part2_seq(program: &Vec<isize>) -> isize {
    let parameters = (5isize..=9isize).permutations(5);
    parameters.map(|params| eval_params(&program, &params)).max().expect("should have one result")
}

fn eval_params(program: &[isize], params: &[isize]) -> isize {
    let mut amp0 = Amp::new(Vec::from(program), params[0]);
    let mut amp1 = Amp::new(Vec::from(program), params[1]);
    let mut amp2 = Amp::new(Vec::from(program), params[2]);
    let mut amp3 = Amp::new(Vec::from(program), params[3]);
    let mut amp4 = Amp::new(Vec::from(program), params[4]);

    let mut a4 = 0isize;
    loop {
        let a0;
        match amp0.execute(Some(a4)) {
            Yield::OutputReady(x) => a0 = x,
            Yield::Halt => return a4,
            x => panic!("Amp0 unexpectedly returned {:?}", x)
        }
        let a1 = amp1.execute(Some(a0)).unwrap_output();
        let a2 = amp2.execute(Some(a1)).unwrap_output();
        let a3 = amp3.execute(Some(a2)).unwrap_output();
        a4 = amp4.execute(Some(a3)).unwrap_output();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part2_example1() {
        let program = vec![3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26,
                           27, 4, 27, 1001, 28, -1, 28, 1005, 28, 6, 99, 0, 0, 5];
        assert_eq!(part2_seq(&program), 139629729);
    }

    #[test]
    fn part2_example1_eval_solution() {
        let program = vec![3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26,
                           27, 4, 27, 1001, 28, -1, 28, 1005, 28, 6, 99, 0, 0, 5];
        let params = [9, 8, 7, 6, 5];
        assert_eq!(eval_params(&program[..], &params), 139629729);
    }


    #[test]
    fn part2_example2() {
        let program = vec![3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
                           -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
                           53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10];
        assert_eq!(part2_seq(&program), 18216);
    }

    #[test]
    fn part2_example2_eval_solution() {
        let program = vec![3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
                           -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
                           53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10];
        let params = [9, 7, 8, 5, 6];
        assert_eq!(eval_params(&program[..], &params), 18216);
    }
}