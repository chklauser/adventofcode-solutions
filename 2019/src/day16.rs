use std::iter::{repeat, FromIterator};
use itertools::Itertools;
use std::str::FromStr;
use std::time::Instant;

#[aoc_generator(day16)]
pub fn generator(input: &str) -> Vec<i32> {
    input.lines()
        .next().expect("at least one line")
        .chars()
        .map(|c| c.to_digit(10).expect("only digits") as i32)
        .collect()
}

#[aoc(day16, part1)]
pub fn part1(input: &Vec<i32>) -> String {
    let mut buf = input.clone();

    one_hundred_phases(&mut buf, 0);

    render(&buf[0..8])
}

fn render(digits: &[i32]) -> String {
    digits.iter().map(|d| char::from('0' as u8 + *d as u8)).join("")
}

#[allow(unused)]
static BASE_PATTERN: [i32; 4] = [1, 0, -1, 0];

#[allow(unused)]
fn pat(pos: usize) -> impl Iterator<Item=i32> {
    BASE_PATTERN.iter().flat_map(move |x| repeat(*x).take(pos)).cycle()
}

fn phase(buf: &mut [i32], offset: usize) {
    let checkpoint = ((buf.len()-offset) / 1_00).max(1000);
    //eprintln!("buf: [_; {}]checkpoint: {}", (buf.len()-offset), checkpoint);
    for i in offset..buf.len() {
        buf[i] = quick_evolve(&buf, i + 1);
        if (i % checkpoint) == 0 {
          eprintln!("\t{}/{} ({:.1}%)",i-offset,buf.len()-offset, ((i-offset) as f64/(buf.len()-offset) as f64) * 100.0)
        }
    }
}

fn one_hundred_phases(buf: &mut[i32], offset: usize) {
    for i in 1..=100 {
        eprintln!("Phase {}", i);
        let start = Instant::now();
        phase(buf, offset);
        eprintln!("  dT={:?}", start.elapsed());
    }
}

fn read_signal(input: &[i32]) -> String {
    // repeat signal 10'000 times
    let mut buf: Vec<i32> = Vec::from_iter(input.into_iter().map(|x| *x).cycle().take(input.len()*10_000));
    let offset = usize::from_str(&render(&input[0..7])).expect("offset is a number");

    one_hundred_phases(&mut buf[..], offset);

    // read 8-digit number at indicated offset
    render(&buf[offset..offset+8])
}

#[aoc(day16, part2)]
fn part2(input: &Vec<i32>) -> String {
    read_signal(&input[..])
}

#[allow(unused)]
#[inline]
fn evolve(buf: &[i32], pos: usize) -> i32 {
    (buf.iter()
        .skip(pos - 1)
        .zip(pat(pos))
        .map(|(d, p)| (d * p))
        .sum::<i32>() % 10).abs()
}

