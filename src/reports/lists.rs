//! Text reports: ancestry, descendants, summaries, lists, books, calendars and custom reports.

use super::*;
use crate::rules::*;
use crate::data::*;
use std::collections::BTreeMap;

/// Ancestry chart: everyone by Ahnentafel number, generation by generation.
pub fn ancestry(db: &Db, start: u32, gens: usize) -> Report {
    let mut f = Flow::new(&format!("Ancestors of {}", db.name(start)), &footer(db));
    let mut level = vec![(1usize, start)];
    let mut g = 1;
    let mut seen = vec![];
    while !level.is_empty() && g <= gens {
        f.gap(5.0);
        f.line(0.0, 11.0, true, &format!("Generation {g}"));
        let mut next = Vec::new();
        for (n, r) in level {
            if seen.contains(&r) {
                continue;
            }
            seen.push(r);
            let Some(p) = db.person(r) else { continue };
            f.line(6.0, 10.0, true, &format!("{n}. {}", plain_name(&p.name)));
            for (l, e) in [("Born", &p.birth), ("Died", &p.death)] {
                if let Some(s) = event_line(l, e) {
                    f.line(22.0, 9.0, false, &s);
                }
            }
            if let Some(d) = db.father(r) {
                next.push((n * 2, d));
            }
            if let Some(m) = db.mother(r) {
                next.push((n * 2 + 1, m));
            }
        }
        level = next;
        g += 1;
    }
    f.finish()
}

pub fn descendancy(db: &Db, start: u32, gens: u32) -> Report {
    let mut f = Flow::new(&format!("Descendants of {}", db.name(start)), &footer(db));
    fn walk(db: &Db, f: &mut Flow, rin: u32, depth: u32, gens: u32, seen: &mut Vec<u32>) {
        if depth >= gens || seen.contains(&rin) {
            return;
        }
        seen.push(rin);
        let Some(p) = db.person(rin) else { return };
        let indent = (depth.min(10) as f32) * 12.0;
        let mut life = String::new();
        if !p.birth.date.trim().is_empty() || !p.death.date.trim().is_empty() {
            life = format!(" (b. {}{})", p.birth.date.trim(), if p.death.date.trim().is_empty() { String::new() } else { format!(", d. {}", p.death.date.trim()) });
        }
        f.line(indent, 10.0, false, &format!("{}. {}{}", depth + 1, plain_name(&p.name), life));
        for mr in &p.spouses {
            if let Some(s) = db.spouse_in(*mr, rin) {
                f.line(indent + 8.0, 9.0, false, &format!("+ {}", db.name(s)));
            }
            if let Some(m) = db.marriages.get(mr) {
                for c in &m.children {
                    walk(db, f, *c, depth + 1, gens, seen);
                }
            }
        }
    }
    let mut seen = vec![];
    walk(db, &mut f, start, 0, gens, &mut seen);
    f.finish()
}

