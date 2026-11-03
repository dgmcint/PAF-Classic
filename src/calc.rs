//! Rules taken from the original program: Soundex, dates, relationships, record checks.
use crate::model::*;
use std::collections::{BTreeMap, VecDeque};

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

pub const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ParsedDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub qualifier: Qual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Qual {
    #[default]
    Exact,
    About,
    Before,
    After,
}

fn month_from(word: &str) -> Option<u32> {
    let w = word.to_lowercase();
    let w = w.trim_end_matches('.');
    if w.len() < 3 {
        return None;
    }
    const FULL: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
    FULL.iter().position(|m| m.starts_with(w)).map(|i| i as u32 + 1)
}

/// What the original prints between the two dates of a range.
pub const RANGE_SEPARATOR: &str = " - ";

/// Reads the date forms the guide lists: 23 Mar 1742, Dec 1845, 1799, 3/11/1955, Abt 1850,
/// 14/15 May 1854, 28 Apr 1721/1722, From x To y, <1854>, and 44 BC. Returns None when the text
/// has no usable year, which the program treats as a non-standard date.
pub fn parse_date(text: &str, day_first: bool) -> Option<ParsedDate> {
    let mut s = text.trim().to_string();
    if s.is_empty() {
        return None;
    }
    // A marked date prints inside angle brackets, after any date word.
    s = s.replace(['<', '>'], "");
    let lower = s.to_lowercase();
    if let Some(i) = lower.find(" to ").or(lower.find(RANGE_SEPARATOR)) {
        s = s[..i].to_string();
    }
    let mut words: Vec<String> = s.split_whitespace().map(|w| w.to_string()).collect();
    let mut q = Qual::Exact;
    if let Some(first) = words.first().map(|w| w.to_lowercase()) {
        let f = first.trim_end_matches('.');
        let found = match f {
            "abt" | "about" | "circa" | "ca" | "c" | "est" | "estimated" | "cal" | "calculated" => Some(Qual::About),
            "bef" | "before" => Some(Qual::Before),
            "aft" | "after" => Some(Qual::After),
            "from" | "sub" | "submitted" | "between" | "bet" | "on" => Some(Qual::Exact),
            _ => None,
        };
        if let Some(found) = found {
            q = found;
            words.remove(0);
        }
    }
    let is_bc = |w: &String| w.to_lowercase().replace('.', "") == "bc";
    let bc_first = words.len() > 1 && words.first().map(is_bc).unwrap_or(false);
    if bc_first {
        words.remove(0);
    }
    let bc_last = words.last().map(is_bc).unwrap_or(false);
    if bc_last {
        words.pop();
    }
    let bc = bc_first || bc_last;
    if words.len() == 1 && words[0].contains('/') && words[0].split('/').count() == 3 {
        let parts: Vec<&str> = words[0].split('/').collect();
        let a: u32 = parts[0].parse().ok()?;
        let b: u32 = parts[1].parse().ok()?;
        let y: i32 = parts[2].parse().ok()?;
        let (d, m) = if day_first { (a, b) } else { (b, a) };
        if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            return None;
        }
        return Some(ParsedDate { year: y, month: m, day: d, qualifier: q });
    }
    let first_num = |w: &str| w.split('/').next().and_then(|n| n.parse::<i32>().ok());
    let mut out = ParsedDate { qualifier: q, ..Default::default() };
    match words.len() {
        1 => out.year = first_num(&words[0])?,
        2 => {
            out.month = month_from(&words[0])?;
            out.year = first_num(&words[1])?;
        }
        3 => {
            out.day = first_num(&words[0])? as u32;
            out.month = month_from(&words[1])?;
            out.year = first_num(&words[2])?;
        }
        _ => {
            // "2 Jun 1918 To ..." was cut above. Anything longer is not a standard date.
            return None;
        }
    }
    if out.day > 31 {
        return None;
    }
    if bc {
        out.year = -out.year;
    }
    if out.year == 0 {
        return None;
    }
    Some(out)
}

pub fn is_standard_date(text: &str, day_first: bool) -> bool {
    let t = text.trim();
    if t.is_empty() || parse_date(t, day_first).is_some() {
        return true;
    }
    // Words the guide allows in date fields.
    const WORDS: [&str; 22] = [
        "bic", "cancelled", "child", "chi", "cleared", "completed", "dead", "deceased", "done", "dns", "dns/can",
        "infant", "inf", "not married", "pre-1970", "stillborn", "sti", "uncleared", "bc", "submitted", "sub", "living",
    ];
    let l = t.to_lowercase();
    WORDS.iter().any(|w| l == *w || l.starts_with(&format!("{w} ")))
}

pub fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Days from civil date, Gregorian.
pub fn serial(y: i32, m: u32, d: u32) -> i64 {
    let y = y as i64 - if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The other way round: a day count back to a calendar date.
pub fn from_serial(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y as i32, m, d)
}

/// A date as a day count, using the first of the month or year when parts are missing.
pub fn date_value(text: &str, day_first: bool) -> Option<i64> {
    let d = parse_date(text, day_first)?;
    Some(serial(d.year, d.month.max(1), d.day.max(1)))
}

pub fn format_date(y: i32, m: u32, d: u32) -> String {
    format!("{} {} {}", d, MONTHS[(m.clamp(1, 12) - 1) as usize], y)
}

