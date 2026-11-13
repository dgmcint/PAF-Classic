//! Splitting and sorting names written as Given /Surname/ Suffix.

pub fn given_of(name: &str) -> String {
    match name.find('/') {
        Some(i) => name[..i].trim().to_string(),
        None => name.trim().to_string(),
    }
}

pub fn surname_of(name: &str) -> String {
    let mut parts = name.split('/');
    parts.next();
    match parts.next() {
        Some(s) => s.trim().to_string(),
        None => String::new(),
    }
}

pub fn suffix_of(name: &str) -> String {
    let mut parts = name.split('/');
    parts.next();
    parts.next();
    parts.next().map(|s| s.trim().to_string()).unwrap_or_default()
}

/// A name with its surname marks taken out, as it appears on screen.
pub fn plain_name(name: &str) -> String {
    let joined = [given_of(name), surname_of(name), suffix_of(name)];
    joined.iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ")
}

/// Endings that aren't surnames, ignoring case.
const NAME_SUFFIXES: [&str; 19] = [
    "Jr.", "Sr.", "Jr", "Sr", "II", "III", "IV", "V", "VI", "PhD", "MD", "M.D.", "DDS", "D.D.S.", "Esq", "Esq.", "Esquire", "Second", "Third",
];

/// Words that join the surname when they come right before it.
const SURNAME_WORDS: [&str; 5] = ["de", "von", "van", "Mac", "Mc"];

/// Puts slashes around the surname when the person typed no marks. A single word has
/// an empty surname. An ending like Jr. is set aside first. The surname is then the last word,
/// with "de la" or "van der" from the fourth word on, or one of the joining words from the third.
pub fn with_surname_marks(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() || name.contains('/') {
        return name.to_string();
    }
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() == 1 {
        return format!("{name} //");
    }
    let is = |word: &str, other: &str| word.eq_ignore_ascii_case(other);
    let has_suffix = NAME_SUFFIXES.iter().any(|s| is(words[words.len() - 1], s));
    let last = if has_suffix { words.len() - 2 } else { words.len() - 1 };
    let start = if last >= 3 && ((is(words[last - 2], "de") && is(words[last - 1], "la")) || (is(words[last - 2], "van") && is(words[last - 1], "der"))) {
        last - 2
    } else if last > 1 && SURNAME_WORDS.iter().any(|w| is(words[last - 1], w)) {
        last - 1
    } else {
        last
    };
    let mut out = format!("/{}/", words[start..=last].join(" "));
    if start > 0 {
        out = format!("{} {out}", words[..start].join(" "));
    }
    if has_suffix {
        out = format!("{out} {}", words[words.len() - 1]);
    }
    out
}

/// Sorted by surname, then given name. People without a surname sort first.
pub fn sort_key(name: &str) -> (String, String, String) {
    (surname_of(name).to_lowercase(), given_of(name).to_lowercase(), suffix_of(name).to_lowercase())
}
