use itertools::Itertools;

type Vector3 = (isize, isize, isize);

#[aoc_generator(day12)]
pub fn generator(input: &str) -> Vec<Vector3> {
    input.lines().map(|l| serde_scan::scan!("<x={}, y={}, z={}>" <- l).expect("input")).collect()
}



#[inline(always)]
fn apply_gravity(a: usize, b: usize, pos: &mut [isize], vel: &mut [isize]) -> () {
    let d = (pos[a] - pos[b]).signum();
    vel[a] -= d;
    vel[b] += d;
}
#[inline(always)]
fn apply_velocity(a: usize, pos: &mut [isize], vel: &mut [isize]) -> () {
    pos[a] += vel[a];
}

fn total_energy_after_steps(input: &[Vector3], steps: isize) -> isize {
    let n = input.len();
    let mut buf = vec![0isize; n * 6];
    let (px, vx, py, vy, pz, vz) = split_dimensions(&mut buf);
    initialize_pos(input, px, py, pz);

    fn total_energy(px: &mut [isize], vx: &mut [isize], py: &mut [isize], vy: &mut [isize], pz: &mut [isize], vz: &mut [isize]) -> isize {
        (0usize..px.len()).map(|a| (px[a].abs() + py[a].abs() + pz[a].abs()) * (vx[a].abs() + vy[a].abs() + vz[a].abs())).sum()
    }

    #[allow(unused)]
    for i in 1..=steps {
        // Apply gravity
        for (a,b) in &PAIRS {
            apply_gravity(*a, *b, px, vx);
            apply_gravity(*a, *b, py, vy);
            apply_gravity(*a, *b, pz, vz);
        }

        // Apply velocity
        for a in 0usize..n {
            apply_velocity(a, px, vx);
            apply_velocity(a, py, vy);
            apply_velocity(a, pz, vz);
        }

//        if (i % (steps / 10)) == 0 {
//            eprintln!("After {} steps:", i);
//            for a in 0usize..n {
//                eprintln!("pos=({}, {}, {}) vel=({}, {}, {})", px[a], py[a], pz[a], vx[a], vy[a], vz[a]);
//            }
//            eprintln!("total energy: {}\n", total_energy(px,py,pz, vx,vy, vz));
//        }
    }

    // Calculate total energy
    total_energy(px, vx, py, vy, pz, vz)
}

fn initialize_pos(input: &[Vector3], px: &mut [isize], py: &mut [isize], pz: &mut [isize]) {
    for (i, moon) in input.into_iter().enumerate() {
        px[i] = moon.0;
        py[i] = moon.1;
        pz[i] = moon.2;
    }
}


#[aoc(day12, part1)]
pub fn part1(input: &Vec<Vector3>) -> isize {
    total_energy_after_steps(&input[..], 1000)
}

static PAIRS : [(usize,usize); 6] = [(0,1), (0,2), (0,3), (1,2),(1,3),(2,3)];

const N: usize = 4;
//noinspection NonAsciiCharacters
#[aoc(day12, part2)]
pub fn part2(input: &Vec<Vector3>) -> String {
    if input.len() != N {
        panic!("only {} moons supported", N);
    }
    let init_x = input.iter().map(|m|m.0).collect::<Vec<_>>();
    let init_y = input.iter().map(|m|m.1).collect::<Vec<_>>();
    let init_z = input.iter().map(|m|m.2).collect::<Vec<_>>();
    let mut hpx = [0isize; 4];
    let mut hvx = [0isize; 4];
    let mut hpy = [0isize; 4];
    let mut hvy = [0isize; 4];
    let mut hpz = [0isize; 4];
    let mut hvz = [0isize; 4];
    let mut tpx = [0isize; 4];
    let mut tvx = [0isize; 4];
    let mut tpy = [0isize; 4];
    let mut tvy = [0isize; 4];
    let mut tpz = [0isize; 4];
    let mut tvz = [0isize; 4];
    initialize_pos(input, &mut hpx, &mut hpy, &mut hpz);
    initialize_pos(input, &mut tpx, &mut tpy, &mut tpz);

    let (x_mu, x_lam) = dim_find_cycle(&init_x[..], &mut hpx, &mut hvx, &mut tpx, &mut tvx);
    let (y_mu, y_lam) = dim_find_cycle(&init_y[..], &mut hpy, &mut hvy, &mut tpy, &mut tvy);
    let (z_mu, z_lam) = dim_find_cycle(&init_z[..], &mut hpz, &mut hvz, &mut tpz, &mut tvz);

    // I can't be asked to implement prime factorization and LCM for three integers.
    // To determine the minimal number of steps, we have to perform the μ unique steps (max) and
    // then on top of that the LCM of the λ (phase).
    // Wolfram|Alpha can easily do this with a query like:
    // "least common multiple of 22958, 286332, 231614"
    // https://www.wolframalpha.com/input/?i=least+common+multiple+of+22958%2C+286332%2C+231614
    format!("{:?}", (x_mu.max(y_mu).max(z_mu), (x_lam, y_lam, z_lam)))
}