/// The earliest date the Date Calculator accepts.
pub const CALENDAR_START: (i32, u32) = (1752, 10);

pub fn date_in_calendar(y: i32, m: u32) -> bool {
    (y, m) >= CALENDAR_START
}

/// Years, months and days between two dates, counted the way people count birthdays.
pub fn date_difference(a: (i32, u32, u32), b: (i32, u32, u32)) -> (i32, i32, i32) {
    let mut months = (b.0 - a.0) * 12 + b.1 as i32 - a.1 as i32;
    let target = serial(b.0, b.1, b.2);
    let anchor = |n: i32| {
        let d = date_add(a, 0, n, 0);
        serial(d.0, d.1, d.2)
    };
    if anchor(months) > target {
        months -= 1;
    }
    ((months.div_euclid(12)), months.rem_euclid(12), (target - anchor(months)) as i32)
}

pub fn date_add(a: (i32, u32, u32), years: i32, months: i32, days: i32) -> (i32, u32, u32) {
    let total = a.0 * 12 + (a.1 as i32 - 1) + years * 12 + months;
    let y = total.div_euclid(12);
    let m = total.rem_euclid(12) as u32 + 1;
    let d = a.2.min(days_in_month(y, m));
    from_serial(serial(y, m, d) + days as i64)
}

/// Today's date, from the computer clock.
pub fn today() -> (i32, u32, u32) {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    from_serial((secs / 86400) as i64)
}

fn ancestors_with_depth(db: &Db, start: u32) -> BTreeMap<u32, u32> {
    let mut seen = BTreeMap::new();
    let mut q = VecDeque::new();
    q.push_back((start, 0u32));
    while let Some((rin, depth)) = q.pop_front() {
        if seen.contains_key(&rin) {
            continue;
        }
        seen.insert(rin, depth);
        if let Some(p) = db.person(rin) {
            for l in &p.parents {
                if let Some(m) = db.marriages.get(&l.mrin) {
                    for par in [m.husband, m.wife].into_iter().flatten() {
                        q.push_back((par, depth + 1));
                    }
                }
            }
        }
    }
    seen
}

