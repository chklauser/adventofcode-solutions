
use std::iter::Iterator;

#[aoc_generator(day1)]
pub fn generator(input: &str) -> Vec<i32> {
    serde_scan::from_str(input).expect("input")
}

#[aoc(day1, part1, iter_map_sum)]
pub fn part1(input: &[i32]) -> i32 {
    input.into_iter().map(|m| fuel_for_mass(*m)).sum()
}

fn fuel_for_mass(mass: i32) -> i32 {
    (mass / 3 - 2).max(0)
}

#[aoc(day1, part2, iter_map_loop_sum)]
pub fn part2(input: &[i32]) -> i32 {
    input.into_iter().map(|m| oomph_for_mass(*m)).sum()
}

fn oomph_for_mass(mass: i32) -> i32 {
    let mut remaining_mass = mass;
    let mut required_oomph = 0;
    while remaining_mass > 0 {
        let additional_oomph = fuel_for_mass(remaining_mass);
        remaining_mass = additional_oomph;
        required_oomph += additional_oomph;
    }
    required_oomph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_for_mass_examples_1() {
        assert_eq!(fuel_for_mass(12), 2);
    }
    #[test]
    fn fuel_for_mass_examples_2() {
        assert_eq!(fuel_for_mass(14), 2);
    }
    #[test]
    fn fuel_for_mass_examples_3() {
        assert_eq!(fuel_for_mass(1969), 654);
    }
    #[test]
    fn fuel_for_mass_examples_4() {
        assert_eq!(fuel_for_mass(100756), 33583);
    }
    #[test]
    fn negative_fuel_1() {
        assert_eq!(fuel_for_mass(0), 0);
    }
    #[test]
    fn negative_fuel_2() {
        assert_eq!(fuel_for_mass(1), 0);
    }
    #[test]
    fn negative_fuel_3() {
        assert_eq!(fuel_for_mass(2), 0);
    }
    #[test]
    fn negative_fuel_4() {
        assert_eq!(fuel_for_mass(3), 0);
    }
    #[test]
    fn negative_fuel_5() {
        assert_eq!(fuel_for_mass(4), 0);
    }
    #[test]
    fn negative_fuel_6() {
        assert_eq!(fuel_for_mass(5), 0);
    }
    #[test]
    fn negative_fuel_7() {
        assert_eq!(fuel_for_mass(6), 0);
    }
    #[test]
    fn minimum_fuel() {
        assert_eq!(fuel_for_mass(9), 1);
    }

    #[test]
    fn oomph_example_1() {
        assert_eq!(oomph_for_mass(14), 2);
    }

    #[test]
    fn oomph_example_2() {
        assert_eq!(oomph_for_mass(1969), 966);
    }

    #[test]
    fn oomph_example_3() {
        assert_eq!(oomph_for_mass(100756), 50346);
    }

    #[test]
    fn generator_fn() {
        assert_eq!(generator("57351\n149223\n142410\n"), vec![57351, 149223, 142410]);
    }
}