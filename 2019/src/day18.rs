use serde::export::Formatter;
use std::collections::VecDeque;
use std::iter::once;
use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicUsize, AtomicIsize, Ordering};
use std::sync::Arc;
use threadpool_crossbeam_channel::ThreadPool;

#[aoc_generator(day18)]
pub fn generator(input: &str) -> Vec<u8> {
    input.bytes().collect()
}


#[aoc(day18, part1, serial)]
pub fn part1_serial(input: &Vec<u8>) -> isize {
    let mut pending_states = VecDeque::new();
    pending_states.push_front(State { maze: Maze::new(input.clone()), total_distance: 0 });

    let mut low_score = std::isize::MAX;
    let mut last_status_update = Instant::now();
    let mut num_states_added = 0usize;
    let mut num_states_explored = 0usize;
    let interval = 5;
    while let Some(mut state) = if pending_states.len() > 100_000_000 { pending_states.pop_back() } else { pending_states.pop_front() } {
        let num_states_before = pending_states.len();
        let last_score = state.explore(low_score, &mut pending_states);
        num_states_added += pending_states.len() - num_states_before;
        num_states_explored += 1;
        low_score = low_score.min(last_score);
        if last_status_update.elapsed() > Duration::from_secs(interval) {
            last_status_update = Instant::now();
            eprintln!("Low score: {} Pending States: {} Explore {}/s Add {}/s", low_score, pending_states.len(), num_states_explored / interval as usize, num_states_added / interval as usize);
            num_states_added = 0;
            num_states_explored = 0;
        }
    }

    low_score
}

fn schedule_state(pool: &ThreadPool, state: State, low_score: &Arc<AtomicIsize>, num_states_added: &Arc<AtomicUsize>, num_states_explored: &Arc<AtomicUsize>) {
    let num_states_added = num_states_added.clone();
    let num_states_explored = num_states_explored.clone();
    let low_score = low_score.clone();
    let shared_pool = pool.clone();
    pool.execute(move || {
        let mut pending_states = VecDeque::new();
        pending_states.push_front(state);

        while let Some(mut state) = pending_states.pop_back() {
            let mut last_score = low_score.load(Ordering::Relaxed);
            let mut new_score = last_score.min(state.explore(last_score, &mut pending_states));
            while let Err(updated_score) = low_score.compare_exchange(last_score, new_score, Ordering::SeqCst, Ordering::SeqCst) {
                new_score = updated_score.min(new_score);
                last_score = updated_score;
                if new_score == last_score {
                    break;
                }
            }

            num_states_added.fetch_add(pending_states.len(), Ordering::Relaxed);
            num_states_explored.fetch_add(1, Ordering::Relaxed);

            if shared_pool.queued_count() <= 1_000_000 {
                for pending_state in pending_states.drain(..) {
                    schedule_state(&shared_pool, pending_state, &low_score, &num_states_added, &num_states_explored);
                }
            }
        }
    });
}

#[aoc(day18, part1)]
pub fn part1(input: &Vec<u8>) -> isize {
    let pool = Arc::new(ThreadPool::new(6));
    let low_score = Arc::new(AtomicIsize::new(std::isize::MAX));
    let num_states_added = Arc::new(AtomicUsize::new(0));
    let  num_states_explored = Arc::new(AtomicUsize::new(0));
    schedule_state(&pool, State { maze: Maze::new(input.clone()), total_distance: 0 }, &low_score, &num_states_added, &num_states_explored);

    let interval = 5;
    while pool.queued_count() + pool.active_count() > 0 {
        let low_score = low_score.load(Ordering::Relaxed);
        let num_states_added = num_states_added.swap(0, Ordering::Relaxed);
        let num_states_explored = num_states_explored.swap(0, Ordering::Relaxed);

        eprintln!("Low score: {} Pending States: {} Explore {}/s Add {}/s", low_score, pool.queued_count(), num_states_explored / interval as usize, num_states_added / interval as usize);
        std::thread::sleep(Duration::from_secs(interval));
    }

    low_score.load(Ordering::SeqCst)
}

#[derive(Clone)]
struct State {
    maze: Maze,
    total_distance: isize,
}

