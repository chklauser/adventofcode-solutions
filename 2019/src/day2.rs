use std::iter::Iterator;
use crate::day2::Next::{Halt, Continue};

#[aoc_generator(day2)]
pub fn generator(input: &str) -> Vec<usize> {
    serde_scan::from_str_skipping(",", input).expect("input")
}

#[aoc(day2, part1)]
pub fn part1(input: &[usize]) -> usize {
    let noun = 12;
    let verb = 2;
    run(input, noun, verb)
}

fn run(program: &[usize], noun: usize, verb: usize) -> usize {
    let mut memory = Vec::from(program);
    memory[1] = noun;
    memory[2] = verb;
    execute(&mut memory);
    memory[0]
}

#[aoc(day2, part2)]
pub fn part2(program: &[usize]) -> usize {
    for (noun, verb) in (0..99).flat_map(|noun| (0..99).map(move |verb| (noun, verb))) {
        let result = run(program, noun, verb);
        if result == 19690720 {
            return 100 * noun + verb;
        }
    }
    panic!("Did not find a solution!");
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq)]
enum Next {
    Continue,
    Halt,
}

fn step(memory: &mut [usize], instruction_pointer: usize) -> Next {
    let op = memory[instruction_pointer];
    if op == 99 {
        return Halt;
    }

    let lhs = memory[memory[instruction_pointer + 1]];
    let rhs = memory[memory[instruction_pointer + 2]];

    let val = match op {
        1 => lhs + rhs,
        2 => lhs * rhs,
        _ => panic!("unexpected op code {} at position {}", memory[instruction_pointer], instruction_pointer)
    };

    memory[memory[instruction_pointer + 3]] = val;

    Continue
}

/// returns number of steps
fn execute(memory: &mut [usize]) -> i32 {
    let mut num_steps = 1;
    let mut instruction_pointer: usize = 0;

    while step(memory, instruction_pointer) == Continue {
        num_steps += 1;
        instruction_pointer += 4;
    }

    num_steps
}

//#[aoc(day1, part2)]
//pub fn part2(input: &[usize]) -> usize {
//    unimplemented!()
//}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::day2::Next::Continue;

    #[test]
    fn step_next_on_add() {
        let mut program = vec![1, 0, 0, 0];
        assert_eq!(step(&mut program, 0), Continue);
    }

    #[test]
    fn step_next_on_mul() {
        let mut program = vec![2, 0, 0, 0];
        assert_eq!(step(&mut program, 0), Continue);
    }


    #[test]
    fn step_halt_on_halt() {
        let mut program = vec![99, 0, 0, 0];
        assert_eq!(step(&mut program, 0), Halt);
    }

    #[test]
    fn step_example_1() {
        let mut program = vec![1, 0, 0, 0, 99];
        step(&mut program, 0);
        assert_eq!(program, vec![2, 0, 0, 0, 99]);
    }


    #[test]
    fn step_example_2() {
        let mut program = vec![2, 3, 0, 3, 99];
        step(&mut program, 0);
        assert_eq!(program, vec![2, 3, 0, 6, 99]);
    }

    #[test]
    fn step_example_3() {
        let mut program = vec![2, 4, 4, 5, 99, 0];
        step(&mut program, 0);
        assert_eq!(program, vec![2, 4, 4, 5, 99, 9801]);
    }

    #[test]
    fn execute_example_1() {
        let mut program = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        let num_steps = execute(&mut program);
        assert_eq!(num_steps, 3);
        assert_eq!(program, vec![30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn execute_example_2() {
        let mut program = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];
        let num_steps = execute(&mut program);
        assert_eq!(num_steps, 3);
        assert_eq!(program, vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50]);
    }
}