pub fn ordinal(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn gendered(sex: Sex, f: &str, m: &str) -> String {
    if sex == Sex::Female { f.to_string() } else { m.to_string() }
}

fn great(n: u32) -> String {
    if n >= 2 { format!("{} Great ", ordinal(n)) } else { "Great ".to_string() }
}

/// The word for what A is to B, with A `up_a` steps and B `up_b` steps below a common ancestor.
/// Turns generation counts into relationship words.
pub fn relationship_word(sex_a: Sex, up_a: u32, up_b: u32) -> String {
    let (f, m) = (sex_a == Sex::Female, sex_a);
    let _ = f;
    if up_a == 0 {
        return match up_b {
            0 => "Same person".into(),
            1 => gendered(m, "Mother", "Father"),
            2 => gendered(m, "Grandmother", "Grandfather"),
            n => format!("{}{}", great(n - 2), gendered(m, "Grandmother", "Grandfather")),
        };
    }
    if up_b == 0 {
        return match up_a {
            1 => gendered(m, "Daughter", "Son"),
            2 => gendered(m, "Granddaughter", "Grandson"),
            n => format!("{}{}", great(n - 2), gendered(m, "Granddaughter", "Grandson")),
        };
    }
    if up_a == 1 && up_b == 1 {
        return gendered(m, "Sister", "Brother");
    }
    if up_a == 1 {
        return match up_b {
            2 => gendered(m, "Aunt", "Uncle"),
            3 => gendered(m, "Grand Aunt", "Grand Uncle"),
            n => format!("{}{}", great(n - 3), gendered(m, "Grand Aunt", "Grand Uncle")),
        };
    }
    if up_b == 1 {
        return match up_a {
            2 => gendered(m, "Niece", "Nephew"),
            3 => gendered(m, "Grand Niece", "Grand Nephew"),
            n => format!("{}{}", great(n - 3), gendered(m, "Grand Niece", "Grand Nephew")),
        };
    }
    let degree = up_a.min(up_b) - 1;
    let removed = up_a.abs_diff(up_b);
    let mut s = format!("{} Cousin", ordinal(degree));
    if removed == 1 {
        s.push_str(" 1 Time Removed");
    } else if removed > 1 {
        s.push_str(&format!(" {removed} Times Removed"));
    }
    s
}

pub struct Relationship {
    pub words: String,
    pub ancestors: Vec<u32>,
}

/// Searches back generation by generation for ancestors the two share. Step and in-law
/// relationships are not shown, as the guide says; a spouse is named directly.
pub fn relationships(db: &Db, a: u32, b: u32) -> Vec<Relationship> {
    let mut out = Vec::new();
    let sex_a = db.person(a).map(|p| p.sex).unwrap_or_default();
    if a == b {
        out.push(Relationship { words: "Same person".into(), ancestors: vec![] });
        return out;
    }
    let is_spouse = db.person(a).map(|p| p.spouses.iter().any(|m| db.spouse_in(*m, a) == Some(b))).unwrap_or(false);
    if is_spouse {
        let word = if sex_a == Sex::Female { "Wife" } else { "Husband" };
        out.push(Relationship { words: format!("{word} of {}", db.name(b)), ancestors: vec![] });
    }
    let aa = ancestors_with_depth(db, a);
    let ab = ancestors_with_depth(db, b);
    let mut common: Vec<(u32, u32, u32)> = aa.iter().filter_map(|(r, da)| ab.get(r).map(|db_| (*r, *da, *db_))).collect();
    common.sort_by_key(|c| c.1 + c.2);
    // Only nearest shared ancestors: skip anyone who is an ancestor of another shared ancestor.
    let mut groups: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (r, da, db_) in &common {
        let shadowed = common.iter().any(|(o, oa, ob)| {
            o != r && oa + ob < da + db_ && ancestors_with_depth(db, *o).contains_key(r)
        });
        if !shadowed {
            groups.entry((*da, *db_)).or_default().push(*r);
        }
    }
    let mut seen = Vec::new();
    for ((da, db_), anc) in groups {
        let words = relationship_word(sex_a, da, db_);
        if !seen.contains(&words) {
            seen.push(words.clone());
            out.push(Relationship { words, ancestors: anc });
        }
    }
    out
}

pub fn relation_to_root(db: &Db, rin: u32) -> Option<String> {
    let root = db.root_person?;
    if root == rin {
        return Some("Root Person".into());
    }
    relationships(db, rin, root).first().map(|r| r.words.clone())
}

/// Ages and gaps that trigger a warning. Each number comes from the program's own messages.
pub mod limits {
    pub const MIN_PARENT_AGE: i32 = 12; // 3725
    pub const MAX_FATHER_AGE: i32 = 70; // 3726
    pub const MAX_MOTHER_AGE: i32 = 52; // 3728
    /// "9 months" is counted as 280 days.
    pub const DAYS_AFTER_FATHER_DEATH: i64 = 280;
    pub const DAYS_BETWEEN_SIBLINGS: i64 = 280;
    pub const AGE_OF_ACCOUNTABILITY: i32 = 8;
    pub const MIN_SEALING_AGE: i32 = 12;
    pub const MAX_CHRISTENING_DELAY: i32 = 8; // 3730
    pub const MAX_LIFESPAN: i32 = 110; // 3731, 3747
    pub const MAX_BURIAL_DELAY: i32 = 1; // 3732
    pub const MIN_MARRIAGE_AGE: i32 = 12; // 3750
    pub const MAX_MARRIAGE_AGE: i32 = 60; // 3751
    pub const LDS_BAPTISM_START: (i32, u32, u32) = (1830, 4, 6); // 3296
    pub const LDS_ENDOWMENT_START: i32 = 1842; // 3297
}

fn years_between(a: i64, b: i64) -> f64 {
    (b - a) as f64 / 365.2425
}

/// Possible problems with one person's dates.
pub fn check_person(db: &Db, rin: u32) -> Vec<String> {
    use limits::*;
    let mut out = Vec::new();
    let Some(p) = db.person(rin) else { return out };
    let df = db.prefs.day_first_numeric;
    let val = |s: &str| date_value(s, df);
    let birth = val(&p.birth.date);
    let chr = val(&p.chr.date);
    let death = val(&p.death.date);
    let burial = val(&p.burial.date);
    let earliest = birth.or(chr);
    let now = { let t = today(); serial(t.0, t.1, t.2) };
    let mut later = |a: Option<i64>, b: Option<i64>, msg: &str| {
        if let (Some(a), Some(b)) = (a, b) {
            if a > b {
                out.push(msg.to_string());
            }
        }
    };
    later(birth, chr, "Birth date is later than christening date.");
    later(birth, death, "Birth date is later than death date.");
    later(birth, burial, "Birth date is later than burial date.");
    later(chr, death, "Christening date is later than death date.");
    later(chr, burial, "Christening date is later than burial date.");
    later(death, burial, "Death date is later than burial date.");
    let bap = val(&p.baptism.date);
    let end = val(&p.endowment.date);
    later(birth, bap, "Birth date is later than baptism date.");
    later(birth, end, "Birth date is later than endowment date.");
    later(bap, end, "Baptism date is later than endowment date.");
    if db.prefs.use_lds {
        if let Some(b) = bap {
            let s = LDS_BAPTISM_START;
            if b < serial(s.0, s.1, s.2) {
                out.push("Baptism date is before 6 Apr 1830.".into());
            }
        }
        if let Some(e) = end {
            if e < serial(LDS_ENDOWMENT_START, 1, 1) {
                out.push("Endowment date is before 1842.".into());
            }
        }
    }
    for (d, _) in [(birth, ""), (chr, ""), (death, ""), (burial, "")] {
        if d.map(|x| x > now).unwrap_or(false) {
            out.push("The event date for the individual cannot be in the future.".into());
            break;
        }
    }
    if let (Some(b), Some(c)) = (birth, chr) {
        if years_between(b, c) > MAX_CHRISTENING_DELAY as f64 {
            out.push(format!("The christening date is more than {MAX_CHRISTENING_DELAY} years after the birth date."));
        }
    }
    if let (Some(b), Some(d)) = (earliest, death) {
        if years_between(b, d) > MAX_LIFESPAN as f64 {
            out.push(format!("The death date is more than {MAX_LIFESPAN} years after the birth or christening date."));
        }
    }
    if let (Some(b), Some(d)) = (earliest, burial) {
        if years_between(b, d) > MAX_LIFESPAN as f64 {
            out.push(format!("The burial date is more than {MAX_LIFESPAN} years after the birth or christening date."));
        }
    }
    if let (Some(d), Some(b)) = (death, burial) {
        if years_between(d, b) > MAX_BURIAL_DELAY as f64 {
            out.push(format!("The burial date is more than {MAX_BURIAL_DELAY} year after the death date."));
        }
    }
    if let Some(e) = earliest {
        for link in &p.parents {
            let Some(m) = db.marriages.get(&link.mrin) else { continue };
            if let Some(md) = val(&m.event.date) {
                if e < md {
                    out.push("The individual's birth/christening date is before the parent's marriage date.".into());
                }
            }
            for (par, is_father) in [(m.husband, true), (m.wife, false)] {
                let Some(pp) = par.and_then(|r| db.person(r)) else { continue };
                if let Some(pb) = val(&pp.birth.date).or(val(&pp.chr.date)) {
                    let gap = years_between(pb, e);
                    if gap < 0.0 {
                        out.push("The individual's birth/christening date is before a parent's birth/christening date.".into());
                    } else if gap < MIN_PARENT_AGE as f64 {
                        out.push(format!("The individual's birth/christening date is less than {MIN_PARENT_AGE} years after a parent's birth/christening date."));
                    } else if is_father && gap > MAX_FATHER_AGE as f64 {
                        out.push(format!("The individual's birth/christening date is more than {MAX_FATHER_AGE} years after the father's birth/christening date."));
                    } else if !is_father && gap > MAX_MOTHER_AGE as f64 {
                        out.push(format!("The individual's birth/christening date is more than {MAX_MOTHER_AGE} years after the mother's birth/christening date."));
                    }
                }
                if is_father && link.relation == "Biological" {
                    if let Some(fd) = val(&pp.death.date) {
                        if e > fd + DAYS_AFTER_FATHER_DEATH {
                            out.push("The individual's birth/christening date is more than 9 months after the natural father's death date.".to_string());
                        }
                    }
                }
            }
        }
    }
    // Next older and younger siblings, by their place in the family's list.
    if let (Some(b), Some(link)) = (birth, p.parents.first()) {
        if let Some(kids) = db.marriages.get(&link.mrin).map(|m| &m.children) {
            if let Some(i) = kids.iter().position(|c| *c == rin) {
                let sib = |c: Option<&u32>| c.and_then(|c| db.person(*c)).and_then(|s| val(&s.birth.date));
                if let Some(older) = i.checked_sub(1).and_then(|j| sib(kids.get(j))) {
                    if b < older {
                        out.push("This birth date precedes the next older sibling's birth date.".to_string());
                    } else if b - older < DAYS_BETWEEN_SIBLINGS {
                        out.push("The birth date of the next older sibling is less than 9 months from this birth date.".to_string());
                    }
                }
                if let Some(younger) = sib(kids.get(i + 1)) {
                    if younger < b {
                        out.push("The next younger sibling's birth date precedes this birth date.".to_string());
                    } else if younger - b < DAYS_BETWEEN_SIBLINGS {
                        out.push("The birth date of the next younger sibling is less than 9 months from this birth date.".to_string());
                    }
                }
            }
        }
    }
    if let (Some(bp), Some(b)) = (bap, birth) {
        if b <= bp && years_between(b, bp) < AGE_OF_ACCOUNTABILITY as f64 {
            out.push("The baptism date is before the age of accountability.".to_string());
        }
    }
    for mrin in &p.spouses {
        let Some(m) = db.marriages.get(mrin) else { continue };
        if let (Some(s), Some(e)) = (val(&m.sealing.date), earliest) {
            if s < e {
                out.push(format!("The sealing date is before a spouse's birth or christening date (MRIN {mrin})."));
            } else if years_between(e, s) < MIN_SEALING_AGE as f64 {
                out.push(format!("The sealing date is before the age of {MIN_SEALING_AGE} (MRIN {mrin})."));
            }
        }
        if let Some(md) = val(&m.event.date) {
            if let Some(e) = earliest {
                let gap = years_between(e, md);
                if gap < 0.0 {
                    out.push(format!("The marriage date is before a spouse's birth or christening date (MRIN {mrin})."));
                } else if gap < MIN_MARRIAGE_AGE as f64 {
                    out.push(format!("The marriage date is less than {MIN_MARRIAGE_AGE} years after a spouse's birth or christening (MRIN {mrin})."));
                } else if gap > MAX_MARRIAGE_AGE as f64 {
                    out.push(format!("The marriage date is more than {MAX_MARRIAGE_AGE} years after a spouse's birth or christening date (MRIN {mrin})."));
                }
            }
            if let Some(d) = death {
                if md > d {
                    out.push(format!("The marriage date is after a spouse's death date (MRIN {mrin})."));
                }
            }
        }
        if m.husband == Some(rin) {
            if let (Some(h), Some(w)) = (earliest, m.wife.and_then(|r| db.person(r)).and_then(|x| val(&x.birth.date))) {
                let span = db.prefs.spouse_span;
                if years_between(h.min(w), h.max(w)) > span as f64 {
                    out.push(format!("Age between husband and wife is greater than {span} years (MRIN {mrin})."));
                }
            }
        }
    }
    out
}

/// What the Possible Problems list reports: every date check above, plus the parent age range
/// and the marriage and children order, both set in the list's options.
pub fn problem_list(db: &Db, rin: u32) -> Vec<String> {
    let mut out = check_person(db, rin);
    let Some(p) = db.person(rin) else { return out };
    let df = db.prefs.day_first_numeric;
    let val = |s: &str| date_value(s, df);
    if let Some(e) = val(&p.birth.date).or(val(&p.chr.date)) {
        for link in &p.parents {
            let Some(m) = db.marriages.get(&link.mrin) else { continue };
            for (par, word, low, high) in [
                (m.husband, "Father", db.prefs.father_low, db.prefs.father_high),
                (m.wife, "Mother", db.prefs.mother_low, db.prefs.mother_high),
            ] {
                let Some(b) = par.and_then(|r| db.person(r)).and_then(|x| val(&x.birth.date).or(val(&x.chr.date))) else { continue };
                let age = years_between(b, e);
                if age >= 0.0 && (age < low as f64 || age > high as f64) {
                    out.push(format!("{word}'s age at this birth, {}, is outside {low} to {high} years.", age as i32));
                }
            }
        }
    }
    if db.prefs.check_order {
        for mrin in &p.spouses {
            let Some(m) = db.marriages.get(mrin) else { continue };
            if m.husband != Some(rin) && (m.husband.is_some() || m.wife != Some(rin)) {
                continue;
            }
            let dates: Vec<Option<i64>> = m.children.iter().filter_map(|c| db.person(*c)).map(|c| val(&c.birth.date).or(val(&c.chr.date))).collect();
            let known: Vec<i64> = dates.into_iter().flatten().collect();
            if known.windows(2).any(|w| w[0] > w[1]) {
                out.push(format!("The children in marriage {mrin} are not in birth order."));
            }
        }
        let known: Vec<i64> = p.spouses.iter().filter_map(|m| db.marriages.get(m)).filter_map(|m| val(&m.event.date)).collect();
        if known.windows(2).any(|w| w[0] > w[1]) {
            out.push("The marriages are not in date order.".into());
        }
    }
    if !db.prefs.check_lds {
        let lds = ["Baptism", "Endowment", "baptism", "endowment"];
        out.retain(|m| !lds.iter().any(|w| m.contains(w)));
    }
    out
}

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

/// Is this person treated as possibly living? No death or burial, and born within the limit.
pub fn maybe_living(db: &Db, rin: u32) -> bool {
    let Some(p) = db.person(rin) else { return false };
    if !p.death.date.trim().is_empty() || !p.burial.date.trim().is_empty() || !p.death.place.trim().is_empty() {
        return false;
    }
    let df = db.prefs.day_first_numeric;
    match date_value(&p.birth.date, df).or(date_value(&p.chr.date, df)) {
        Some(b) => {
            let t = today();
            years_between(b, serial(t.0, t.1, t.2)) < limits::MAX_LIFESPAN as f64
        }
        None => true,
    }
}

/// What an ordinance field means for its letter.
#[derive(Clone, Copy, PartialEq)]
enum Mark {
    Blank,
    Dash,
    Done,
    Pending,
}

#[derive(Clone, Copy, PartialEq)]
enum Rite {
    Baptism,
    Endowment,
    SealParents,
    SealSpouse,
}

fn ordinance_mark(field: &str, rite: Rite) -> Mark {
    let t = field.trim().to_lowercase();
    let is = |w: &str| t == w || t.starts_with(&format!("{w} "));
    if t.is_empty() {
        Mark::Blank
    } else if is("bic") {
        if rite == Rite::SealParents { Mark::Done } else { Mark::Blank }
    } else if is("cancelled") || is("completed") || is("done") || is("pre-1970") {
        Mark::Done
    } else if is("child") || is("infant") {
        if matches!(rite, Rite::Baptism | Rite::Endowment) { Mark::Dash } else { Mark::Blank }
    } else if is("cleared") || is("submitted") || is("sub") {
        Mark::Pending
    } else if is("dns") || is("dns/can") || is("uncleared") {
        Mark::Dash
    } else if is("stillborn") {
        if rite == Rite::SealSpouse { Mark::Blank } else { Mark::Dash }
    } else if parse_date(&t, false).is_some() {
        Mark::Done
    } else {
        Mark::Blank
    }
}

fn mark_letter(mark: Mark, letter: char) -> char {
    match mark {
        Mark::Blank => ' ',
        Mark::Dash => '-',
        Mark::Done => letter,
        Mark::Pending => letter.to_ascii_lowercase(),
    }
}

/// The ordinance letters on the Family screen, in the original's order: B, E, P, S.
/// With no marriages the last one is a dash. A woman shows S from any one sealed marriage, and a
/// man only when all are sealed.
pub fn ordinance_flags(db: &Db, rin: u32) -> String {
    let Some(p) = db.person(rin) else { return String::new() };
    let par = p.parents.first().map(|l| l.sealed.date.as_str()).unwrap_or("");
    let marks: Vec<Mark> = p.spouses.iter().filter_map(|m| db.marriages.get(m)).map(|m| ordinance_mark(&m.sealing.date, Rite::SealSpouse)).collect();
    let spouse = if marks.is_empty() {
        '-'
    } else if p.sex == Sex::Female {
        let mut shown = '-';
        for m in &marks {
            match m {
                Mark::Done => {
                    shown = 'S';
                    break;
                }
                Mark::Pending => {
                    shown = 's';
                    break;
                }
                Mark::Blank => shown = ' ',
                Mark::Dash => {}
            }
        }
        shown
    } else {
        let mut shown = 'S';
        for m in &marks {
            if shown == ' ' {
                break;
            }
            shown = match m {
                Mark::Blank => ' ',
                Mark::Dash => '-',
                Mark::Pending => shown.to_ascii_lowercase(),
                Mark::Done => shown,
            };
        }
        shown
    };
    // The parents' letter shows only when done or pending, never as a dash.
    let parents = match ordinance_mark(par, Rite::SealParents) {
        Mark::Dash => ' ',
        m => mark_letter(m, 'P'),
    };
    format!(
        "{} {} {} {}",
        mark_letter(ordinance_mark(&p.baptism.date, Rite::Baptism), 'B'),
        mark_letter(ordinance_mark(&p.endowment.date, Rite::Endowment), 'E'),
        parents,
        spouse
    )
}

/// Everyone descended from the person, with generation numbers.
pub fn descendants(db: &Db, rin: u32) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let mut seen = vec![rin];
    fn walk(db: &Db, rin: u32, depth: u32, out: &mut Vec<(u32, u32)>, seen: &mut Vec<u32>) {
        for c in db.children_of(rin) {
            if seen.contains(&c) {
                continue;
            }
            seen.push(c);
            out.push((c, depth + 1));
            walk(db, c, depth + 1, out, seen);
        }
    }
    walk(db, rin, 0, &mut out, &mut seen);
    out
}