impl State {
    fn explore(&mut self, low_score: isize, pending_states: &mut VecDeque<State>) -> isize {
        let mut reachable_keys= Vec::new();
        loop {
            let limit = low_score - self.total_distance - 1;
            self.maze.flood(limit, &mut reachable_keys);
            if reachable_keys.is_empty() {
                //eprintln!("Cannot reach last key in time, aborting state exploration");
                return std::isize::MAX;
            }
            let closest = reachable_keys.get(0).expect("moved into a dead end?!");
            if self.maze.num_keys_left == 1 {
                self.total_distance += closest.distance;
                return self.total_distance;
            } else if self.maze.num_keys_left < 1 {
                panic!("unexpectedly no keys left");
            } else {
                // Split off states that we need to explore later
                pending_states.extend(reachable_keys.iter().skip(1).map(|k| {
                    let mut state: State = self.clone();
                    state.move_to_key(k);
                    //eprintln!("          Will need to try out key {} (distance {}, remaining {}) later", char::from(k.key), k.distance, state.maze.num_keys_left);
                    state
                }));

                // Continue with 'our' copy of the state
                self.move_to_key(closest);
                //eprintln!("Continuing with key {} (distance {}, remaining {}) now", char::from(closest.key), closest.distance, self.maze.num_keys_left);
            }
        }
    }

    fn move_to_key(&mut self, candidate: &KeyCandidate) {
        self.maze.move_to_key(candidate.key);
        self.total_distance += candidate.distance;
    }
}

#[derive(Clone)]
struct Maze {
    maze: Vec<u8>,
    stride: usize,
    // used for test assertions
    #[allow(unused)]
    height: isize,
    // state
    player_idx: usize,
    num_keys_left: usize,
}

impl Maze {
    fn new(maze: Vec<u8>) -> Maze {
        let stride = 1 + maze.iter()
            .enumerate()
            .filter(|(_, c)| **c == b'\n')
            .map(|(i, _)| i)
            .next().expect("at least one line");
        let height = (maze.len() / stride) as isize + if maze.len() % stride == 0 { 0 } else { 1 };
        let player_idx = maze.iter()
            .enumerate()
            .find(|(_, x)| **x == b'@')
            .expect("maze to contain @")
            .0;
        let num_keys_left = maze.iter()
            .filter(|x| b'a' <= **x && **x <= b'z')
            .count();
        Maze { maze, stride, height, player_idx, num_keys_left }
    }

    fn idx(&self, coord: (isize, isize)) -> usize {
        let (x, y) = coord;
        (y as usize) * self.stride + (x as usize)
    }

    fn coord(&self, index: usize) -> (isize, isize) {
        let x = index % self.stride;
        let y = (index - x) / self.stride;
        (x as isize, y as isize)
    }

    fn flood(&self, limit: isize, candidates: &mut Vec<KeyCandidate>) {
        candidates.clear();
        let mut pending = VecDeque::new();
        pending.push_back((self.player_idx, 0));
        let mut overlay = self.maze.clone();
        while let Some((idx, distance)) = pending.pop_front() {
            // Abort if we would exceed the distance limit (guaranteed to be longer)
            if distance > limit {
                break;
            }

            // React to tile
            match overlay[idx] {
                b'#' | b'_' => continue,
                c if b'A' <= c && c <= b'Z' => continue,
                b'@' | b'.' => (),
                c if b'a' <= c && c <= b'z' => {
                    candidates.push(KeyCandidate { key: c, distance })
                }
                c => panic!("unexpected character '{}' at position {:?}", c, self.coord(idx))
            }

            // Paint tile
            overlay[idx] = b'_';

            // Flood adjacent tiles
            for adj_idx in self.adjacent_iter(idx) {
                pending.push_back((adj_idx, distance + 1))
            }
        }
        candidates.sort();
    }

