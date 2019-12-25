use itertools::Itertools;
use crate::intcode::{Computer, WireInput, WireOutput, wire, CombinedDevice, OutputSpy, InputDevice, OutputDevice};
use async_std::{
    task
};
use futures::join;
use std::future::Future;

// generator: see day7.rs
pub fn generator(input: &str) -> Vec<isize> {
    crate::day7::generator(input)
}

#[aoc(day7, part2, async)]
fn part2_async(program: &Vec<isize>) -> isize {
    let parameters = (5isize..=9isize).permutations(5);
    parameters.map(|params| eval_params(&program, &params)).max().expect("should have one result")
}

fn amp(program: &[isize]) -> Computer {
    Computer::new(Vec::from(program))
}

async fn run_to_completion(mut computer: Computer, input: impl InputDevice + Send, output: impl OutputDevice + Send) -> () {
    computer.execute(&mut CombinedDevice::new(input, output)).await
}

fn eval_params(program: &[isize], params: &[isize]) -> isize {
    task::block_on(eval_params_async(program, params))
}

fn init_wire(param: isize, initial: Option<isize>) -> (WireInput, WireOutput, impl Future<Output=()>) {
    let (i, o) = wire(if initial.is_some() { 2 } else { 1 });
    let o_initial = o.clone();
    let init = task::Builder::new()
        .name(format!("init-wire-{}-{}", param, if initial.is_some() { "init" } else { "inter" }))
        .spawn(async move {
            o_initial.send(param).await;
            if let Some(initial) = initial {
                o_initial.send(initial).await;
            }
        })
        .expect("spawned init_wire");
    (i,o,init)
}

async fn eval_params_async(program: &[isize], params: &[isize]) -> isize {
    let amp0 = amp(program);
    let amp1 = amp(program);
    let amp2 = amp(program);
    let amp3 = amp(program);
    let amp4 = amp(program);

    let (i0, o0, init0) = init_wire(params[1], None);
    let (i1, o1, init1) = init_wire(params[2], None);
    let (i2, o2, init2) = init_wire(params[3], None);
    let (i3, o3, init3) = init_wire(params[4], None);
    let (i4, o4, init4) = init_wire(params[0], Some(0));

    join!(init0, init1, init2, init3, init4);

    let loopback = OutputSpy::new(o4);
    let result = loopback.latest_value.clone();

    let t0 = task::Builder::new().name("amp0".to_owned()).spawn(run_to_completion(amp0, i4, o0)).expect("spawn amp0");
    let t1 = task::Builder::new().name("amp1".to_owned()).spawn(run_to_completion(amp1, i0, o1)).expect("spawn amp1");
    let t2 = task::Builder::new().name("amp2".to_owned()).spawn(run_to_completion(amp2, i1, o2)).expect("spawn amp2");
    let t3 = task::Builder::new().name("amp3".to_owned()).spawn(run_to_completion(amp3, i2, o3)).expect("spawn amp3");
    let t4 = task::Builder::new().name("amp4".to_owned()).spawn(run_to_completion(amp4, i3, loopback)).expect("spawn amp4");

    join!(t0,t1,t2,t3,t4);
    let result_guard = result.lock().expect("lock not poisoned");
    result_guard.expect("output value")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part2_example1() {
        let program = vec![3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26,
                           27, 4, 27, 1001, 28, -1, 28, 1005, 28, 6, 99, 0, 0, 5];
        assert_eq!(part2_async(&program), 139629729);
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
        assert_eq!(part2_async(&program), 18216);
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