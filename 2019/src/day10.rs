use itertools::Itertools;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::cmp::Ordering;

type Scalar = isize;
type Point2 = (Scalar, Scalar);
type Vector2 = Point2;

#[aoc_generator(day10)]
pub fn generator(input: &str) -> Vec<Point2> {
    input.lines()
        .enumerate()
        .flat_map(|(y,line)|
            line.chars()
                .enumerate()
                .filter(|t| t.1 != '.')
                .map(move |(x,_)| (x as Scalar, y as Scalar))
        ).collect()
}

#[derive(PartialEq,Eq,Debug,Clone,Copy)]
enum Direction {
    North,
    NorthEast(isize,isize),
    East,
    SouthEast(isize,isize),
    South,
    SouthWest(isize,isize),
    West,
    NorthWest(isize,isize)
}
impl Direction {
    fn normalized(self) -> Self {
        fn norm(f: fn (isize,isize) -> Direction, x: isize, y: isize) -> Direction {
            let (nx, ny) = normalized_num(x,y);
            f(nx,ny)
        }
        match self {
            Direction::NorthEast(x,y) => norm(Direction::NorthEast, x, y),
            Direction::SouthEast(x,y) => norm(Direction::SouthEast, x, y),
            Direction::SouthWest(x,y) => norm(Direction::SouthWest, x, y),
            Direction::NorthWest(x,y) => norm(Direction::NorthWest, x, y),
            x => panic!("normalized not supported for {:?}", x)
        }
    }
}

impl Ord for Direction {
    fn cmp(&self, other: &Self) -> Ordering {
        use Direction::*;
        use Ordering::*;
        match (self,other) {
            (North,North) => Equal,
            (North,_) => Less,
            (NorthEast(_,_), North) => Greater,
            (NorthEast(x1,y1), NorthEast(x2,y2)) => (x1.abs() as f64/y1.abs() as f64).partial_cmp(&(x2.abs() as f64/y2.abs() as f64)).expect("a number"),
            (NorthEast(_,_), _) => Less,
            (East,North) => Greater,
            (East,NorthWest(_,_)) => Greater,
            (East,East) => Equal,
            (East,_) => Less,
            (SouthEast(_,_),North) => Greater,
            (SouthEast(_,_),NorthEast(_,_)) => Greater,
            (SouthEast(_,_),East) => Greater,
            (SouthEast(x1,y1), SouthEast(x2,y2)) => (y1.abs() as f64/x1.abs() as f64).partial_cmp(&(y2.abs() as f64/x2.abs() as f64)).expect("a number"),
            (SouthEast(_,_), _) => Less,
            (South,North) => Greater,
            (South,NorthEast(_,_)) => Greater,
            (South,East) => Greater,
            (South,SouthEast(_,_)) => Greater,
            (South, South) => Equal,
            (South, _) => Less,
            (SouthWest(x1, y1), SouthWest(x2, y2)) => (x1.abs() as f64/y1.abs() as f64).partial_cmp(&(x2.abs() as f64/y2.abs() as f64)).expect("a number"),
            (SouthWest(_,_),West) => Less,
            (SouthWest(_,_),NorthWest(_,_)) => Less,
            (SouthWest(_,_),_) => Greater,
            (West,West) => Equal,
            (West,NorthWest(_,_)) => Less,
            (West,_) => Greater,
            (NorthWest(x1,y1),NorthWest(x2,y2)) => (y1.abs() as f64/x1.abs() as f64).partial_cmp(&(y2.abs() as f64/x2.abs() as f64)).expect("a number"),
            (NorthWest(_,_),_) => Greater
        }
    }
}
impl PartialOrd for Direction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn normalized_num(on: isize, od: isize) -> (isize, isize) {
    // GCD https://en.wikipedia.org/wiki/Greatest_common_divisor (binary)
    let mut a = on.abs();
    let mut b = od.abs();
    let mut d = 0;

    #[inline(always)]
    fn is_even(x: Scalar) -> bool { (x&1) == 0 }

    while is_even(a) && is_even(b) {
        a = a>>1;
        b = b>>1;
        d += 1;
    }

    while a != b {
        if is_even(a) { a = a>>1; }
        else if is_even(b) { b = b>>1; }
        else if a > b { a = (a - b)>>1; }
        else { b = (b - a)>>1; }
    }
    let gcd = a<<d;
    (on/gcd, od/gcd)
}
impl From<Vector2> for Direction {
    fn from((x,y): (isize, isize)) -> Self {
        if x == 0 && y == 0 {
            panic!("NOPE ORIGIN");
        } else if x == 0 && y < 0 {
            Direction::North
        } else if x > 0 && y < 0 {
            Direction::NorthEast(x,y).normalized()
        } else if x > 0 && y == 0 {
            Direction::East
        } else if x > 0 && y > 0 {
            Direction::SouthEast(x,y).normalized()
        } else if x == 0 && y > 0 {
            Direction::South
        } else if x < 0 && y > 0 {
            Direction::SouthWest(x,y).normalized()
        } else if x < 0 && y == 0 {
            Direction::West
        } else if x < 0 && y < 0 {
            Direction::NorthWest(x,y).normalized()
        } else {
            panic!("9th direction {},{}", x, y);
        }
    }
}

fn visible_from(center: &Point2, input: &[Point2]) -> usize {
    input.iter()
        .filter(|p| *p != center)
        .map(|remote| {
            Direction::from((remote.0 - center.0, remote.1 - center.1))
        })
        .sorted()
        .dedup()
        .count()
}