pub fn individual_summary(db: &Db, rin: u32) -> Report {
    let Some(p) = db.person(rin) else { return Report::default() };
    let mut f = Flow::new(&format!("Individual Summary: {}", plain_name(&p.name)), &footer(db));
    f.line(0.0, 9.0, false, &format!("RIN {}    Sex {}", p.rin, p.sex.letter()));
    if !p.title.is_empty() {
        f.line(0.0, 9.0, false, &format!("Title: {}", p.title));
    }
    if !p.nickname.is_empty() {
        f.line(0.0, 9.0, false, &format!("Nickname: {}", p.nickname));
    }
    if !p.aka.is_empty() {
        f.line(0.0, 9.0, false, &format!("Also known as: {}", p.aka));
    }
    f.rule();
    for (l, e) in [("Born", &p.birth), ("Christened", &p.chr), ("Died", &p.death), ("Buried", &p.burial)] {
        if let Some(s) = event_line(l, e) {
            f.line(0.0, 10.0, false, &s);
        }
    }
    for c in &p.custom {
        f.line(0.0, 10.0, false, &format!("{}: {} {} {}", c.kind, c.description, c.date, c.place));
    }
    if db.prefs.use_lds {
        ordinance_lines(&mut f, db, p);
    }
    if !p.parents.is_empty() {
        f.gap(4.0);
        f.line(0.0, 11.0, true, "Parents");
        for l in &p.parents {
            if let Some(m) = db.marriages.get(&l.mrin) {
                let dad = m.husband.map(|r| db.name(r)).unwrap_or_default();
                let mum = m.wife.map(|r| db.name(r)).unwrap_or_default();
                f.line(6.0, 10.0, false, &format!("{dad} and {mum} ({})", l.relation));
            }
        }
    }
    for mr in &p.spouses {
        let Some(m) = db.marriages.get(mr) else { continue };
        f.gap(4.0);
        let s = db.spouse_in(*mr, rin).map(|r| db.name(r)).unwrap_or_else(|| "Unknown spouse".into());
        f.line(0.0, 11.0, true, &format!("Spouse: {s}"));
        if let Some(e) = event_line("Married", &m.event) {
            f.line(6.0, 10.0, false, &e);
        }
        for (i, c) in m.children.iter().enumerate() {
            f.line(6.0, 10.0, false, &format!("{}. {}", i + 1, db.name(*c)));
        }
    }
    if !p.notes.trim().is_empty() {
        f.gap(4.0);
        f.line(0.0, 11.0, true, "Notes");
        f.line(6.0, 9.0, false, &p.notes);
    }
    let cites: Vec<&Citation> = p.citations.iter().chain(p.birth.citations.iter()).chain(p.death.citations.iter()).collect();
    if !cites.is_empty() {
        f.gap(4.0);
        f.line(0.0, 11.0, true, "Sources");
        for c in cites {
            let t = db.sources.get(&c.source).map(|s| s.title.clone()).unwrap_or_default();
            f.line(6.0, 9.0, false, &format!("{t} {}", c.page));
        }
    }
    f.finish()
}

pub const LIST_KINDS: [&str; 11] = [
    "Individuals sorted alphabetically",
    "Individuals sorted by RIN",
    "Unlinked individuals",
    "Duplicate individuals",
    "Marriages",
    "Possible problems",
    "End-of-line individuals",
    "Family reunion contacts",
    "Places sorted alphabetically",
    "Individuals with notes",
    "Sources",
];

pub fn list_report(db: &Db, kind: &str, focus: Option<&[u32]>) -> Report {
    let mut f = Flow::new(kind, &footer(db));
    let chosen = |r: u32| focus.map(|l| l.contains(&r)).unwrap_or(true);
    let mut names: Vec<&Person> = db.people.values().filter(|p| chosen(p.rin)).collect();
    names.sort_by_key(|p| sort_key(&p.name));
    let cols = |f: &mut Flow, p: &Person| {
        f.columns(
            &[
                (0.0, p.rin.to_string()),
                (40.0, plain_name(&p.name)),
                (250.0, p.birth.date.clone()),
                (340.0, p.death.date.clone()),
                (430.0, p.birth.place.clone()),
            ],
            9.0,
            false,
        );
    };
    let head = |f: &mut Flow| {
        f.columns(&[(0.0, "RIN".into()), (40.0, "Name".into()), (250.0, "Born".into()), (340.0, "Died".into()), (430.0, "Birthplace".into())], 9.0, true);
    };
    match kind {
        "Individuals sorted alphabetically" => {
            head(&mut f);
            names.iter().for_each(|p| cols(&mut f, p));
        }
        "Individuals sorted by RIN" => {
            head(&mut f);
            db.people.values().filter(|p| chosen(p.rin)).for_each(|p| cols(&mut f, p));
        }
        "Unlinked individuals" => {
            head(&mut f);
            names.iter().filter(|p| !db.is_linked(p.rin)).for_each(|p| cols(&mut f, p));
        }
        "Duplicate individuals" => {
            for (a, b) in duplicate_candidates(db, true, db.prefs.merge_years) {
                f.line(0.0, 9.0, false, &format!("{} (RIN {a})  and  {} (RIN {b})", db.name(a), db.name(b)));
            }
        }
        "Marriages" => {
            f.columns(&[(0.0, "MRIN".into()), (40.0, "Husband".into()), (220.0, "Wife".into()), (400.0, "Married".into())], 9.0, true);
            for m in db.marriages.values() {
                f.columns(
                    &[
                        (0.0, m.mrin.to_string()),
                        (40.0, m.husband.map(|r| db.name(r)).unwrap_or_default()),
                        (220.0, m.wife.map(|r| db.name(r)).unwrap_or_default()),
                        (400.0, m.event.date.clone()),
                    ],
                    9.0,
                    false,
                );
            }
        }
        "Possible problems" => {
            for p in db.people.values().filter(|p| chosen(p.rin)) {
                let found = problem_list(db, p.rin);
                if !found.is_empty() {
                    f.line(0.0, 10.0, true, &format!("{} (RIN {})", plain_name(&p.name), p.rin));
                    for m in found {
                        f.line(12.0, 9.0, false, &m);
                    }
                }
            }
        }
        "End-of-line individuals" => {
            head(&mut f);
            names.iter().filter(|p| p.parents.is_empty()).for_each(|p| cols(&mut f, p));
        }
        "Family reunion contacts" => {
            for p in names.iter().filter(|p| !p.contact.is_empty()) {
                f.line(0.0, 10.0, true, &plain_name(&p.name));
                f.line(12.0, 9.0, false, &p.contact.address);
                if !p.contact.phone.is_empty() {
                    f.line(12.0, 9.0, false, &p.contact.phone);
                }
                if !p.contact.email.is_empty() {
                    f.line(12.0, 9.0, false, &p.contact.email);
                }
            }
        }
        "Places sorted alphabetically" => {
            for p in db.all_places() {
                f.line(0.0, 9.0, false, &p);
            }
        }
        "Individuals with notes" => {
            head(&mut f);
            names.iter().filter(|p| !p.notes.trim().is_empty()).for_each(|p| cols(&mut f, p));
        }
        "Sources" => {
            for s in db.sources.values() {
                f.line(0.0, 10.0, true, &s.title);
                if !s.author.is_empty() {
                    f.line(12.0, 9.0, false, &format!("Author: {}", s.author));
                }
                if !s.publication.is_empty() {
                    f.line(12.0, 9.0, false, &format!("Publication: {}", s.publication));
                }
            }
        }
        _ => {}
    }
    f.finish()
}