    fn adjacent_iter<'a>(&'a self, idx: usize) -> impl Iterator<Item=usize> + 'a {
        let (x, y) = self.coord(idx);
        once((x, y - 1))
            .chain(once((x + 1, y)))
            .chain(once((x, y + 1)))
            .chain(once((x - 1, y)))
            .filter(move |(x, y)| 0 <= *x && *x < self.stride as isize
                && 0 <= *y && *y < self.height)
            .map(move |x| self.idx(x))
    }

    fn move_to_key(&mut self, key: u8) {
        // remove player
        self.maze[self.player_idx] = b'.';

        // remove keys and open door
        for (idx, x) in self.maze.iter_mut().enumerate() {
            if *x == key {
                *x = b'@';
                self.player_idx = idx;
            } else if *x + TO_LOWER == key {
                *x = b'.'
            }
        }
        self.num_keys_left -= 1;
    }
}

const TO_LOWER: u8 = b'a' - b'A';

#[derive(Clone, Copy, Eq, PartialEq, Debug, PartialOrd, Ord)]
struct KeyCandidate {
    distance: isize,
    key: u8,
}

impl std::fmt::Display for Maze {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.maze[..]).expect("maze should be valid utf-8"))
    }
}


#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_simple() {
        let m = Maze::new(generator("\
#########
#b.A.@.a#
#########"));
        assert_eq!(m.num_keys_left, 2);
        assert_eq!(m.coord(m.player_idx), (5, 1));
        assert_eq!(m.stride, 10);
        assert_eq!(m.height, 3);
    }

    #[test]
    fn flood_1() {
        let m = Maze::new(generator("\
#########
#b...@.a#
#########"));
        let mut cs = Vec::new();
        m.flood(1000, &mut cs);
        assert_eq!(cs, vec![
            KeyCandidate { key: b'a', distance: 2 },
            KeyCandidate { key: b'b', distance: 4 }
        ])
    }

    #[test]
    fn flood_2() {
        let m = Maze::new(generator("\
#########
#b.A.@.a#
#########"));
        let mut cs = Vec::new();
        m.flood(1000, &mut cs);
        assert_eq!(cs, vec![
            KeyCandidate { key: b'a', distance: 2 }
        ])
    }

    #[test]
    fn flood_3() {
        let m = Maze::new(generator("\
#########
#bc..@.a#
#########"));
        let mut cs = Vec::new();
        m.flood(1000, &mut cs);
        assert_eq!(cs, vec![
            KeyCandidate { key: b'a', distance: 2 },
            KeyCandidate { key: b'c', distance: 3 },
            KeyCandidate { key: b'b', distance: 4 }
        ])
    }

    #[test]
    fn flood_limit() {
        let m = Maze::new(generator("\
#########
#bc..@.a#
#########"));
        let mut cs = Vec::new();
        m.flood(3, &mut cs);
        assert_eq!(cs, vec![
            KeyCandidate { key: b'a', distance: 2 },
            KeyCandidate { key: b'c', distance: 3 }
        ])
    }

    #[test]
    fn explore_greedy() {
        let maze = Maze::new(generator("\
#########
#b.A.@.a#
#########"));
        let mut state = State { maze, total_distance: 0 };
        let mut pending_states = VecDeque::new();
        let low_score = state.explore(1000, &mut pending_states);
        assert_eq!(low_score, 8);
        assert_eq!(pending_states.len(), 0);
    }

    #[test]
    fn part1_simple() {
        let input = generator("\
#########
#b.A.@.a#
#########");
        assert_eq!(part1(&input), 8);
    }

    #[test]
    fn part1_example_larger() {
        let input = generator("\
########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################");
        assert_eq!(part1(&input), 86);
    }

    #[test]
    fn part1_example_1() {
        let input = generator("\
########################
#...............b.C.D.f#
#.######################
#.....@.a.B.c.d.A.e.F.g#
########################");
        assert_eq!(part1(&input), 132);
    }
    #[test]
    fn part1_example_2() {
        let input = generator("\
#################
#i.G..c...e..H.p#
########.########
#j.A..b...f..D.o#
########@########
#k.E..a...g..B.n#
########.########
#l.F..d...h..C.m#
#################");
        assert_eq!(part1(&input), 136);
    }
    #[test]
    fn part1_example_3() {
        let input = generator("\
########################
#@..............ac.GI.b#
###d#e#f################
###A#B#C################
###g#h#i################
########################");
        assert_eq!(part1(&input), 81);
    }
}