/// Everyone in the person's ancestry, with generation numbers.
pub fn ancestors(db: &Db, rin: u32) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = ancestors_with_depth(db, rin).into_iter().filter(|(r, _)| *r != rin).collect();
    v.sort_by_key(|x| x.1);
    v
}

/// Pairs that look like the same person, for Match/Merge, with the original's default options.
/// Names are compared as whole surname and given name, and then these
/// checks apply: same sex, not spouses or parent and child, a surname and a date on both, dates within
/// the year range, and parents whose names agree.
pub fn duplicate_candidates(db: &Db, sound_alike: bool, year_slack: i32) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let v: Vec<&Person> = db.people.values().filter(|p| !p.name.trim().is_empty()).collect();
    let df = db.prefs.day_first_numeric;
    let same_name = |x: u32, y: u32| {
        let (Some(px), Some(py)) = (db.person(x), db.person(y)) else { return true };
        soundex(&surname_of(&px.name)) == soundex(&surname_of(&py.name)) && soundex(&given_of(&px.name)) == soundex(&given_of(&py.name))
    };
    let parents_agree = |a: &Person, b: &Person| {
        if a.parents.is_empty() || b.parents.is_empty() {
            return true;
        }
        a.parents.iter().filter_map(|l| db.marriages.get(&l.mrin)).any(|ma| {
            b.parents.iter().filter_map(|l| db.marriages.get(&l.mrin)).any(|mb| {
                let fathers = match (ma.husband, mb.husband) {
                    (Some(x), Some(y)) => same_name(x, y),
                    _ => true,
                };
                let mothers = match (ma.wife, mb.wife) {
                    (Some(x), Some(y)) => same_name(x, y),
                    _ => true,
                };
                fathers && mothers
            })
        })
    };
    for (i, a) in v.iter().enumerate() {
        for b in &v[i + 1..] {
            let (sa, sb) = (surname_of(&a.name), surname_of(&b.name));
            let (ga, gb) = (given_of(&a.name).to_lowercase(), given_of(&b.name).to_lowercase());
            let name_match = if sound_alike {
                soundex(&sa) == soundex(&sb) && soundex(&ga) == soundex(&gb)
            } else {
                sa.eq_ignore_ascii_case(&sb) && ga == gb
            };
            if !name_match || sa.trim().is_empty() || sb.trim().is_empty() {
                continue;
            }
            // The original compares the sex byte itself, so Unknown only matches Unknown.
            if a.sex != b.sex {
                continue;
            }
            // People already joined as spouses or as parent and child are never the same person.
            let linked = a.spouses.iter().any(|m| db.spouse_in(*m, a.rin) == Some(b.rin))
                || db.children_of(a.rin).contains(&b.rin)
                || db.children_of(b.rin).contains(&a.rin);
            if linked || !parents_agree(a, b) {
                continue;
            }
            // A person's christening stands in when there's no birth date, and both need a date.
            let field = |p: &Person| if p.birth.date.trim().is_empty() { p.chr.date.clone() } else { p.birth.date.clone() };
            if field(a).trim().is_empty() || field(b).trim().is_empty() {
                continue;
            }
            let (ya, yb) = (parse_date(&field(a), df), parse_date(&field(b), df));
            if let (Some(x), Some(y)) = (ya, yb) {
                if (x.year - y.year).abs() > year_slack {
                    continue;
                }
            }
            out.push((a.rin, b.rin));
        }
    }
    out
}

