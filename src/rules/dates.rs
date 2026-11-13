//! Reading, checking and calculating with genealogy dates.

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
