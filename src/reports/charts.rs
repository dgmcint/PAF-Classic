//! Pedigree charts and family group records, filled in or blank.

use super::*;
use crate::data::*;
use std::collections::BTreeMap;

/// Pedigree chart, with as many generations per page as asked (the original offers 4, 5 and 6).
pub fn pedigree(db: &Db, start: u32, gens: usize) -> Report {
    let gens = gens.clamp(2, 6);
    let title = if db.people.contains_key(&start) { format!("Pedigree Chart of {}", db.name(start)) } else { "Pedigree Chart".to_string() };
    let mut page = Page::default();
    page.ops.push(Op::Text { x: MARGIN, y: MARGIN, size: 15.0, bold: true, text: title.clone() });
    // Ahnentafel numbering: person 1, father 2, mother 3, then doubling.
    let mut slots: BTreeMap<usize, u32> = BTreeMap::new();
    slots.insert(1, start);
    for n in 1..(1usize << (gens - 1)) {
        if let Some(r) = slots.get(&n).copied() {
            if let Some(f) = db.father(r) {
                slots.insert(n * 2, f);
            }
            if let Some(m) = db.mother(r) {
                slots.insert(n * 2 + 1, m);
            }
        }
    }
    let top = MARGIN + 30.0;
    let height = PAGE_H - top - MARGIN - 20.0;
    let col_w = (PAGE_W - 2.0 * MARGIN) / gens as f32;
    let mut pos: BTreeMap<usize, (f32, f32)> = BTreeMap::new();
    for g in 0..gens {
        let count = 1usize << g;
        for i in 0..count {
            let n = count + i;
            let y = top + (i as f32 + 0.5) * height / count as f32;
            let x = MARGIN + g as f32 * col_w;
            pos.insert(n, (x, y));
        }
    }
    let size = match gens {
        2..=4 => 9.0,
        5 => 7.5,
        _ => 6.0,
    };
    for (n, (x, y)) in &pos {
        page.ops.push(Op::Line { x1: *x, y1: *y, x2: *x + col_w - 4.0, y2: *y });
        if *n > 1 {
            let (px, py) = pos[&(n / 2)];
            page.ops.push(Op::Line { x1: *x, y1: py, x2: *x, y2: *y });
            let _ = px;
        }
        let number = format!("{n}");
        page.ops.push(Op::Text { x: *x, y: *y + 1.0, size: size - 1.5, bold: false, text: number });
        if let Some(p) = slots.get(n).and_then(|r| db.people.get(r)) {
            let text = fit(&plain_name(&p.name), col_w - 8.0, size);
            page.ops.push(Op::Text { x: *x + 1.0, y: *y - size - 2.0, size, bold: true, text });
            let mut dy = *y + size;
            let mut extra = vec![];
            if !p.birth.date.trim().is_empty() {
                extra.push(format!("b: {}", p.birth.date.trim()));
            }
            if !p.death.date.trim().is_empty() {
                extra.push(format!("d: {}", p.death.date.trim()));
            }
            if gens <= 4 && !p.birth.place.trim().is_empty() {
                extra.push(format!("p: {}", p.birth.place.trim()));
            }
            for e in extra {
                page.ops.push(Op::Text { x: *x + 10.0, y: dy, size: size - 1.5, bold: false, text: fit(&e, col_w - 16.0, size - 1.5) });
                dy += size;
            }
        }
    }
    page.ops.push(Op::Text { x: MARGIN, y: PAGE_H - MARGIN, size: 8.0, bold: false, text: footer(db) });
    Report { title, pages: vec![page] }
}

/// An empty chart to fill in by hand: the Blank Form choice on the original's Pedigree tab.
pub fn blank_pedigree(gens: usize) -> Report {
    pedigree(&Db::default(), 0, gens)
}

/// An empty record to fill in by hand: the Blank Form choice on the original's Family Group tab.
pub fn blank_family_group() -> Report {
    let mut f = Flow::new("Family Group Record", "");
    let blank = "______________________________________________";
    for who in ["Husband", "Wife"] {
        f.rule();
        f.line(0.0, 11.0, true, &format!("{who}: {blank}"));
        for label in ["Born", "Christened", "Died", "Buried", "Father", "Mother"] {
            f.line(14.0, 10.0, false, &format!("{label}: {blank}"));
            f.gap(2.0);
        }
        f.gap(4.0);
    }
    f.rule();
    f.line(0.0, 11.0, true, &format!("Marriage: {blank}"));
    f.gap(6.0);
    f.rule();
    f.line(0.0, 11.0, true, "Children");
    for n in 1..=8 {
        f.gap(5.0);
        f.line(0.0, 10.0, false, &format!("{n}.  Name: {blank}   Sex: ____"));
        f.line(18.0, 10.0, false, "Born: ______________________   Died: ______________________");
    }
    f.finish()
}