fn quick_evolve(buf: &[i32], pos: usize) -> i32 {
    let mut i = pos - 1;
    let mut s = 0i32;
//    eprintln!("quick_evolve({}, {}) i={}", render(buf), pos, i);
    while i < buf.len() {
        // sum pos digits
        s += buf[i..(i+pos).min(buf.len())].iter().sum::<i32>();
//        eprintln!(" +ones[{}..{}] {}", i, (i+pos).min(buf.len()), render(&buf[i..(i+pos).min(buf.len())]));
        i += pos;
//        if i >= buf.len() {
//            break;
//        }

        // skip pos digits
//        eprintln!("zeroes[{}..{}] {}", i, (i+pos).min(buf.len()), render(&buf[i..(i+pos).min(buf.len())]));
        i += pos;
        if i >= buf.len() {
            break;
        }
        
        // sum pos digits & subtract
        s -= buf[i..(i+pos).min(buf.len())].iter().sum::<i32>();
//        eprintln!(" -ones[{}..{}] {}", i, (i+pos).min(buf.len()), render(&buf[i..(i+pos).min(buf.len())]));
        i += pos;
//        if i >= buf.len() {
//            break;
//        }

        // skip pos digits
//        eprintln!("zeroes[{}..{}] {}", i, (i+pos).min(buf.len()), render(&buf[i..(i+pos).min(buf.len())]));
        i += pos;
    }
    (s % 10i32).abs()
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_t1() {
        assert_eq!(generator("1234\n"), vec![1,2,3,4])
    }

    #[test]
    fn pattern_pos_1() {
        assert_eq!(pat(1).take(8).collect::<Vec<i32>>(), vec![1i32, 0, -1, 0, 1, 0, -1, 0]);
    }

    #[test]
    fn pattern_pos_2() {
        assert_eq!(pat(2).take(32).collect::<Vec<i32>>(), vec![
            1i32, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1, 0, 0,
            1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 0, 0, -1, -1, 0, 0
        ]);
    }

    #[test]
    fn evolve_pos_1() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(evolve(&buf, 1), 4);
    }

    #[test]
    fn evolve_pos_2() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(evolve(&buf, 2), 8);
    }

    #[test]
    fn evolve_pos_4() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(evolve(&buf, 4), 2);
    }

    #[test]
    fn evolve_pos_7() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(evolve(&buf, 7), 5);
    }

    #[test]
    fn evolve_pos_8() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(evolve(&buf, 8), 8);
    }

    #[test]
    fn quick_evolve_pos_1() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 1), 4);
    }

    #[test]
    fn quick_evolve_pos_2() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 2), 8);
    }

    #[test]
    fn quick_evolve_pos_3() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 3), 2);
    }

    #[test]
    fn quick_evolve_pos_4() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 4), 2);
    }

    #[test]
    fn quick_evolve_pos_5() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 5), 6);
    }

    #[test]
    fn quick_evolve_pos_6() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 6), 1);
    }

    #[test]
    fn quick_evolve_pos_7() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 7), 5);
    }

    #[test]
    fn quick_evolve_pos_8() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8i32];
        assert_eq!(quick_evolve(&buf, 8), 8);
    }


    #[test]
    fn quick_evolve_phase2() {
        let buf = [4,8,2,2,6,1,5, 8i32];
        assert_eq!(quick_evolve(&buf, 1), 3);
        assert_eq!(quick_evolve(&buf, 2), 4);
        assert_eq!(quick_evolve(&buf, 3), 0);
        assert_eq!(quick_evolve(&buf, 4), 4);
        assert_eq!(quick_evolve(&buf, 5), 0);
        assert_eq!(quick_evolve(&buf, 6), 4);
        assert_eq!(quick_evolve(&buf, 7), 3);
        assert_eq!(quick_evolve(&buf, 8), 8);
    }

    #[test]
    fn quick_evolve_phase3() {
        let buf = [3,4,0,4,0,4,3, 8i32];
        assert_eq!(quick_evolve(&buf, 1), 0);
        assert_eq!(quick_evolve(&buf, 2), 3);
        assert_eq!(quick_evolve(&buf, 3), 4);
        assert_eq!(quick_evolve(&buf, 4), 1);
        assert_eq!(quick_evolve(&buf, 5), 5);
        assert_eq!(quick_evolve(&buf, 6), 5);
        assert_eq!(quick_evolve(&buf, 7), 1);
        assert_eq!(quick_evolve(&buf, 8), 8);
    }

    #[test]
    fn quick_evolve_phase4() {
        let buf = [0,3,4,1,5,5,1, 8i32];
        assert_eq!(quick_evolve(&buf, 1), 0);
        assert_eq!(quick_evolve(&buf, 2), 1);
        assert_eq!(quick_evolve(&buf, 3), 0);
        assert_eq!(quick_evolve(&buf, 4), 2);
        assert_eq!(quick_evolve(&buf, 5), 9);
        assert_eq!(quick_evolve(&buf, 6), 4);
        assert_eq!(quick_evolve(&buf, 7), 9);
        assert_eq!(quick_evolve(&buf, 8), 8);
    }

    #[test]
    fn phase1() {
        let mut buf = vec![1, 2, 3, 4, 5, 6, 7, 8i32];
        phase(&mut buf[..], 0);
        assert_eq!(buf, vec![4,8,2,2,6,1,5,8])
    }


    #[test]
    fn phase2() {
        let mut buf = vec![4,8,2,2,6,1,5,8];
        phase(&mut buf[..], 0);
        assert_eq!(buf, vec![3,4,0,4,0,4,3,8])
    }


    #[test]
    fn phase3() {
        let mut buf = vec![3,4,0,4,0,4,3,8];
        phase(&mut buf[..], 0);
        assert_eq!(buf, vec![0,3,4,1,5,5,1,8])
    }

    #[test]
    fn part1_example1() {
        let mut buf = generator("80871224585914546619083218645595");
        one_hundred_phases(&mut buf, 0);
        assert_eq!(Vec::from(&buf[0..8]), vec![2,4,1,7,6,1,7,6])
    }


    #[test]
    fn part1_example2() {
        let mut buf = generator("19617804207202209144916044189917");
        one_hundred_phases(&mut buf, 0);
        assert_eq!(Vec::from(&buf[0..8]), vec![7,3,7,4,5,4,1,8])
    }


    #[test]
    fn part1_example3() {
        let mut buf = generator("69317163492948606335995924319873");
        one_hundred_phases(&mut buf, 0);
        assert_eq!(Vec::from(&buf[0..8]), vec![5,2,4,3,2,1,3,3])
    }

    #[test]
    fn part2_example1() {
        let input = generator("03036732577212944063491565474664");
        assert_eq!(read_signal(&input), String::from("84462026"));
    }

    #[test]
    fn part2_example2() {
        let input = generator("02935109699940807407585447034323");
        assert_eq!(read_signal(&input), String::from("78725270"));
    }

    #[test]
    fn part2_example3() {
        let input = generator("03081770884921959731165446850517");
        assert_eq!(read_signal(&input), String::from("53553731"));
    }
}
