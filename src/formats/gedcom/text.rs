//! Turning a GEDCOM file's bytes into text, including the ANSEL character set.

/// Turns the file's bytes into text. Handles UTF-8, UTF-16 (Unicode), ANSEL and ANSI.
pub fn decode(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        let le = bytes[0] == 0xFF;
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
            .collect();
        return String::from_utf16_lossy(&units);
    }
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4000)]).to_uppercase();
    if head.contains("1 CHAR UTF-8") || head.contains("1 CHAR UTF8") {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    if head.contains("1 CHAR ANSEL") {
        return ansel_to_text(bytes);
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|b| *b as char).collect(),
    }
}

/// ANSEL puts an accent byte before its letter. This covers the marks and letters in everyday use.
pub(crate) fn ansel_to_text(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut marks: Vec<char> = Vec::new();
    for b in bytes {
        let mark = match *b {
            0xE1 => Some('\u{300}'),
            0xE2 => Some('\u{301}'),
            0xE3 => Some('\u{302}'),
            0xE4 => Some('\u{303}'),
            0xE5 => Some('\u{304}'),
            0xE6 => Some('\u{306}'),
            0xE7 => Some('\u{307}'),
            0xE8 => Some('\u{308}'),
            0xE9 => Some('\u{30C}'),
            0xEA => Some('\u{30A}'),
            0xED => Some('\u{315}'),
            0xEE => Some('\u{30B}'),
            0xF0 => Some('\u{327}'),
            0xF1 => Some('\u{328}'),
            0xF2 => Some('\u{323}'),
            0xF3 => Some('\u{324}'),
            0xF6 => Some('\u{332}'),
            _ => None,
        };
        if let Some(m) = mark {
            marks.push(m);
            continue;
        }
        let ch = match *b {
            0xA1 => 'Ł',
            0xA2 => 'Ø',
            0xA3 => 'Đ',
            0xA4 => 'Þ',
            0xA5 => 'Æ',
            0xA6 => 'Œ',
            0xA7 => 'ʹ',
            0xA8 => '·',
            0xA9 => '♭',
            0xAA => '®',
            0xAB => '±',
            0xB1 => 'ł',
            0xB2 => 'ø',
            0xB3 => 'đ',
            0xB4 => 'þ',
            0xB5 => 'æ',
            0xB6 => 'œ',
            0xB7 => 'ʺ',
            0xB8 => 'ı',
            0xB9 => '£',
            0xBA => 'ð',
            0xC3 => '©',
            0xC5 => '¿',
            0xC6 => '¡',
            0xCF => 'ß',
            b if b < 0x80 => b as char,
            _ => '\u{FFFD}',
        };
        out.push(compose(ch, &marks));
        if compose(ch, &marks) == ch {
            for m in &marks {
                out.push(*m);
            }
        }
        marks.clear();
    }
    out
}

fn compose(c: char, marks: &[char]) -> char {
    if marks.len() != 1 {
        return c;
    }
    let table: &[(char, &str, &str)] = &[
        ('\u{300}', "aeiouAEIOU", "àèìòùÀÈÌÒÙ"),
        ('\u{301}', "aeiouyAEIOUY", "áéíóúýÁÉÍÓÚÝ"),
        ('\u{302}', "aeiouAEIOU", "âêîôûÂÊÎÔÛ"),
        ('\u{303}', "anoANO", "ãñõÃÑÕ"),
        ('\u{308}', "aeiouyAEIOU", "äëïöüÿÄËÏÖÜ"),
        ('\u{30A}', "aAuU", "åÅůŮ"),
        ('\u{327}', "cC", "çÇ"),
    ];
    for (m, from, to) in table {
        if *m == marks[0] {
            if let Some(i) = from.chars().position(|x| x == c) {
                return to.chars().nth(i).unwrap();
            }
        }
    }
    c
}
