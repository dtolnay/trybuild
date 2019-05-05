use crate::prng;
use crate::term;
use termcolor::Color;
use termcolor::Color::*;

const WIDTH: usize = 30;
const COLORS: &[Color] = &[Blue, Green, Red, Yellow, Yellow];
const CHARS: &[char] = &['░', '▒'];

pub fn colorful() {
    println!();
    let mut rng = prng::get();

    for _ in 0..WIDTH {
        let color = COLORS[rng.u32() as usize % COLORS.len()];
        term::color(color);

        let ch = CHARS[rng.u32() as usize % CHARS.len()];
        print!("{}{}", ch, ch);
    }

    term::reset();
    print!("\n\n");
}

pub fn dotted() {
    println!("{}", "┈┈".repeat(WIDTH));
}