fn dim_find_cycle(input: &[isize], mut hp: &mut [isize; N], mut hv: &mut [isize; N], mut tp: &mut [isize; N], mut tv: &mut [isize; N]) -> (isize, isize) {

// https://en.wikipedia.org/wiki/Cycle_detection#Floyd's_Tortoise_and_Hare
// Step 1: find a repetition (might not be the first one)
// hare advances twice as fast as the tortoise
    dim_time_step(&mut tp, &mut tv);
    dim_time_step(&mut hp, &mut hv);
    dim_time_step(&mut hp, &mut hv);
    while !dim_eq(&hp, &tp, &hv, &tv) {
        dim_time_step(&mut tp, &mut tv);
        dim_time_step(&mut hp, &mut hv);
        dim_time_step(&mut hp, &mut hv);
    }
// Step 2: find position μ of the first repetition
// tortoise starts at beginning, hare continues in lockstep within cycle
    let mut mu = 0isize;
    for e in tv.iter_mut() { *e = 0; }
    for (i, moon) in input.into_iter().enumerate() {
        tp[i] = *moon;
    }
    while !dim_eq(&hp, &tp, &hv, &tv) {
        dim_time_step(&mut tp, &mut tv);
        dim_time_step(&mut hp, &mut hv);
        mu += 1;
    }
// Step 3: find length of the shortest cycle starting from state μ
    let mut lam = 1isize;
// set hare to the state of tortoise
    for (h, t) in hp.iter_mut().zip_eq(tp.iter()) {
        *h = *t
    }
    for (h, t) in hv.iter_mut().zip_eq(tv.iter()) {
        *h = *t
    }
    dim_time_step(&mut hp, &mut hv);
    while !dim_eq(&hp, &tp, &hv, &tv) {
        dim_time_step(&mut hp, &mut hv);
        lam += 1;
    }
    (mu, lam)
}

fn dim_eq(hp: &[isize; N], tp: &[isize; N], hv: &[isize; N], tv: &[isize; N]) -> bool {
    hp == tp && hv == tv
}

fn dim_time_step(p: &mut [isize; 4], v: &mut [isize; 4]) -> () {
    // apply gravity
    apply_gravity(0, 1, p, v);
    apply_gravity(2, 3, p, v);
    apply_gravity(1, 2, p, v);
    apply_gravity(0, 3, p, v);
    apply_gravity(0, 2, p, v);
    apply_gravity(1, 3, p, v);

    // apply velocity
    for a in 0..N {
        apply_velocity(a, p, v);
    }
}


fn split_dimensions(buf: &mut Vec<isize>) -> (&mut [isize], &mut [isize], &mut [isize], &mut [isize], &mut [isize], &mut [isize]) {
    let (px, buf) = buf.split_at_mut(N);
    let (vx, buf) = buf.split_at_mut(N);
    let (py, buf) = buf.split_at_mut(N);
    let (vy, buf) = buf.split_at_mut(N);
    let (pz, vz) = buf.split_at_mut(N);
    debug_assert_eq!(vz.len(), N);
    (px, vx, py, vy, pz, vz)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator1() {
        let input = "<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>";
        let parsed = generator(input);
        assert_eq!(parsed, vec![(-1, 0, 2), (2, -10, -7), (4, -8, 8), (3, 5, -1)])
    }

    #[test]
    fn part1_example1() {
        let input = vec![(-1, 0, 2), (2, -10, -7), (4, -8, 8), (3, 5, -1)];
        assert_eq!(total_energy_after_steps(&input[..], 10), 179);
    }

    #[test]
    fn part1_example2() {
        let input = generator("<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>");
        assert_eq!(total_energy_after_steps(&input[..], 100), 1940);
    }
}