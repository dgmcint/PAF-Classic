//! Record checks: dates that don't make sense, and who may still be living.

use crate::i18n::{tr_n, trf};
use super::*;
use crate::data::*;

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
            out.push(trf("The christening date is more than {years} years after the birth date.", &[("years", &MAX_CHRISTENING_DELAY)]));
        }
    }
    if let (Some(b), Some(d)) = (earliest, death) {
        if years_between(b, d) > MAX_LIFESPAN as f64 {
            out.push(trf("The death date is more than {years} years after the birth or christening date.", &[("years", &MAX_LIFESPAN)]));
        }
    }
    if let (Some(b), Some(d)) = (earliest, burial) {
        if years_between(b, d) > MAX_LIFESPAN as f64 {
            out.push(trf("The burial date is more than {years} years after the birth or christening date.", &[("years", &MAX_LIFESPAN)]));
        }
    }
    if let (Some(d), Some(b)) = (death, burial) {
        if years_between(d, b) > MAX_BURIAL_DELAY as f64 {
            out.push(tr_n("The burial date is more than {n} year after the death date.", "The burial date is more than {n} years after the death date.", MAX_BURIAL_DELAY as usize, &[]));
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
                        out.push(trf("The individual's birth/christening date is less than {years} years after a parent's birth/christening date.", &[("years", &MIN_PARENT_AGE)]));
                    } else if is_father && gap > MAX_FATHER_AGE as f64 {
                        out.push(trf("The individual's birth/christening date is more than {years} years after the father's birth/christening date.", &[("years", &MAX_FATHER_AGE)]));
                    } else if !is_father && gap > MAX_MOTHER_AGE as f64 {
                        out.push(trf("The individual's birth/christening date is more than {years} years after the mother's birth/christening date.", &[("years", &MAX_MOTHER_AGE)]));
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
                out.push(trf("The sealing date is before a spouse's birth or christening date (MRIN {mrin}).", &[("mrin", &mrin)]));
            } else if years_between(e, s) < MIN_SEALING_AGE as f64 {
                out.push(trf("The sealing date is before the age of {years} (MRIN {mrin}).", &[("years", &MIN_SEALING_AGE), ("mrin", &mrin)]));
            }
        }
        if let Some(md) = val(&m.event.date) {
            if let Some(e) = earliest {
                let gap = years_between(e, md);
                if gap < 0.0 {
                    out.push(trf("The marriage date is before a spouse's birth or christening date (MRIN {mrin}).", &[("mrin", &mrin)]));
                } else if gap < MIN_MARRIAGE_AGE as f64 {
                    out.push(trf("The marriage date is less than {years} years after a spouse's birth or christening (MRIN {mrin}).", &[("years", &MIN_MARRIAGE_AGE), ("mrin", &mrin)]));
                } else if gap > MAX_MARRIAGE_AGE as f64 {
                    out.push(trf("The marriage date is more than {years} years after a spouse's birth or christening date (MRIN {mrin}).", &[("years", &MAX_MARRIAGE_AGE), ("mrin", &mrin)]));
                }
            }
            if let Some(d) = death {
                if md > d {
                    out.push(trf("The marriage date is after a spouse's death date (MRIN {mrin}).", &[("mrin", &mrin)]));
                }
            }
        }
        if m.husband == Some(rin) {
            if let (Some(h), Some(w)) = (earliest, m.wife.and_then(|r| db.person(r)).and_then(|x| val(&x.birth.date))) {
                let span = db.prefs.spouse_span;
                if years_between(h.min(w), h.max(w)) > span as f64 {
                    out.push(trf("Age between husband and wife is greater than {years} years (MRIN {mrin}).", &[("years", &span), ("mrin", &mrin)]));
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
                (m.husband, "Father's age at this birth, {age}, is outside {low} to {high} years.", db.prefs.father_low, db.prefs.father_high),
                (m.wife, "Mother's age at this birth, {age}, is outside {low} to {high} years.", db.prefs.mother_low, db.prefs.mother_high),
            ] {
                let Some(b) = par.and_then(|r| db.person(r)).and_then(|x| val(&x.birth.date).or(val(&x.chr.date))) else { continue };
                let age = years_between(b, e);
                if age >= 0.0 && (age < low as f64 || age > high as f64) {
                    out.push(trf(word, &[("age", &(age as i32)), ("low", &low), ("high", &high)]));
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
                out.push(trf("The children in marriage {mrin} are not in birth order.", &[("mrin", &mrin)]));
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
