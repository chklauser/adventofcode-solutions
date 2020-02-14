use crate::intcode::{Computer, CombinedDevice};
use async_std::task;


#[aoc_generator(day17)]
pub fn generator(input: &str) -> Vec<isize> {
    serde_scan::from_str_skipping(",", input).expect("input")
}

const VERTICAL: [u8; 3] = [b'.', b'#', b'.'];
const HORIZONTAL: [u8; 3] = [b'#', b'#', b'#'];

#[aoc(day17, part1)]
pub fn part1(input: &Vec<isize>) -> isize {
    let mut computer = Computer::new(input.clone());
    let input: [isize; 0] = [];
    let output: Vec<isize> = Vec::new();
    let mut hal = CombinedDevice::new(&input[..], output);
    task::block_on(computer.execute(&mut hal));
    let maze =
        Maze::new(hal.output_device.into_iter().map(|d| d as u8).collect());

    //eprintln!("{}", String::from_utf8(maze.maze.clone()).expect("valid text"));
    maze.alignment()
}

#[aoc(day17, part2)]
pub fn part2(input: &Vec<isize>) -> isize {
    let mut memory = input.clone();
    memory[0] = 2;
    let mut computer = Computer::new(memory.clone());
    let input: Vec<isize> = "\
B,A,B,C,A,B,C,A,B,C
L,6,R,12,R,12,L,8
L,6,L,4,R,12
L,6,L,10,L,10,L,6
n
".bytes().map(|x| x as isize).collect();
    eprintln!("INPUT: {:?}", input);
    let output: Vec<isize> = Vec::new();
    let mut hal = CombinedDevice::new(&input[..], output);
    task::block_on(computer.execute(&mut hal));
    let image = String::from_utf8(hal.output_device.iter().filter(|b| **b < 127).map(|b| *b as u8).collect()).expect("utf8");
    eprintln!("Prorgam Display:\n{}", image);
    for x in hal.output_device.iter().filter(|b| **b >= 127) {
        eprintln!("OUTPUT: {}", *x)
    }
    *hal.output_device.last().expect("at least one output")
}

struct Maze {
    maze: Vec<u8>,
    stride: usize,
    // used for test assertions
    #[allow(unused)]
    height: isize
}

impl Maze {
    fn new(maze: Vec<u8>) -> Maze {
        let stride = 1 + maze.iter()
            .enumerate()
            .filter(|(_, c)| **c == 10)
            .map(|(i, _)| i)
            .next().expect("at least one line");
        let height = (maze.len() / stride) as isize + if maze.len() % stride == 0 { 0 } else { 1 };
        Maze { maze, stride, height }
    }

    fn idx(&self, coord: (isize,isize)) -> usize {
        let (x,y) = coord;
        (y as usize)*self.stride + (x as usize)
    }

    fn coord(&self, index: usize) -> (isize,isize) {
        let x = index % self.stride;
        let y = (index - x) / self.stride;
        (x as isize, y as isize)
    }

    fn alignment(&self) -> isize {
        let mut sum = 0isize;
        for i in 0..(self.maze.len() - 2*self.stride) {
            let (x, y) = self.coord(i);
            if x == 0 || x == self.stride as isize - 1 {
                continue
            }

            if &self.maze[i-1..=i+1] != VERTICAL {
                continue
            }
            let i3 = self.idx((x, y+2));
            if &self.maze[i3 -1..=i3 +1] != VERTICAL {
                continue
            }
            let i2 = self.idx((x, y+1));
            if &self.maze[i2 -1..=i2 +1] != HORIZONTAL {
                continue
            }

            let (cx, cy) = self.coord(i2);
            sum += cx * cy
        }
        sum
    }
}

impl<'a> From<&'a str> for Maze {
    fn from(input: &'a str) -> Self {
        Maze::new(input.bytes().collect())
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example1_dimensions() {
        let maze = Maze::from("..#..........
..#..........
#######...###
#.#...#...#.#
#############
..#...#...#..
..#####...^..");
        assert_eq!(maze.stride, 14, "stride");
        assert_eq!(maze.height, 7);
    }

    #[test]
    fn example1_dimensions_line_break() {
        let maze = Maze::from("..#..........
..#..........
#######...###
#.#...#...#.#
#############
..#...#...#..
..#####...^..
");
        assert_eq!(maze.stride, 14, "stride");
        assert_eq!(maze.height, 7);
    }

    #[test]
    fn part1_example1() {
        let maze = Maze::from("..#..........
..#..........
#######...###
#.#...#...#.#
#############
..#...#...#..
..#####...^..");
        assert_eq!(maze.alignment(), 76);
    }
}