/// Replaces matching text in one group of fields. Returns how many fields changed.
pub fn global_replace(db: &mut Db, field: &str, find: &str, with: &str, whole: bool, case_sensitive: bool) -> usize {
    if find.is_empty() {
        return 0;
    }
    let mut n = 0;
    let mut swap = |s: &mut String| {
        let hit = if whole {
            if case_sensitive { s.trim() == find } else { s.trim().eq_ignore_ascii_case(find) }
        } else if case_sensitive {
            s.contains(find)
        } else {
            s.to_lowercase().contains(&find.to_lowercase())
        };
        if !hit {
            return;
        }
        if whole {
            *s = with.to_string();
        } else if case_sensitive {
            *s = s.replace(find, with);
        } else {
            *s = replace_ignore_case(s, find, with);
        }
        n += 1;
    };
    let people = db.people.values_mut();
    match field {
        "Places" => {
            for p in people {
                for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
                    swap(&mut e.place);
                }
                for c in p.custom.iter_mut() {
                    swap(&mut c.place);
                }
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.event.place);
                for c in m.custom.iter_mut() {
                    swap(&mut c.place);
                }
            }
        }
        "Dates" => {
            for p in people {
                for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
                    swap(&mut e.date);
                }
                swap(&mut p.baptism.date);
                swap(&mut p.endowment.date);
                for c in p.custom.iter_mut() {
                    swap(&mut c.date);
                }
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.event.date);
                swap(&mut m.sealing.date);
            }
        }
        "Full Name" => {
            for p in people {
                swap(&mut p.name);
            }
        }
        "Notes" => {
            for p in people {
                swap(&mut p.notes);
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.notes);
            }
        }
        "Temple Codes" => {
            for p in people {
                swap(&mut p.baptism.temple);
                swap(&mut p.endowment.temple);
                for l in p.parents.iter_mut() {
                    swap(&mut l.sealed.temple);
                }
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.sealing.temple);
            }
        }
        "Multimedia file names" => {
            for p in people {
                for x in p.media.iter_mut() {
                    swap(&mut x.file);
                }
            }
        }
        _ => {}
    }
    n
}

