use std::collections::hash_map::Entry;
use fxhash::FxHashMap;

#[aoc_generator(day6)]
pub fn generator(input: &str) -> Vec<(String, String)> {
    input.lines().map(|l| {
        let mut parts = l.split(")");
        (parts.next().expect("lhs of pair").to_owned(), parts.next().expect("rhs of pair").to_owned())
    }).collect()
}

#[aoc(day6, part1, recursive)]
pub fn part1_rec(orbits: &Vec<(String, String)>) -> usize {
    // Create adjacency map
    let mut map: FxHashMap<&str, Vec<&str>> = FxHashMap::default();
    for orbit in orbits {
        let center = &orbit.0[..];
        let body = &orbit.1[..];

        // Insert the LHS containing the RHS as an orbiting body
        match map.entry(center) {
            Entry::Occupied(mut bodies) => {
                bodies.get_mut().push(body)
            }
            Entry::Vacant(bodies) => {
                let xs = bodies.insert(Vec::new());
                xs.push(body);
            }
        }

        // Insert the RHS with no orbits (so far)
        if let Entry::Vacant(bodies) = map.entry(body) {
            bodies.insert(Vec::new());
        }
    }

    count_orbits_rec(&map, "COM", 0)
}

fn count_orbits_rec(map: &FxHashMap<&str, Vec<&str>>, body: &str, depth: usize) -> usize {
    let orbiting_bodies = &map.get(body).unwrap_or_else(|| panic!("map should contain '{}'", body))[..];
    depth + orbiting_bodies.iter().map(|b| count_orbits_rec(map, *b, depth + 1usize)).sum::<usize>()
}

#[derive(Eq,PartialEq,Debug)]
struct Planet<'a> {
    name: &'a str,
    distance: isize,
}

#[aoc(day6,part2)]
pub fn part2(orbits: &Vec<(String, String)>) -> isize {
    // Create reverse map
    let mut map = FxHashMap::default();
    for orbit in orbits {
        map.insert(&orbit.1[..], Planet { name: &orbit.0[..], distance: -1 });
    }

    // Escape path from YOU all the way to COM
    let max_distance = escape_from_you(&mut map);

    //eprintln!("MAP: {:#?}", map);

    // Escape path from SAN up to previous escape path
    escape_from_santa(&mut map, max_distance)
}

fn escape_from_santa(map: &mut FxHashMap<&str, Planet>, max_escape_distance: isize) -> isize {
    let mut current_name = "SAN";
    let mut current_distance = 0isize;
    while current_name != "COM" {
        let next_planet = map.get(current_name).unwrap_or_else(|| panic!("cannot find parent body of {}", current_name));
        current_name = next_planet.name;
        if next_planet.distance >= 0 {
            // found escape path from YOU
            //eprintln!("Found escape path on planet {}, marked with a red {}. own distance {}", next_planet.name, next_planet.distance, current_distance);
            return current_distance + next_planet.distance - 2;
        } else {
            // continue escape
            //eprintln!("didn't find escape path yet on planet {} (d:{}), distance travelled is {}", next_planet.name,next_planet.distance, current_distance);
            current_distance += 1;
        }
    }
    return current_distance + max_escape_distance - 2;
}

fn escape_from_you(map: &mut FxHashMap<&str, Planet>) -> isize {
    let mut current_name = "YOU";
    let mut current_distance = 0isize;
    while current_name != "COM" {
        let mut next_planet = map.get_mut(current_name).unwrap_or_else(|| panic!("cannot find parent body of {}", current_name));
        next_planet.distance = current_distance;
        //eprintln!("Mark planet {} with red {}", next_planet.name, current_distance);
        current_distance += 1;
        current_name = next_planet.name;
    }
    current_distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example_1() {
        let orbits = generator("COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
");
        assert_eq!(part1_rec(&orbits), 42);
    }

    #[test]
    fn part2_example_1() {
        let orbits = generator("COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN
");
        assert_eq!(part2(&orbits), 4);
    }

    #[test]
    fn part2_mini_0() {
        //     YOU
        //    /
        // COM
        //    \
        //     SAN
        let orbits = generator("COM)YOU
COM)SAN
");
        assert_eq!(part2(&orbits), 0);
    }
    #[test]
    fn part2_mini_1l() {
        //     M - YOU
        //    /
        // COM
        //    \
        //     SAN
        let orbits = generator("COM)M
M)YOU
COM)SAN
");
        assert_eq!(part2(&orbits), 1);
    }
    #[test]
    fn part2_mini_1r() {
        //     M - SAN
        //    /
        // COM
        //    \
        //     YOU
        let orbits = generator("COM)YOU
COM)M
M)SAN
");
        assert_eq!(part2(&orbits), 1);
    }
    #[test]
    fn part2_mini_0_deep() {
        //         YOU
        //        /
        // COM - Q
        //       \
        //        SAN
        let orbits = generator("Q)YOU
Q)SAN
COM)Q
");
        assert_eq!(part2(&orbits), 0);
    }
    #[test]
    fn part2_mini_1l_deep() {
        //         M - YOU
        //        /
        // COM - Q
        //        \
        //         SAN
        let orbits = generator("Q)M
M)YOU
Q)SAN
COM)Q
");
        assert_eq!(part2(&orbits), 1);
    }
    #[test]
    fn part2_mini_1r_deep() {
        //         M - SAN
        //        /
        // COM - Q
        //        \
        //         YOU
        let orbits = generator("Q)YOU
Q)M
M)SAN
COM)Q
");
        assert_eq!(part2(&orbits), 1);
    }
}