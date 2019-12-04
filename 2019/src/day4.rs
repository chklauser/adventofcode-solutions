use std::ops::{RangeInclusive};

type N = usize;

#[aoc_generator(day4)]
pub fn generator(input: &str) -> RangeInclusive<N> {
    let (start,end) = serde_scan::from_str_skipping("-", input).expect("range start-end");
    start..=end
}

fn has_exactly_double_digits(mut n: N) -> bool {
    if n == 0 {
        return false;
    }
    let mut last_digit = N::max_value();
    let mut count_matching = 0;
    while n > 0 {
        let digit = n%10;
        if last_digit == digit {
            count_matching += 1;
        } else {
            if count_matching == 1 {
                return true
            } else {
                count_matching = 0;
            }
        }

        // continue
        last_digit = digit;
        n = n/10;
    }
    count_matching == 1
}

fn has_double_digits(mut n: N) -> bool {
    if n == 0 {
        return true;
    }
    let mut last_digit = N::max_value();
    while n > 0 {
        let digit = n%10;
        if last_digit == digit {
            return true
        }
        last_digit = digit;
        n = n/10;
    }
    false
}

fn has_no_decreasing_digits(mut n: N) -> bool {
    if n == 0 {
        return true;
    }
    let mut last_digit = N::max_value();
    while n > 0 {
        let digit = n%10;
        if digit > last_digit {
            return false;
        }
        n = n/10;
        last_digit = digit;
    }
    true
}

#[allow(dead_code)]
fn int_pow(n: N, mut e: N) -> N{
    let mut r = 1;
    while e > 0 {
        r = r * n;
        e -= 1;
    }
    r
}

// It _should_ be possible to use impl Iterator in both positions, but for some reason, the compiler
// creates an exponentially sized type internally. Increasing the maximum type size by multiple
// orders of magnitude did not work around the type size error.
// Possibly related: https://github.com/rust-lang/rust/issues/64496
fn sequence_iter(head_seq: impl Iterator<Item=N>+'static) -> Box<dyn Iterator<Item=N>> {
    Box::new(head_seq.flat_map(move |head| {
        let start = head%10;
        (start..10).map(move |tail| head*10 + tail)
    }))
}

macro_rules! sequence {
    ($start:expr) => (($start)..10);
    ($_:expr, $($es:expr),+ ) => ( sequence_iter( sequence!( $($es),+ ) ) );
}

#[aoc(day4, part1, sequence_gen)]
pub fn part1_sequence_gen(input: &RangeInclusive<N>) -> N {
    let mut start = *input.start();
    while start > 10 {
        start = start / 10;
    }
    sequence!(start,start,start,start,start,start).filter(|n| input.contains(n)).filter(|n| has_double_digits(*n)).count()
}

#[aoc(day4, part1, naive)]
pub fn part1_naive(input: &RangeInclusive<N>) -> N {
    input.clone().filter(|n| has_double_digits(*n)).filter(|n| has_no_decreasing_digits(*n)).count()
}

#[aoc(day4, part2, sequence_gen)]
pub fn part2_sequence_gen(input: &RangeInclusive<N>) -> N {
    let mut start = *input.start();
    while start > 10 {
        start = start / 10;
    }
    sequence!(start,start,start,start,start,start).filter(|n| input.contains(n)).filter(|n| has_exactly_double_digits(*n)).count()
}

#[aoc(day4, part2, naive)]
pub fn part2_naive(input: &RangeInclusive<N>) -> N {
    input.clone().filter(|n| has_exactly_double_digits(*n)).filter(|n| has_no_decreasing_digits(*n)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_with_double_digits() {
        let inputs = vec![111111, 422350,000000,999999,1123456,123455];

        for input in inputs {
            assert!(has_double_digits(input), "has_double_digits({})", input);
        }
    }

    #[test]
    fn numbers_without_double_digits() {
        let inputs = vec![123456, 549831,121212];

        for input in inputs {
            assert!(!has_double_digits(input), "!has_double_digits({})", input);
        }
    }

    #[test]
    fn numbers_with_double_digits_exactly() {
        let inputs = vec![112233, 111122, 125567, 221111, 112211, 122111, 111221];

        for input in inputs {
            assert!(has_exactly_double_digits(input), "has_exactly_double_digits({})", input);
        }
    }

    #[test]
    fn numbers_without_double_digits_exactly() {
        let inputs = vec![123456, 549831,121212, 111111, 0, 123444];

        for input in inputs {
            assert!(!has_exactly_double_digits(input), "!has_exactly_double_digits({})", input);
        }
    }

    #[test]
    fn numbers_with_non_decreasing_digits() {
        let inputs = vec![123456, 0, 111111, 112233];
        for input in inputs {
            assert!(has_no_decreasing_digits(input), "has_no_decreasing_digits({})", input);
        }
    }

    #[test]
    fn numbers_with_decreasing_digits() {
        let inputs = vec![654321, 123450, 121212];
        for input in inputs {
            assert!(!has_no_decreasing_digits(input), "!has_no_decreasing_digits({})", input);
        }
    }

    #[test]
    fn single_digit_seq() {
        assert_eq!(sequence_iter(vec![0 as N].into_iter()).collect::<Vec<_>>(), vec![0,1,2,3,4,5,6,7,8,9]);
    }


    #[test]
    fn single_digit_seq_custom_start() {
        assert_eq!(sequence_iter(vec![5 as N].into_iter()).collect::<Vec<_>>(), vec![55,56,57,58,59]);
    }


    #[test]
    fn two_digit_seq_custom_start() {
        assert_eq!(sequence_iter(sequence_iter(vec![7 as N].into_iter())).collect::<Vec<_>>(), vec![777,778,779,788,789,799]);
    }

    #[test]
    fn three_digit_seq_custom_start() {
        assert_eq!(sequence_iter(sequence_iter(sequence_iter(vec![8 as N].into_iter()))).collect::<Vec<_>>(), vec![8888,8889,8899,8999]);
    }

    #[test]
    fn macro_equiv_manual_nesting_3() {
        let seed = vec![0 as N, 1 as N, 2 as N, 3 as N, 4 as N, 5 as N, 6 as N, 7 as N, 8 as N, 9 as N].into_iter();
        let manual_nesting: Vec<N> = sequence_iter(sequence_iter(seed)).collect();
        let macro_nesting: Vec<N> = sequence!(0,0,0).collect();
        assert_eq!(macro_nesting, manual_nesting);
    }

    #[test]
    fn macro_digit_length_2() {
        for num in sequence!(0,0) {
            assert!(num < 100, "num {} must be >0 <100", num);
        }
    }

    #[test]
    fn macro_digit_length_1() {
        for num in sequence!(0) {
            assert!(num < 10, "num {} must be >0 <10", num);
        }
    }

    #[test]
    fn part_1_example_1() {
        let input = 123456..=123470;
        assert_eq!(part1_sequence_gen(&input), 1);
    }

    #[test]
    fn part_2_example2() {
        let input = 123456..=123480;
        assert_eq!(part1_sequence_gen(&input), 2);
    }

    #[test]
    fn part_1_example_1_naive() {
        let input = 123456..=123470;
        assert_eq!(part1_naive(&input), 1);
    }

    #[test]
    fn part_2_example_2_naive() {
        let input = 123456..=123480;
        assert_eq!(part1_naive(&input), 2);
    }
}