/// Narrative book of an ancestor's descendants.
pub fn book(db: &Db, start: u32, gens: u32) -> Report {
    let mut f = Flow::new(&format!("Descendants of {}", db.name(start)), &footer(db));
    let mut level = vec![start];
    let mut seen = vec![];
    for g in 1..=gens {
        if level.is_empty() {
            break;
        }
        f.gap(6.0);
        f.line(0.0, 12.0, true, &format!("Generation {g}"));
        let mut next = vec![];
        for r in level {
            if seen.contains(&r) {
                continue;
            }
            seen.push(r);
            let Some(p) = db.person(r) else { continue };
            let mut s = plain_name(&p.name);
            if let Some(pm) = db.primary_parents(r) {
                let word = if p.sex == Sex::Female { "daughter" } else { "son" };
                let dad = pm.husband.map(|x| db.name(x));
                let mum = pm.wife.map(|x| db.name(x));
                match (dad, mum) {
                    (Some(a), Some(b)) => s.push_str(&format!(", {word} of {a} and {b}")),
                    (Some(a), None) | (None, Some(a)) => s.push_str(&format!(", {word} of {a}")),
                    _ => {}
                }
            }
            s.push('.');
            let pron = if p.sex == Sex::Female { "She" } else { "He" };
            if !p.birth.date.trim().is_empty() || !p.birth.place.trim().is_empty() {
                s.push_str(&format!(" {pron} was born {} {}.", on(&p.birth.date), in_(&p.birth.place)));
            }
            if !p.death.date.trim().is_empty() || !p.death.place.trim().is_empty() {
                s.push_str(&format!(" {pron} died {} {}.", on(&p.death.date), in_(&p.death.place)));
            }
            for mr in &p.spouses {
                let Some(m) = db.marriages.get(mr) else { continue };
                if let Some(o) = db.spouse_in(*mr, r) {
                    s.push_str(&format!(" {pron} married {} {} {}.", db.name(o), on(&m.event.date), in_(&m.event.place)));
                }
                if m.divorced {
                    s.push_str(" The marriage ended in divorce.");
                }
                next.extend(m.children.iter().copied());
            }
            let s = s.replace("  ", " ").replace(" .", ".");
            f.line(0.0, 10.0, false, &s);
            if !p.notes.trim().is_empty() {
                f.line(14.0, 9.0, false, &p.notes);
            }
            f.gap(4.0);
        }
        level = next;
    }
    f.finish()
}

