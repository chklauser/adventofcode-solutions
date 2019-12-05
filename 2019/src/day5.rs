#[aoc_generator(day5)]
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

fn step(memory: &mut [isize], op: &Op, input: &mut impl Iterator<Item=isize>, instruction_pointer: &mut usize) -> Option<isize> {
    match op {
        Op::Add { lhs, rhs, dest } => memory[*dest] = lhs.load(memory) + rhs.load(memory),
        Op::Mul { lhs, rhs, dest } => memory[*dest] = lhs.load(memory) * rhs.load(memory),
        Op::Input { dest } => memory[*dest] = input.next().expect("Program requires more input"),
        Op::Output { val } => return Some(val.load(memory)),
        Op::JumpTrue { cond, dest } => if cond.load(memory) != 0 { *instruction_pointer = dest.load(memory) as usize },
        Op::JumpFalse { cond, dest } => if cond.load(memory) == 0 { *instruction_pointer = dest.load(memory) as usize },
        Op::LessThan { lhs, rhs, dest } => memory[*dest] = if lhs.load(memory) < rhs.load(memory) { 1 } else { 0 },
        Op::Equals { lhs, rhs, dest } => memory[*dest] = if lhs.load(memory) == rhs.load(memory) { 1 } else { 0 },
        Op::Halt => ()
    };
    None
}

fn execute(memory: &mut [isize], input: &mut impl Iterator<Item=isize>, output: &mut Vec<isize>) {
    let mut instruction_pointer = 0;
    loop {
        //eprintln!("MEM: {:?}", memory);

        // decode + advance instruction pointer
        let op = Op::decode(memory, &mut instruction_pointer);
        //eprintln!(" OP: {:?}", op);
        if op == Op::Halt {
            return;
        }

        // execute operation
        if let Some(value) = step(memory, &op, input, &mut instruction_pointer) {
            output.push(value);
        }
    }
}

#[aoc(day5, part1)]
pub fn part1(program: &Vec<isize>) -> isize {
    let mut memory = program.clone();
    let input_data = vec![1isize];
    let mut input = input_data.into_iter();
    let mut output = Vec::new();

    execute(&mut memory, &mut input, &mut output);

    for check in &output[0..output.len() - 1] {
        assert_eq!(*check, 0, "Diagnostic program assertion failed");
    }

    *output.last().expect("Expected output")
}

#[aoc(day5, part2)]
pub fn part2(program: &Vec<isize>) -> isize {
    let mut memory = program.clone();
    let input_data = vec![5isize];
    let mut input = input_data.into_iter();
    let mut output = Vec::new();

    execute(&mut memory, &mut input, &mut output);

    for check in &output[0..output.len() - 1] {
        assert_eq!(*check, 0, "Diagnostic program assertion failed");
    }

    *output.last().expect("Expected output")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_1() {
        let program = [1002, 4, 3, 4, 33];
        let mut cursor = 0;
        assert_eq!(Op::decode(&program, &mut cursor), Op::Mul { lhs: Param::Location(4), rhs: Param::Immediate(3), dest: 4 });
        assert_eq!(cursor, 4);
    }

    #[test]
    fn parse_example_2() {
        let program = [3, 0, 4, 0, 99, 1337];
        let mut cursor = 0;
        assert_eq!(Op::decode(&program, &mut cursor), Op::Input { dest: 0 });
        assert_eq!(cursor, 2);

        assert_eq!(Op::decode(&program, &mut cursor), Op::Output { val: Param::Location(0) });
        assert_eq!(cursor, 4);

        assert_eq!(Op::decode(&program, &mut cursor), Op::Halt);
        assert_eq!(cursor, 5);
    }

    #[test]
    fn echo() {
        let mut program = [3, 0, 4, 0, 99, 1337];
        let mut output = Vec::new();
        let input = vec![11isize, 22, 33, 44];
        let mut input_cursor = input.into_iter();
        execute(&mut program, &mut input_cursor, &mut output);
        assert_eq!(input_cursor.next(), Some(22));
        assert_eq!(output, vec![11isize]);
    }

    #[test]
    fn create_halt() {
        let mut program = [1002, 4, 3, 4, 33, 1337];
        let mut output = Vec::new();
        let input = vec![11isize, 22, 33, 44];
        let mut input_cursor = input.into_iter();
        execute(&mut program, &mut input_cursor, &mut output);
        assert_eq!(input_cursor.next(), Some(11));
        assert_eq!(output, vec![]);
    }

    #[test]
    fn example_program_day_2() {
        let mut program = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];
        let mut output = Vec::new();
        let input = vec![11isize, 22, 33, 44];
        let mut input_cursor = input.into_iter();
        execute(&mut program, &mut input_cursor, &mut output);
        assert_eq!(input_cursor.next(), Some(11));
        assert_eq!(output, vec![]);
        assert_eq!(program, vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50]);
    }

    #[test]
    fn part1_example() {
        let mut program = vec![3, 0, 1001, 0, -1, 2, 4, 2, 104, 1337, 99, 0xdead, 0xbeef, 0xdead, 0xbeef, 0xdead, 0xbeef, 0xdead, 0xbeef];
        let mut output = Vec::new();
        let input = vec![1isize, ];
        let mut input_cursor = input.into_iter();
        execute(&mut program, &mut input_cursor, &mut output);
        assert_eq!(input_cursor.next(), None);
        assert_eq!(output, vec![0, 1337]);
        assert_eq!(program, vec![1, 0, 0, 0, -1, 2, 4, 2, 104, 1337, 99, 0xdead, 0xbeef, 0xdead, 0xbeef, 0xdead, 0xbeef, 0xdead, 0xbeef]);
    }

    #[test]
    fn part2_larger_example_below() {
        let mut program = vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31,
            1106, 0, 36, 98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104,
            999, 1105, 1, 46, 1101, 1000, 1, 20, 4, 20, 1105, 1, 46, 98, 99];
        let mut output = Vec::new();
        let input = vec![5];
        let mut input_cursor = input.into_iter();
        execute(&mut program, &mut input_cursor, &mut output);
        assert_eq!(input_cursor.next(), None);
        assert_eq!(output, vec![999]);
    }

    #[test]
    fn part2_larger_example_equal() {
        let mut program = vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31,
            1106, 0, 36, 98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104,
            999, 1105, 1, 46, 1101, 1000, 1, 20, 4, 20, 1105, 1, 46, 98, 99];
        let mut output = Vec::new();
        let input = vec![8];
        let mut input_cursor = input.into_iter();
        execute(&mut program, &mut input_cursor, &mut output);
        assert_eq!(input_cursor.next(), None);
        assert_eq!(output, vec![1000]);
    }

    #[test]
    fn part2_larger_example_greater() {
        let mut program = vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31,
            1106, 0, 36, 98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104,
            999, 1105, 1, 46, 1101, 1000, 1, 20, 4, 20, 1105, 1, 46, 98, 99];
        let mut output = Vec::new();
        let input = vec![200];
        let mut input_cursor = input.into_iter();
        execute(&mut program, &mut input_cursor, &mut output);
        assert_eq!(input_cursor.next(), None);
        assert_eq!(output, vec![1001]);
    }
}