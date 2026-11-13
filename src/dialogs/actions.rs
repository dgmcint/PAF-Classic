//! Changes to the family file that more than one window makes.

use super::*;
use crate::rules;
use crate::data::*;
use crate::reports::{self};

pub fn merge_duplicate_sources(db: &mut Db) -> usize {
    let mut map: Vec<(u32, u32)> = Vec::new();
    let list: Vec<Source> = db.sources.values().cloned().collect();
    for (i, a) in list.iter().enumerate() {
        if map.iter().any(|(old, _)| *old == a.id) {
            continue;
        }
        for b in &list[i + 1..] {
            if a.title.trim().eq_ignore_ascii_case(b.title.trim()) && a.author.trim().eq_ignore_ascii_case(b.author.trim()) && a.publication.trim().eq_ignore_ascii_case(b.publication.trim()) {
                map.push((b.id, a.id));
            }
        }
    }
    let swap = |list: &mut Vec<Citation>, map: &[(u32, u32)]| {
        for c in list.iter_mut() {
            if let Some((_, new)) = map.iter().find(|(old, _)| *old == c.source) {
                c.source = *new;
            }
        }
        let mut seen: Vec<Citation> = Vec::new();
        list.retain(|c| {
            if seen.contains(c) {
                false
            } else {
                seen.push(c.clone());
                true
            }
        });
    };
    for p in db.people.values_mut() {
        swap(&mut p.citations, &map);
        for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
            swap(&mut e.citations, &map);
        }
    }
    for m in db.marriages.values_mut() {
        swap(&mut m.citations, &map);
        swap(&mut m.event.citations, &map);
    }
    for (old, _) in &map {
        db.sources.remove(old);
    }
    map.len()
}

pub fn remove_source_citations(db: &mut Db, source: u32) {
    let clean = |l: &mut Vec<Citation>| l.retain(|c| c.source != source);
    for p in db.people.values_mut() {
        clean(&mut p.citations);
        for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
            clean(&mut e.citations);
        }
    }
    for m in db.marriages.values_mut() {
        clean(&mut m.citations);
        clean(&mut m.event.citations);
    }
}

pub fn filter_people(db: &Db, base: usize, start: u32, gens: u32, rows: &[FilterRow], any: bool) -> Vec<u32> {
    let mut who: Vec<u32> = match base {
        1 => rules::ancestors(db, start).into_iter().filter(|x| x.1 < gens).map(|x| x.0).collect(),
        2 => rules::descendants(db, start).into_iter().filter(|x| x.1 < gens).map(|x| x.0).collect(),
        _ => db.people.keys().copied().collect(),
    };
    if base != 0 && db.people.contains_key(&start) {
        who.insert(0, start);
    }
    if rows.is_empty() {
        return who;
    }
    let df = db.prefs.day_first_numeric;
    who.retain(|r| {
        let Some(p) = db.person(*r) else { return false };
        let test = |f: &FilterRow| {
            let v = reports::field_value(db, p, &f.field);
            let want = f.value.trim();
            match f.op.as_str() {
                "contains" => v.to_lowercase().contains(&want.to_lowercase()),
                "equals" => v.trim().eq_ignore_ascii_case(want),
                "is empty" => v.trim().is_empty(),
                "is not empty" => !v.trim().is_empty(),
                "is before" | "is after" => match (rules::date_value(&v, df), rules::date_value(want, df)) {
                    (Some(a), Some(b)) => {
                        if f.op == "is before" {
                            a < b
                        } else {
                            a > b
                        }
                    }
                    _ => false,
                },
                _ => false,
            }
        };
        if any { rows.iter().any(test) } else { rows.iter().all(test) }
    });
    who
}

pub fn parse_ymd(text: &str, day_first: bool) -> Option<(i32, u32, u32)> {
    let d = rules::parse_date(text, day_first)?;
    if d.month == 0 || d.day == 0 || d.day > rules::days_in_month(d.year, d.month) {
        return None;
    }
    Some((d.year, d.month, d.day))
}
