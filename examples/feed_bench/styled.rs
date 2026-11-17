use std::io::Write;

use trame::term::{Cell, Color, Flags};

use crate::cases::{ALPHABET, DIGITS, LINES, generate};

const SGR_COLORS: usize = 7;
const TOKEN_CELLS: usize = 3;
const TRUECOLOR: [u8; 3] = [249, 38, 114];
const PALETTE_COLOR: u8 = 81;
const BASIC_COLOR: u8 = 4;
const RESET: &str = "\x1b[0m";

fn sgr_color(index: usize) -> usize {
    index % SGR_COLORS + 1
}

pub fn sgr_lines() -> Vec<u8> {
    generate(LINES, |out, index| {
        let color = sgr_color(index);
        write!(
            out,
            "\x1b[3{color}m ligne {index} \x1b[1;4{color}m {ALPHABET} {RESET} {DIGITS} {ALPHABET}\r\n"
        )
    })
}

pub fn sgr_probe() -> Cell {
    let color = Color::Indexed(sgr_color(LINES - 1) as u8);
    Cell {
        ch: first_letter(),
        fg: color,
        bg: color,
        flags: Flags::BOLD,
        ..Cell::default()
    }
}

pub fn dense_lines() -> Vec<u8> {
    let [red, green, blue] = TRUECOLOR;
    let styles = [
        format!("\x1b[38;2;{red};{green};{blue}m"),
        format!("\x1b[38;5;{PALETTE_COLOR}m"),
        format!("\x1b[01;3{BASIC_COLOR}m"),
        RESET.to_owned(),
    ];
    generate(LINES, |out, index| {
        write!(out, "ligne {index} ")?;
        let tokens = ALPHABET.as_bytes().chunks(TOKEN_CELLS);
        for (token, style) in tokens.zip(styles.iter().cycle()) {
            out.extend_from_slice(style.as_bytes());
            out.extend_from_slice(token);
        }
        write!(out, "{RESET}\r\n")
    })
}

pub fn dense_probe() -> Cell {
    let [red, green, blue] = TRUECOLOR;
    Cell {
        ch: first_letter(),
        fg: Color::Rgb(red, green, blue),
        ..Cell::default()
    }
}

fn first_letter() -> char {
    ALPHABET.chars().next().expect("the alphabet has letters")
}
