//! Soundex codes, as the original worked them out.

/// Soundex: spaces vanish first, then upper-case, map to digits, and drop a digit when it
/// repeats the one right before it. Anything uncoded, punctuation included, sits between digits,
/// so H and W separate repeats here, unlike the census rule. "Mc Carthy" gives M263.
pub fn soundex(name: &str) -> String {
    let letters: Vec<char> = name.chars().filter(|c| !c.is_whitespace()).map(|c| c.to_ascii_uppercase()).collect();
    if letters.is_empty() {
        return String::new();
    }
    fn digit(c: char) -> char {
        match c {
            'B' | 'F' | 'P' | 'V' => '1',
            'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
            'D' | 'T' => '3',
            'L' => '4',
            'M' | 'N' => '5',
            'R' => '6',
            _ => '0',
        }
    }
    let mut out = String::new();
    out.push(letters[0]);
    let mut prev = digit(letters[0]);
    for c in &letters[1..] {
        let d = digit(*c);
        if d != prev && d != '0' {
            out.push(d);
        }
        prev = d;
        if out.len() == 4 {
            break;
        }
    }
    while out.len() < 4 {
        out.push('0');
    }
    out
}
