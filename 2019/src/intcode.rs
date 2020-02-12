#[allow(unused)]
use async_std::{
    prelude::*,
    task,
    sync,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Eq, PartialEq)]
enum Param {
    Location(usize),
    Immediate(isize),
    Relative(isize),
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

#[derive(Clone)]
pub(crate) struct Computer {
    memory: Vec<isize>,
    instruction_pointer: usize,
    relative_base: usize,
    instr_cycles: usize,
}

impl Computer {
    pub fn new(memory: Vec<isize>) -> Computer {
        Computer { memory, instruction_pointer: 0, relative_base: 0, instr_cycles: 0 }
    }

    fn load(&self, location: &Param) -> isize {
        *location.access(&self.memory[..], self.relative_base)
    }

    fn store(&mut self, location: &Param, value: isize) {
        *location.access_mut(&mut self.memory, self.relative_base) = value;
    }

    async fn step(&mut self, op: &Op, hal: &mut (impl Hal + Send)) -> () {
        match op {
            Op::Add { lhs, rhs, dest } => self.store(dest, self.load(lhs) + self.load(rhs)),
            Op::Mul { lhs, rhs, dest } => self.store(dest, self.load(lhs) * self.load(rhs)),
            Op::Input { dest } => self.store(dest, hal.input().await),
            Op::Output { val } => hal.output(self.load(val)).await,
            Op::JumpTrue { cond, dest } => if self.load(cond) != 0 { self.instruction_pointer = self.load(dest) as usize },
            Op::JumpFalse { cond, dest } => if self.load(cond) == 0 { self.instruction_pointer = self.load(dest) as usize },
            Op::LessThan { lhs, rhs, dest } => self.store(dest, if self.load(lhs) < self.load(rhs) { 1 } else { 0 }),
            Op::Equals { lhs, rhs, dest } => self.store(dest, if self.load(lhs) == self.load(rhs) { 1 } else { 0 }),
            Op::RelBase { delta: pos } => self.relative_base = (self.relative_base as isize + self.load(pos)) as usize,
            Op::Halt => panic!("cannot execute halt instruction")
        };
    }

    pub async fn execute(&mut self, hal: &mut (impl Hal + Send)) -> () {
        while hal.powered() {
            //eprintln!("MEM: {:?}", memory);

            // decode + advance instruction pointer
            let op = Op::decode(&self.memory, &mut self.instruction_pointer);
            self.instr_cycles += 1;
            //eprintln!(" OP: {:?}", op);
            if op == Op::Halt {
                return;
            }

            self.step(&op, hal).await;
        }
    }
}

#[async_trait]
pub(crate) trait InputDevice {
    async fn input(&mut self) -> isize;
}

#[async_trait]
pub(crate) trait OutputDevice {
    async fn output(&mut self, value: isize) -> ();
}

pub(crate) trait Hal: InputDevice + OutputDevice {
    fn powered(&mut self) -> bool { true }
}

pub(crate) struct CombinedDevice<I, O> {
    input_device: I,
    output_device: O,
}

impl<I, O> CombinedDevice<I, O> {
    pub(crate) fn new(input_device: I, output_device: O) -> CombinedDevice<I, O> {
        CombinedDevice { input_device, output_device }
    }
}
impl<I, O> Hal for CombinedDevice<I,O> where I: InputDevice+Send, O: OutputDevice+Send {}

#[async_trait]
impl<I, O> InputDevice for CombinedDevice<I, O> where I: InputDevice + Send, O: Send {
    async fn input(&mut self) -> isize {
        self.input_device.input().await
    }
}

#[async_trait]
impl<I, O> OutputDevice for CombinedDevice<I, O> where I: Send, O: OutputDevice + Send {
    async fn output(&mut self, value: isize) -> () {
        self.output_device.output(value).await
    }
}

#[async_trait]
impl InputDevice for &[isize] {
    async fn input(&mut self) -> isize {
        let value = self[0];
        *self = &self[1..];
        value
    }
}

#[async_trait]
impl OutputDevice for Vec<isize> {
    async fn output(&mut self, value: isize) -> () {
        self.push(value);
    }
}

pub(crate) struct OutputSpy<O> {
    device: O,
    pub latest_value: Arc<Mutex<Option<isize>>>,
}

impl<O> OutputSpy<O> {
    pub fn new(device: O) -> Self {
        OutputSpy { device, latest_value: Arc::new(Mutex::new(None)) }
    }
}

#[async_trait]
impl<O> OutputDevice for OutputSpy<O> where O: OutputDevice + Send {
    async fn output(&mut self, value: isize) -> () {
        *self.latest_value.lock().expect("not poisoned") = Some(value);
        self.device.output(value).await;
    }
}

pub(crate) type WireOutput = async_std::sync::Sender<isize>;
#[async_trait]
impl OutputDevice for WireOutput {
    async fn output(&mut self, value: isize) -> () {
        self.send(value).await
    }
}

pub(crate) type WireInput = async_std::sync::Receiver<isize>;
#[async_trait]
impl InputDevice for WireInput {
    async fn input(&mut self) -> isize {
        self.recv().await.expect("input over wire")
    }
}

pub(crate) fn wire(capacity: usize) -> (WireInput, WireOutput) {
    let (sender,receiver) = async_std::sync::channel::<isize>(capacity);
    (receiver,sender)
}