/// Birthdays and anniversaries by month.
pub fn calendar(db: &Db, year: i32, include_dead: bool) -> Report {
    let mut f = Flow::new(&format!("Calendar for {year}"), &footer(db));
    let mut by_month: BTreeMap<u32, Vec<(u32, String)>> = BTreeMap::new();
    let df = db.prefs.day_first_numeric;
    for p in db.people.values() {
        if !include_dead && !maybe_living(db, p.rin) {
            continue;
        }
        if let Some(d) = parse_date(&p.birth.date, df) {
            if d.month > 0 && d.day > 0 {
                by_month.entry(d.month).or_default().push((d.day, format!("{} born {}", plain_name(&p.name), d.year)));
            }
        }
    }
    for m in db.marriages.values() {
        if let Some(d) = parse_date(&m.event.date, df) {
            if d.month > 0 && d.day > 0 {
                let n = |r: Option<u32>| r.map(|x| db.name(x)).unwrap_or_default();
                by_month.entry(d.month).or_default().push((d.day, format!("{} and {} married {}", n(m.husband), n(m.wife), d.year)));
            }
        }
    }
    const FULL: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    for (m, name) in FULL.iter().enumerate() {
        let mut items = by_month.remove(&(m as u32 + 1)).unwrap_or_default();
        items.sort();
        f.gap(5.0);
        f.line(0.0, 11.0, true, name);
        for (d, t) in items {
            f.line(10.0, 9.0, false, &format!("{d}: {t}"));
        }
    }
    f.finish()
}

/// A table of chosen fields: the Custom report.
pub const CUSTOM_FIELDS: [&str; 12] = ["RIN", "Full Name", "Sex", "Birth Date", "Birth Place", "Christening Date", "Death Date", "Death Place", "Burial Date", "Marriage Date", "Spouse", "AFN"];

pub fn field_value(db: &Db, p: &Person, field: &str) -> String {
    let first_m = p.spouses.first().and_then(|m| db.marriages.get(m));
    match field {
        "RIN" => p.rin.to_string(),
        "Full Name" => plain_name(&p.name),
        "Sex" => p.sex.letter().into(),
        "Birth Date" => p.birth.date.clone(),
        "Birth Place" => p.birth.place.clone(),
        "Christening Date" => p.chr.date.clone(),
        "Christening Place" => p.chr.place.clone(),
        "Death Date" => p.death.date.clone(),
        "Death Place" => p.death.place.clone(),
        "Burial Date" => p.burial.date.clone(),
        "Burial Place" => p.burial.place.clone(),
        "Marriage Date" => first_m.map(|m| m.event.date.clone()).unwrap_or_default(),
        "Spouse" => first_m.and_then(|m| db.spouse_in(m.mrin, p.rin)).map(|r| db.name(r)).unwrap_or_default(),
        "AFN" => p.afn.clone(),
        "Notes" => p.notes.clone(),
        "Surname" => surname_of(&p.name),
        "Given Name" => given_of(&p.name),
        "Soundex" => soundex(&surname_of(&p.name)),
        _ => String::new(),
    }
}

pub fn custom_report(db: &Db, title: &str, fields: &[String], who: &[u32]) -> Report {
    let mut f = Flow::new(if title.is_empty() { "Custom Report" } else { title }, &footer(db));
    let width = (PAGE_W - 2.0 * MARGIN) / fields.len().max(1) as f32;
    let head: Vec<(f32, String)> = fields.iter().enumerate().map(|(i, n)| (i as f32 * width, n.clone())).collect();
    f.columns(&head, 9.0, true);
    f.rule();
    for r in who {
        let Some(p) = db.person(*r) else { continue };
        let row: Vec<(f32, String)> = fields.iter().enumerate().map(|(i, n)| (i as f32 * width, field_value(db, p, n))).collect();
        f.columns(&row, 8.5, false);
    }
    f.finish()
}

/// A plain report of lines under a title.
pub fn lines_report(db: &Db, title: &str, lines: &[String]) -> Report {
    let mut f = Flow::new(title, &footer(db));
    for l in lines {
        f.line(0.0, 10.0, false, l);
    }
    f.finish()
}