fn angular_frequency(center: &Point2, input: &[Point2], freq: &mut BTreeMap<Direction, Vec<Point2>>) {
    input.iter()
        .filter(|p| *p != center)
        .map(|remote| {
            (Direction::from((remote.0 - center.0, remote.1 - center.1)), remote)
        }).for_each(move |(d,p)| {
        match freq.entry(d) {
            Entry::Vacant(v) => {
                v.insert(vec![*p]);
            },
            Entry::Occupied(mut v) => {
                v.get_mut().push(*p);
            },
        };
    });
}

fn best_part_1(input: &[Point2]) -> (&Point2, usize) {
    input.iter().map(|center| {
        (center, visible_from(center, input))
    }).max_by(|l,r| l.1.cmp(&r.1)).expect("Expected asteroids")
}

#[aoc(day10, part1)]
pub fn part1(input: &Vec<Point2>) -> usize {
    best_part_1(input).1
}


fn distance(lhs: &Vector2, rhs: &Vector2) -> isize {
    let d = (rhs.0 - lhs.0, rhs.1 - lhs.1);
    d.0*d.0 + d.1*d.1
}

pub fn part2(input: &[Point2]) -> isize {
    let (center,_) = best_part_1(input);
    eprintln!("Installing laser on {:?}", center);
    let mut freq = BTreeMap::new();
    angular_frequency(center, input, &mut freq);
    for (_, asteroids) in freq.iter_mut() {
        asteroids.sort_by(|l,r| distance(center, l).cmp(&distance(center, r)).reverse())
    }
    let mut to_remove: Vec<Direction> = Vec::new();
    let mut n = 0;
    while !freq.is_empty() {
        for (d, asteroids) in freq.iter_mut() {
            let asteroid = asteroids.pop().expect("direction should have asteroids");
            n += 1;
            eprintln!("vaporized asteroid #{} {:?} in direction {:?}", n, asteroid, d);
            if n == 200 {
                return asteroid.0 * 100 + asteroid.1;
            }

            if asteroids.is_empty() {
                eprintln!("    no more asteroids in direction {:?}", d);
                to_remove.push(d.clone());
            }
        }
        for d in to_remove.drain(..) {
            freq.remove(&d);
        }
    }

    panic!("There are not 200 asteroids to vaporize!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn direction_order() {
        let mut set = BTreeSet::new();
        set.insert(Direction::South);
        set.insert(Direction::East);
        set.insert(Direction::West);
        set.insert(Direction::North);
        set.insert(Direction::NorthWest(-1,-2));
        set.insert(Direction::NorthWest(-2,-1));
        set.insert(Direction::SouthWest(-2,1));
        set.insert(Direction::SouthWest(-1,1));
        set.insert(Direction::NorthEast(4,-3));
        set.insert(Direction::NorthEast(3,-4));
        set.insert(Direction::SouthEast(1,1));
        set.insert(Direction::SouthEast(1,2));
        set.insert(Direction::SouthEast(2,1));

        let sorted : Vec<Direction> = set.into_iter().collect();
        assert_eq!(sorted, vec![
            Direction::North,
            Direction::NorthEast(3,-4),
            Direction::NorthEast(4,-3),
            Direction::East,
            Direction::SouthEast(2,1),
            Direction::SouthEast(1,1),
            Direction::SouthEast(1,2),
            Direction::South,
            Direction::SouthWest(-1,1),
            Direction::SouthWest(-2,1),
            Direction::West,
            Direction::NorthWest(-2,-1),
            Direction::NorthWest(-1,-2)

        ]);
    }

    #[test]
    fn visible_from_1() {
        let field = generator(".#..#
.....
#####
....#
...##");
        assert_eq!(visible_from(&(1,  0), &field), 7);
        assert_eq!(visible_from(&(3, 4), &field), 8);
    }

    #[test]
    fn part1_example0() {
        let field = generator(".#..#
.....
#####
....#
...##");
        assert_eq!(part1(&field), 8);
    }

    #[test]
    fn part1_los_example() {
        let field = generator("#.........
...A......
...B..a...
.EDCG....a
..F.c.b...
.....c....
..efd.c.gb
.......c..
....f...c.
...e..d..c");
        assert_eq!(visible_from(&(0, 0), &field), 7);
    }

    #[test]
    fn part1_example1() {
        let field = generator("......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####");
        let (candidate, count) = best_part_1(&field);
        assert_eq!(candidate, &(5,  8));
        assert_eq!(count, 33);
    }

    #[test]
    fn part1_example1_expected() {
        let field = generator("......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####");
        assert_eq!(visible_from(&(5,  8), &field), 33);
    }

    #[test]
    fn part1_example2() {
        let field = generator("#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###.
");
        assert_eq!(part1(&field), 35);
    }

    #[test]
    fn part1_example3() {
        let field = generator(".#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..
");
        assert_eq!(part1(&field), 41);
    }

    #[test]
    fn part1_example4() {
        let field = generator(".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##
");
        assert_eq!(part1(&field), 210);
    }

    #[test]
    fn part1_example4_expected() {
        let field = generator(".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##
");
        assert_eq!(visible_from(&(11,  13), &field), 210);
    }

    #[test]
    fn part2_example4() {
        let field = generator(".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##
");
        assert_eq!(part2(&field), 802);
    }
}