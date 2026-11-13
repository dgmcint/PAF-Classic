//! Changing names and places from CAPITALS to Mixed Case.

/// Words a name keeps in lower case: such as "de", "van" and "of".
const NAME_SMALL_WORDS: [&str; 47] = [
    "ap", "au", "da", "de", "degli", "dei", "dela", "delas", "della", "dello", "delos", "den",
    "der", "di", "do", "du", "el", "et", "las", "los", "of", "op", "or", "te", "ten", "ter", "uijt",
    "uit", "uitden", "und", "under", "up", "uyt", "van", "vande", "vanden", "vander", "vd", "vdr",
    "vn", "vom", "von", "vonder", "zer", "zu", "zum", "zur",
];

/// Words a place keeps in lower case: such as "of" and "on".
const PLACE_SMALL_WORDS: [&str; 120] = [
    "'r", "'s", "'t", "a", "a'r", "aan", "abu", "ad", "agli", "ai", "al", "alla", "alle", "am",
    "an", "and", "ap", "as", "ash", "at", "au", "auf", "aux", "az", "aá", "bei", "by", "cogli",
    "coi", "col", "colla", "colle", "da", "dans", "das", "de", "degli", "dei", "del", "della",
    "dello", "dem", "den", "der", "des", "det", "di", "do", "dos", "du", "ed", "el", "en", "es",
    "et", "het", "ibn", "im", "in", "juxta", "la", "las", "le", "les", "los", "med", "n", "na",
    "naarbij", "nad", "near", "negli", "nei", "nel", "nella", "nelle", "nr", "o", "ob", "och", "of",
    "og", "on", "op", "or", "os", "pri", "przy", "sous", "sugli", "sui", "sul", "sulla", "sullo",
    "super", "sur", "te", "ten", "ter", "the", "to", "u", "uber", "ueber", "um", "und", "under",
    "upon", "usa", "uèber", "van", "vander", "ved", "von", "vor", "y", "y'r", "zu", "zum", "zur",
];

/// Roman numerals, which become all capitals.
const ROMAN: [&str; 10] = ["x", "ix", "viii", "vii", "vi", "v", "iv", "iii", "ii", "i"];

/// What separates words.
fn word_break(c: char) -> bool {
    c.is_whitespace() || matches!(c, ',' | '(' | ')' | '[' | ']' | ';' | '.' | '/' | '"' | '<' | '>' | '-')
}

/// One word, by the original's rules.
fn word_case(word: &str, first: &mut bool, place: bool) -> String {
    let lower = word.to_lowercase();
    let is_first = std::mem::replace(first, false);
    // Only words all in capitals change, plus Roman numerals.
    if word != word.to_uppercase() && !ROMAN.contains(&lower.as_str()) {
        return word.to_string();
    }
    // A place's first word keeps its capital, unless it's "of".
    if !is_first || lower == "of" {
        if place && lower == "usa" {
            return word.to_uppercase();
        }
        let small: &[&str] = if place { &PLACE_SMALL_WORDS } else { &NAME_SMALL_WORDS };
        if small.contains(&lower.as_str()) {
            return lower;
        }
    }
    let chars: Vec<char> = lower.chars().collect();
    let capital_at = |positions: &[usize]| -> String {
        let mut out = String::new();
        for (i, c) in chars.iter().enumerate() {
            if positions.contains(&i) {
                out.extend(c.to_uppercase());
            } else {
                out.push(*c);
            }
        }
        out
    };
    let starts = |prefix: &str| lower.starts_with(prefix);
    let n = chars.len();
    // Two capitals: McDonald, MacKay, FitzGerald, O'Brien.
    if n >= 5 && starts("fitz") {
        return capital_at(&[0, 4]);
    }
    if n >= 4 && starts("mac") {
        return capital_at(&[0, 3]);
    }
    if n >= 3 && (starts("mc") || starts("o'") || starts("m'")) {
        return capital_at(&[0, 2]);
    }
    // One capital after a small joined word: dell'Arte, l'Ouverture.
    if n > 5 && (starts("coll'") || starts("dell'") || starts("nell'") || starts("sull'")) {
        return capital_at(&[5]);
    }
    if n >= 5 && starts("all'") {
        return capital_at(&[4]);
    }
    if n >= 3 && starts("l'") {
        return capital_at(&[2]);
    }
    if lower == "d'" {
        return lower;
    }
    if ROMAN.contains(&lower.as_str()) {
        return lower.to_uppercase();
    }
    capital_at(&[0])
}

/// Names and places written in capitals become upper and lower case. A name's small
/// words can come first (VAN GOGH to van Gogh), and a place's first word keeps its capital.
pub fn mixed_case(s: &str, place: bool) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let mut first = place;
    for c in s.chars() {
        if word_break(c) {
            if !word.is_empty() {
                out.push_str(&word_case(&word, &mut first, place));
                word.clear();
            }
            out.push(c);
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        out.push_str(&word_case(&word, &mut first, place));
    }
    out
}