pub fn family_group(db: &Db, mrin: u32) -> Report {
    let Some(m) = db.marriages.get(&mrin) else { return Report::default() };
    let hn = m.husband.map(|r| db.name(r)).unwrap_or_default();
    let wn = m.wife.map(|r| db.name(r)).unwrap_or_default();
    let mut f = Flow::new("Family Group Record", &footer(db));
    f.line(0.0, 10.0, false, &format!("MRIN {mrin}"));
    f.gap(4.0);
    let person = |f: &mut Flow, label: &str, who: Option<u32>| {
        f.rule();
        let name = who.map(|r| db.name(r)).unwrap_or_default();
        f.line(0.0, 11.0, true, &format!("{label}: {name}"));
        if let Some(p) = who.and_then(|r| db.person(r)) {
            f.line(14.0, 9.0, false, &format!("RIN {}", p.rin));
            for (l, e) in [("Born", &p.birth), ("Christened", &p.chr), ("Died", &p.death), ("Buried", &p.burial)] {
                if let Some(s) = event_line(l, e) {
                    f.line(14.0, 9.0, false, &s);
                }
            }
            if let Some(pm) = db.primary_parents(p.rin) {
                let dad = pm.husband.map(|r| db.name(r)).unwrap_or_default();
                let mum = pm.wife.map(|r| db.name(r)).unwrap_or_default();
                f.line(14.0, 9.0, false, &format!("Father: {dad}"));
                f.line(14.0, 9.0, false, &format!("Mother: {mum}"));
            }
            if db.prefs.use_lds {
                ordinance_lines(f, db, p);
            }
        }
    };
    person(&mut f, "Husband", m.husband);
    if let Some(s) = event_line("Married", &m.event) {
        f.line(14.0, 9.0, false, &s);
    }
    if m.divorced {
        f.line(14.0, 9.0, false, "The marriage ended in divorce.");
    }
    if db.prefs.use_lds && !m.sealing.is_empty() {
        f.line(14.0, 9.0, false, &format!("Sealed to spouse: {} {}", m.sealing.date, m.sealing.temple));
    }
    person(&mut f, "Wife", m.wife);
    f.rule();
    f.line(0.0, 11.0, true, "Children");
    for (i, c) in m.children.iter().enumerate() {
        let Some(p) = db.person(*c) else { continue };
        f.gap(3.0);
        f.line(0.0, 10.0, true, &format!("{}. {} {}", i + 1, p.sex.letter(), plain_name(&p.name)));
        for (l, e) in [("Born", &p.birth), ("Christened", &p.chr), ("Died", &p.death), ("Buried", &p.burial)] {
            if let Some(s) = event_line(l, e) {
                f.line(14.0, 9.0, false, &s);
            }
        }
        for mr in &p.spouses {
            if let Some(s) = db.spouse_in(*mr, *c) {
                f.line(14.0, 9.0, false, &format!("Spouse: {}", db.name(s)));
            }
            if let Some(x) = db.marriages.get(mr) {
                if let Some(s) = event_line("Married", &x.event) {
                    f.line(14.0, 9.0, false, &s);
                }
            }
        }
        if db.prefs.use_lds {
            ordinance_lines(&mut f, db, p);
        }
    }
    if !m.notes.trim().is_empty() {
        f.rule();
        f.line(0.0, 11.0, true, "Notes");
        f.line(0.0, 9.0, false, &m.notes);
    }
    let mut r = f.finish();
    r.title = format!("Family Group Record: {hn} and {wn}");
    r
}

pub(crate) fn ordinance_lines(f: &mut Flow, db: &Db, p: &Person) {
    let line = |f: &mut Flow, l: &str, o: &Ordinance| {
        if !o.is_empty() {
            f.line(14.0, 9.0, false, &format!("{l}: {} {}", o.date.trim(), o.temple.trim()));
        }
    };
    line(f, "Baptized", &p.baptism);
    line(f, "Endowed", &p.endowment);
    if let Some(l) = p.parents.first() {
        line(f, "Sealed to parents", &l.sealed);
    }
    let _ = db;
}
