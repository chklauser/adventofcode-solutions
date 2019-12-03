use std::str::FromStr;

#[aoc_generator(day3)]
pub fn generator(input: &str) -> (Vec<Segment>, Vec<Segment>) {
    let raw_lines: Vec<Vec<Segment>> = input
        .lines()
        .map(|line| serde_scan::from_str_skipping::<Vec<String>>(",", line)
            .expect("comma separated line directions")
            .into_iter()
            .map(Segment::from)
            .collect())
        .take(2)
        .collect();
    let mut d = raw_lines.into_iter();
    (d.next().expect("first line"), d.next().expect("second line"))
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Segment {
    U(isize),
    D(isize),
    L(isize),
    R(isize),
}

impl Segment {
    fn steps(&self, origin: Pos) -> Box<dyn Iterator<Item=Pos>> {
        match *self {
            Segment::U(distance) => Box::new(((origin.1 + 1)..=(origin.1 + distance)).map(move |y| (origin.0, y))),
            Segment::D(distance) => Box::new(((origin.1 - distance)..=(origin.1 - 1)).rev().map(move |y| (origin.0, y))),
            Segment::R(distance) => Box::new(((origin.0 + 1)..=(origin.0 + distance)).map(move |x| (x, origin.1))),
            Segment::L(distance) => Box::new(((origin.0 - distance)..=(origin.0 - 1)).rev().map(move |x| (x, origin.1)))
        }
    }
}

impl<'a> From<&'a str> for Segment {
    fn from(input: &'a str) -> Self {
        let distance = isize::from_str(&input[1..]).expect("Direction followed by number");
        let direction = input.chars().next().expect("must not be empty");
        match direction {
            'U' => Segment::U(distance),
            'D' => Segment::D(distance),
            'R' => Segment::R(distance),
            'L' => Segment::L(distance),
            _ => panic!("Unexpected direction")
        }
    }
}

impl From<String> for Segment {
    fn from(input: String) -> Self {
        Self::from(&input[..])
    }
}

type Pos = (isize, isize);

const STRIDE: isize = 6 * 4096;

fn coord(pos: Pos) -> usize {
    let x = pos.0 + STRIDE / 2;
    let y = pos.1 + STRIDE / 2;
    (x + y * STRIDE) as usize
}

fn norm_1(val: Pos) -> isize {
    val.0.abs() + val.1.abs()
}

fn min_norm_1(lhs: Pos, rhs: Pos) -> Pos {
    if norm_1(lhs) < norm_1(rhs) {
        lhs
    } else {
        rhs
    }
}

#[aoc(day3, part1, bool_grid)]
pub fn part1(input: &(Vec<Segment>, Vec<Segment>)) -> isize {
    let mut grid = vec![false; (STRIDE * STRIDE) as usize];

    let mut cursor = (0, 0);
    for segment in &input.0[..] {
        for step in segment.steps(cursor) {
            cursor = step;
            if cursor != (0, 0) {
                grid[coord(cursor)] = true;
            }
        }
    }

    let mut closest = (STRIDE, STRIDE);
    cursor = (0, 0);
    for segment in &input.1[..] {
        for step in segment.steps(cursor) {
            cursor = step;
            if cursor != (0, 0) {
                if grid[coord(cursor)] {
                    closest = min_norm_1(closest, cursor);
                }
            }
        }
    }

    norm_1(closest)
}

type Distance = u16;

#[aoc(day3, part2, dist_grid)]
pub fn part2(input: &(Vec<Segment>, Vec<Segment>)) -> Distance {
    let mut grid = vec![0 as Distance; (STRIDE * STRIDE) as usize];

    let mut cursor = (0, 0);
    let mut distance: Distance = 0;
    for segment in &input.0[..] {
        for step in segment.steps(cursor) {
            cursor = step;
            distance += 1;
            let space = &mut grid[coord(cursor)];
            if cursor != (0, 0) && *space == 0 {
                *space = distance;
            }
        }
    }

    let mut closest = Distance::max_value();
    cursor = (0, 0);
    distance = 0;
    for segment in &input.1[..] {
        for step in segment.steps(cursor) {
            cursor = step;
            distance += 1;
            let crossing_distance = grid[coord(cursor)];
            if crossing_distance > 0 {
                let total_distance = distance + crossing_distance;
                closest = closest.min(total_distance);
            }
        }
    }

    closest
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::day3::Segment::{D, L, U, R};

    #[test]
    fn parse_segment() {
        assert_eq!(Segment::from("R8"), R(8));
        assert_eq!(Segment::from("U5"), U(5));
        assert_eq!(Segment::from("L5"), L(5));
        assert_eq!(Segment::from("D3"), D(3));
    }

    #[test]
    fn parse_short_line() {
        let input = "R8,U5,L5,D3\nR8,U5,L5,D3\n";
        let (line1, line2) = generator(input);

        assert_eq!(line1, vec![R(8), U(5), L(5), D(3)]);
        assert_eq!(line2, vec![R(8), U(5), L(5), D(3)]);
    }

    #[test]
    fn grid_step_3() {
        assert_eq!(U(3).steps((6, 6)).collect::<Vec<_>>(), vec![(6, 7), (6, 8), (6, 9)]);
        assert_eq!(D(3).steps((6, 6)).collect::<Vec<_>>(), vec![(6, 5), (6, 4), (6, 3)]);
        assert_eq!(L(3).steps((6, 6)).collect::<Vec<_>>(), vec![(5, 6), (4, 6), (3, 6)]);
        assert_eq!(R(3).steps((6, 6)).collect::<Vec<_>>(), vec![(7, 6), (8, 6), (9, 6)]);
    }

    #[test]
    fn part_1_example_1() {
        let input = "R8,U5,L5,D3\nU7,R6,D4,L4\n";
        let lines = generator(input);
        assert_eq!(part1(&lines), 6);
    }

    #[test]
    fn part_1_example_2() {
        let input = "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83\n";
        let lines = generator(input);
        assert_eq!(part1(&lines), 159);
    }

    #[test]
    fn part_1_example_3() {
        let input = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7\n";
        let lines = generator(input);
        assert_eq!(part1(&lines), 135);
    }

    #[test]
    fn part_2_example_1() {
        let input = "R8,U5,L5,D3\nU7,R6,D4,L4\n";
        let lines = generator(input);
        assert_eq!(part2(&lines), 30);
    }

    #[test]
    fn part_2_example_2() {
        let input = "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83\n";
        let lines = generator(input);
        assert_eq!(part2(&lines), 610);
    }

    #[test]
    fn part_2_example_3() {
        let input = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7\n";
        let lines = generator(input);
        assert_eq!(part2(&lines), 410);
    }
}