fn replace_ignore_case(s: &str, find: &str, with: &str) -> String {
    let (ls, lf) = (s.to_lowercase(), find.to_lowercase());
    if ls.len() != s.len() {
        return s.replace(find, with);
    }
    let mut out = String::new();
    let mut i = 0;
    while let Some(j) = ls[i..].find(&lf) {
        out.push_str(&s[i..i + j]);
        out.push_str(with);
        i += j + lf.len();
    }
    out.push_str(&s[i..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soundex_matches_the_guide() {
        for n in ["Smith", "Smithe", "Smyth", "Smythe"] {
            assert_eq!(soundex(n), "S530");
        }
        assert_eq!(soundex("Kelly"), "K400");
        assert_eq!(soundex("McCarthy"), "M263");
        assert_eq!(soundex("Mc Carthy"), "M263");
    }

    #[test]
    fn soundex_keeps_the_original_quirks() {
        // First letter's own code counts, so the F is dropped.
        assert_eq!(soundex("Pfister"), "P236");
        // H separates repeats in this program (the census rule would give A261).
        assert_eq!(soundex("Ashcraft"), "A226");
    }

    #[test]
    fn dates_parse() {
        let d = parse_date("23 Mar 1742", false).unwrap();
        assert_eq!((d.year, d.month, d.day), (1742, 3, 23));
        assert_eq!(parse_date("Dec 1845", false).unwrap().month, 12);
        assert_eq!(parse_date("1799", false).unwrap().year, 1799);
        assert_eq!(parse_date("Abt 1850", false).unwrap().qualifier, Qual::About);
        assert_eq!(parse_date("14/15 May 1854", false).unwrap().day, 14);
        assert_eq!(parse_date("28 Apr 1721/1722", false).unwrap().year, 1721);
        assert_eq!(parse_date("<1854>", false).unwrap().year, 1854);
        assert_eq!(parse_date("From 10 Sep 1867 To 17 Aug 1869", false).unwrap().year, 1867);
        assert_eq!(parse_date("10 Sep 1867 - 17 Aug 1869", false).unwrap().year, 1867);
        assert_eq!(parse_date("Abt <1854>", false).unwrap().year, 1854);
        assert_eq!(parse_date("<1854>", false).unwrap().year, 1854);
        assert_eq!(parse_date("BC 44", false).unwrap().year, -44);
        assert_eq!(parse_date("44 BC", false).unwrap().year, -44);
        assert_eq!(parse_date("44 BC", false).unwrap().year, -44);
        let us = parse_date("3/11/1955", false).unwrap();
        assert_eq!((us.month, us.day), (3, 11));
        let eu = parse_date("3/11/1955", true).unwrap();
        assert_eq!((eu.month, eu.day), (11, 3));
        assert!(parse_date("feast of St John", false).is_none());
        assert!(is_standard_date("Stillborn", false));
        assert!(!is_standard_date("feast of St John", false));
    }

    #[test]
    fn date_math() {
        assert_eq!(from_serial(serial(1900, 3, 1)), (1900, 3, 1));
        assert_eq!(date_difference((2000, 1, 31), (2000, 3, 1)), (0, 1, 1));
        assert_eq!(date_difference((1950, 6, 15), (2000, 6, 15)), (50, 0, 0));
        assert_eq!(date_add((2000, 1, 31), 0, 1, 0), (2000, 2, 29));
        assert_eq!(date_add((1999, 12, 31), 0, 0, 1), (2000, 1, 1));
    }

    #[test]
    fn relationship_words() {
        assert_eq!(relationship_word(Sex::Male, 0, 1), "Father");
        assert_eq!(relationship_word(Sex::Female, 0, 2), "Grandmother");
        assert_eq!(relationship_word(Sex::Male, 0, 3), "Great Grandfather");
        assert_eq!(relationship_word(Sex::Male, 0, 4), "2nd Great Grandfather");
        assert_eq!(relationship_word(Sex::Female, 1, 0), "Daughter");
        assert_eq!(relationship_word(Sex::Female, 1, 1), "Sister");
        assert_eq!(relationship_word(Sex::Male, 1, 2), "Uncle");
        assert_eq!(relationship_word(Sex::Male, 1, 4), "Great Grand Uncle");
        assert_eq!(relationship_word(Sex::Male, 1, 5), "2nd Great Grand Uncle");
        assert_eq!(relationship_word(Sex::Female, 2, 1), "Niece");
        assert_eq!(relationship_word(Sex::Male, 2, 2), "1st Cousin");
        assert_eq!(relationship_word(Sex::Male, 3, 2), "1st Cousin 1 Time Removed");
        assert_eq!(relationship_word(Sex::Male, 3, 3), "2nd Cousin");
        assert_eq!(relationship_word(Sex::Male, 5, 3), "2nd Cousin 2 Times Removed");
    }

    #[test]
    fn relationships_in_sample() {
        let db = sample_family();
        let me = db.home.unwrap();
        let dad = db.father(me).unwrap();
        assert_eq!(relationships(&db, dad, me)[0].words, "Father");
        let wife = db.person(me).unwrap().spouses[0];
        let wife = db.spouse_in(wife, me).unwrap();
        assert!(relationships(&db, wife, me)[0].words.starts_with("Wife of"));
        // Parents who share two common ancestors only print one word.
        let kids = db.children_of(me);
        let r = relationships(&db, kids[0], kids[1]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].words, "Brother");
        assert_eq!(r[0].ancestors.len(), 2);
    }

    #[test]
    fn checks_catch_problems() {
        let mut db = sample_family();
        let me = db.home.unwrap();
        assert!(check_person(&db, me).is_empty(), "{:?}", check_person(&db, me));
        db.people.get_mut(&me).unwrap().death.date = "1900".into();
        let p = check_person(&db, me);
        assert!(p.iter().any(|m| m.contains("later than death date")));
    }

    #[test]
    fn case_changes() {
        assert_eq!(mixed_case("SMITH", false), "Smith");
        assert_eq!(mixed_case("LA ROSE", false), "La Rose");
        assert_eq!(mixed_case("VOM RATH", false), "vom Rath");
        assert_eq!(mixed_case("ISLE OF THE LAKE, USA", true), "Isle of the Lake, USA");
        assert_eq!(mixed_case("MCDONALD", false), "McDonald");
        assert_eq!(mixed_case("MACKINTOSH", false), "MacKintosh");
        assert_eq!(mixed_case("VAN DER BERG", false), "van der Berg");
        // The original's rules: all capitals only, numerals, joined words and separators.
        assert_eq!(mixed_case("McDonald", false), "McDonald");
        assert_eq!(mixed_case("JOHN /SMITH/ III", false), "John /Smith/ III");
        assert_eq!(mixed_case("O'BRIEN", false), "O'Brien");
        assert_eq!(mixed_case("FITZGERALD", false), "FitzGerald");
        assert_eq!(mixed_case("DELL'ARTE", false), "dell'Arte");
        assert_eq!(mixed_case("SMITH-JONES", false), "Smith-Jones");
        assert_eq!(mixed_case("BOISE, ADA, IDAHO, USA", true), "Boise, Ada, Idaho, USA");
        assert_eq!(mixed_case("DE SOTO, FLORIDA", true), "De Soto, Florida");
    }

    #[test]
    fn global_replace_works() {
        let mut db = sample_family();
        let n = global_replace(&mut db, "Places", "Boise", "Nampa", false, true);
        assert!(n > 0);
        assert!(db.all_places().iter().all(|p| !p.contains("Boise")));
    }

    #[test]
    fn duplicates_found() {
        let mut db = sample_family();
        let mut twin = db.person(db.home.unwrap()).unwrap().clone();
        twin.rin = 0;
        twin.spouses.clear();
        twin.parents.clear();
        twin.name = "Thomas Edward /Harow/".into();
        let t = db.add_person(twin);
        let c = duplicate_candidates(&db, true, 2);
        assert!(c.contains(&(db.home.unwrap(), t)));
        assert_eq!(with_surname_marks("John Henry Smith Jr."), "John Henry /Smith/ Jr.");
        assert_eq!(with_surname_marks("John Smith"), "John /Smith/");
        assert_eq!(with_surname_marks("Madonna"), "Madonna //");
        // Two words stay a given name and surname, even when the last looks like an ending.
        assert_eq!(with_surname_marks("Anna V"), "/Anna/ V");
        assert_eq!(with_surname_marks("Ludwig van Beethoven"), "Ludwig /van Beethoven/");
        assert_eq!(with_surname_marks("Maria Elena de la Cruz"), "Maria Elena /de la Cruz/");
        assert_eq!(with_surname_marks("Hans van der Berg Jr."), "Hans /van der Berg/ Jr.");
        assert_eq!(with_surname_marks("Angus Mac Donald"), "Angus /Mac Donald/");
        assert_eq!(with_surname_marks("Van Gogh"), "Van /Gogh/");
        db.merge_people(db.home.unwrap(), t);
        assert!(db.person(t).is_none());
    }
}
