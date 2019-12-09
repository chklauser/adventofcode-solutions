
#[aoc_generator(day8)]
pub fn generator(input: &str) -> Vec<u8> {
    input.chars().filter(|c| *c != '\n').map(|c| c.to_digit(10).expect("digits only") as u8).collect()
}

#[aoc(day8, part1)]
pub fn part1(input: &Vec<u8>) -> i16 {
    check_layer(input, 25 * 6)
}

fn check_layer(input: &Vec<u8>, stride: usize) -> i16 {
    let mut idx = 0;
    let mut best_layer_code = -1;
    let mut best_layer_score = stride as i16 + 1;
    while idx < input.len() {
        // next layer
        let mut freq = [0i16; 10];
        let end = idx + stride;
        while idx < end {
            // next pixel
            freq[input[idx] as usize] += 1;
            idx += 1;
        }

        // evaluate layer
        if freq[0] < best_layer_score {
            best_layer_code = freq[1] * freq[2];
            best_layer_score = freq[0];
        }
    }
    best_layer_code
}

fn z_merge(input: &[u8], stride: usize) -> Vec<u8> {
    // iterate in reverse to process layers from back to front
    let mut screen = vec![0; stride];
    let mut idx = input.len() - 1;
    loop {
        let color = input[idx];
        if color < 2 {
            screen[idx % stride] = color;
        }

        if idx == 0 {
            break;
        }
        idx -= 1;
    }

    screen
}

fn render(input: &[u8], width: usize) -> String {
    let mut buf = String::with_capacity(input.len()/width + input.len() + 5);

    buf.push('\n');
    for line in input.chunks_exact(width) {
        for c in line {
            //buf.push(('0' as u8 + *c) as char);
            if *c == 0 {
                buf.push(' ');
            } else {
                buf.push_str("\u{2588}");
            }
        }
        buf.push('\n')
    }

    buf
}

#[aoc(day8,part2)]
pub fn part2(input: &Vec<u8>) -> String {
    render(&z_merge(&input[..], 25 * 6), 25)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_1() {
        assert_eq!(generator("123\n126\n\n789\n012\n"), vec![1,2,3,1,2,6,7,8,9,0,1,2]);
    }

    #[test]
    fn part1_example1() {
        assert_eq!(check_layer(&generator("123\n126\n\n789\n012\n"), 6), 4);
    }

    #[test]
    fn part2_example1_flat() {
        assert_eq!(z_merge(&generator("0222\n1122\n2212\n0000"), 4), vec![0,1,1,0]);
    }
}