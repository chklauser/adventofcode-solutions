extern crate aoc_runner;
#[macro_use]
extern crate aoc_runner_derive;
#[macro_use]
extern crate async_trait;
extern crate futures;

extern crate fxhash;
extern crate itertools;

pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;
pub mod day5;
pub mod day6;
pub mod day7;
pub mod day7_part2;
pub mod day7_part2_async;
pub mod day8;
pub mod day9;
pub mod day10;
pub mod day11;
pub mod day12;
pub mod day13;
pub mod day14;
pub mod day15;
pub mod day16;

mod intcode;

aoc_lib! { year = 2019 }