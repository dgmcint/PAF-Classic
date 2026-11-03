//! Printable reports, a small PDF writer, and web page creation.
use crate::calc::*;
use crate::model::*;
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub const PAGE_W: f32 = 612.0;
pub const PAGE_H: f32 = 792.0;
const MARGIN: f32 = 42.0;

#[derive(Clone, Debug)]
pub enum Op {
    Text { x: f32, y: f32, size: f32, bold: bool, text: String },
    Line { x1: f32, y1: f32, x2: f32, y2: f32 },
    #[allow(dead_code)]
    Rect { x: f32, y: f32, w: f32, h: f32, shade: bool },
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    pub ops: Vec<Op>,
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub title: String,
    pub pages: Vec<Page>,
}

/// Lays text out from the top of the page, starting a new page when one fills.
struct Flow {
    report: Report,
    page: Page,
    y: f32,
    footer: String,
}

impl Flow {
    fn new(title: &str, footer: &str) -> Self {
        let mut f = Flow { report: Report { title: title.into(), pages: vec![] }, page: Page::default(), y: MARGIN, footer: footer.into() };
        f.heading(title);
        f
    }
    fn heading(&mut self, title: &str) {
        self.page.ops.push(Op::Text { x: MARGIN, y: self.y, size: 15.0, bold: true, text: title.into() });
        self.y += 24.0;
    }
    fn space(&mut self, need: f32) {
        if self.y + need > PAGE_H - MARGIN - 14.0 {
            self.flush();
        }
    }
    fn flush(&mut self) {
        let n = self.report.pages.len() + 1;
        let footer = format!("{}    Page {}", self.footer, n);
        self.page.ops.push(Op::Text { x: MARGIN, y: PAGE_H - MARGIN, size: 8.0, bold: false, text: footer });
        self.report.pages.push(std::mem::take(&mut self.page));
        self.y = MARGIN;
    }
    fn line(&mut self, indent: f32, size: f32, bold: bool, text: &str) {
        let width = PAGE_W - 2.0 * MARGIN - indent;
        for part in wrap(text, width, size) {
            self.space(size + 3.0);
            self.page.ops.push(Op::Text { x: MARGIN + indent, y: self.y, size, bold, text: part });
            self.y += size + 3.0;
        }
    }
    fn gap(&mut self, h: f32) {
        self.y += h;
    }
    fn rule(&mut self) {
        self.space(6.0);
        self.page.ops.push(Op::Line { x1: MARGIN, y1: self.y, x2: PAGE_W - MARGIN, y2: self.y });
        self.y += 6.0;
    }
    fn columns(&mut self, cols: &[(f32, String)], size: f32, bold: bool) {
        self.space(size + 3.0);
        for (i, (x, t)) in cols.iter().enumerate() {
            let next = cols.get(i + 1).map(|c| c.0).unwrap_or(PAGE_W - 2.0 * MARGIN);
            let fitted = fit(t, next - x - 4.0, size);
            self.page.ops.push(Op::Text { x: MARGIN + x, y: self.y, size, bold, text: fitted });
        }
        self.y += size + 3.0;
    }
    fn finish(mut self) -> Report {
        self.flush();
        self.report
    }
}

/// Helvetica is about half an em wide per letter, which is close enough for fitting.
pub fn text_width(s: &str, size: f32) -> f32 {
    s.chars().count() as f32 * size * 0.5
}

fn fit(s: &str, width: f32, size: f32) -> String {
    if text_width(s, size) <= width {
        return s.to_string();
    }
    let keep = ((width / (size * 0.5)) as usize).saturating_sub(1);
    let mut out: String = s.chars().take(keep).collect();
    out.push('…');
    out
}

fn wrap(s: &str, width: f32, size: f32) -> Vec<String> {
    let max = ((width / (size * 0.5)) as usize).max(10);
    let mut out = Vec::new();
    for para in s.split('\n') {
        let mut cur = String::new();
        for word in para.split_whitespace() {
            if cur.chars().count() + word.chars().count() + 1 > max && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
        out.push(cur);
    }
    out
}

fn footer(db: &Db) -> String {
    let t = today();
    let mut s = format!("Printed {}", format_date(t.0, t.1, t.2));
    if !db.prefs.prepared_name.trim().is_empty() {
        s.push_str(&format!("    Prepared by {}", db.prefs.prepared_name.trim()));
    }
    s
}

fn event_line(label: &str, e: &Event) -> Option<String> {
    if e.date.trim().is_empty() && e.place.trim().is_empty() {
        return None;
    }
    let mut s = format!("{label}: {}", e.date.trim());
    if !e.place.trim().is_empty() {
        if !e.date.trim().is_empty() {
            s.push_str(", ");
        }
        s.push_str(e.place.trim());
    }
    Some(s)
}

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

fn ordinance_lines(f: &mut Flow, db: &Db, p: &Person) {
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

fn on(d: &str) -> String {
    let d = d.trim();
    if d.is_empty() {
        String::new()
    } else if d.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("on {d}")
    } else {
        d.to_lowercase()
    }
}

fn in_(p: &str) -> String {
    if p.trim().is_empty() { String::new() } else { format!("in {}", p.trim()) }
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

/// Latin-1 text for the built-in PDF fonts, with PDF's special characters escaped.
fn pdf_text(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for c in s.chars() {
        let code = c as u32;
        let byte = match c {
            '(' | ')' | '\\' => {
                out.push(b'\\');
                code as u8
            }
            '…' => 0x85,
            '–' => 0x96,
            '—' => 0x97,
            '’' => 0x92,
            _ if code < 0x20 => b' ',
            _ if code <= 0xFF => code as u8,
            _ => b'?',
        };
        out.push(byte);
    }
    out
}

/// Writes a report as PDF. It uses PDF's built-in fonts, so letters outside Western European
/// alphabets print as a question mark.
pub fn to_pdf(r: &Report) -> Vec<u8> {
    let mut objs: Vec<Vec<u8>> = Vec::new();
    let n = r.pages.len().max(1);
    let content_start = 5;
    let page_start = content_start + n;
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", page_start + i)).collect();
    objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objs.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), n).into_bytes());
    objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
    objs.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec());
    let empty = Page::default();
    for i in 0..n {
        let page = r.pages.get(i).unwrap_or(&empty);
        let mut c: Vec<u8> = Vec::new();
        c.extend_from_slice(b"0.5 w\n");
        for op in &page.ops {
            match op {
                Op::Text { x, y, size, bold, text } => {
                    let font = if *bold { "F2" } else { "F1" };
                    c.extend_from_slice(format!("BT /{font} {size} Tf {x:.1} {:.1} Td (", PAGE_H - y - size).as_bytes());
                    c.extend(pdf_text(text));
                    c.extend_from_slice(b") Tj ET\n");
                }
                Op::Line { x1, y1, x2, y2 } => {
                    c.extend_from_slice(format!("{x1:.1} {:.1} m {x2:.1} {:.1} l S\n", PAGE_H - y1, PAGE_H - y2).as_bytes());
                }
                Op::Rect { x, y, w, h, shade } => {
                    let op = if *shade { "0.9 g" } else { "1 g" };
                    c.extend_from_slice(format!("{op} {x:.1} {:.1} {w:.1} {h:.1} re B 0 g\n", PAGE_H - y - h).as_bytes());
                }
            }
        }
        let mut obj = format!("<< /Length {} >>\nstream\n", c.len()).into_bytes();
        obj.extend(c);
        obj.extend_from_slice(b"\nendstream");
        objs.push(obj);
    }
    for i in 0..n {
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
                content_start + i
            )
            .into_bytes(),
        );
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in &offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[derive(Clone, Debug)]
pub struct WebOptions {
    pub title: String,
    pub hide_living: bool,
    pub include_notes: bool,
    pub who: Option<Vec<u32>>,
}

/// Creates a folder of plain web pages: an index, plus one page for each person.
pub fn write_web(db: &Db, o: &WebOptions, folder: &std::path::Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(folder)?;
    let chosen = |r: u32| o.who.as_ref().map(|l| l.contains(&r)).unwrap_or(true);
    let hidden = |r: u32| o.hide_living && maybe_living(db, r);
    let name_of = |r: u32| if hidden(r) { "Living".to_string() } else { db.name(r) };
    let style = "<style>body{font-family:Georgia,serif;max-width:46em;margin:2em auto;padding:0 1em;line-height:1.5}h1{font-size:1.5em}a{color:#34386a}li{margin:.15em 0}</style>";
    let mut people: Vec<&Person> = db.people.values().filter(|p| chosen(p.rin)).collect();
    people.sort_by_key(|p| sort_key(&p.name));
    let mut index = String::new();
    let _ = write!(index, "<!doctype html><meta charset=\"utf-8\"><title>{0}</title>{style}<h1>{0}</h1><p>{1} people</p><ul>", esc(&o.title), people.len());
    for p in &people {
        let _ = write!(index, "<li><a href=\"p{}.html\">{}</a></li>", p.rin, esc(&name_of(p.rin)));
    }
    index.push_str("</ul>");
    std::fs::write(folder.join("index.html"), index)?;
    let link = |r: u32| -> String {
        if chosen(r) {
            format!("<a href=\"p{r}.html\">{}</a>", esc(&name_of(r)))
        } else {
            esc(&name_of(r))
        }
    };
    let mut count = 0;
    for p in &people {
        let mut h = String::new();
        let _ = write!(h, "<!doctype html><meta charset=\"utf-8\"><title>{0}</title>{style}<p><a href=\"index.html\">Index</a></p><h1>{0}</h1>", esc(&name_of(p.rin)));
        if hidden(p.rin) {
            h.push_str("<p>Details are hidden because this person may still be living.</p>");
        } else {
            for (l, e) in [("Born", &p.birth), ("Christened", &p.chr), ("Died", &p.death), ("Buried", &p.burial)] {
                if let Some(s) = event_line(l, e) {
                    let _ = write!(h, "<p>{}</p>", esc(&s));
                }
            }
            if o.include_notes && !p.notes.trim().is_empty() {
                let _ = write!(h, "<h2>Notes</h2><p>{}</p>", esc(&p.notes).replace('\n', "<br>"));
            }
        }
        if let Some(m) = db.primary_parents(p.rin) {
            h.push_str("<h2>Parents</h2><ul>");
            for r in [m.husband, m.wife].into_iter().flatten() {
                let _ = write!(h, "<li>{}</li>", link(r));
            }
            h.push_str("</ul>");
        }
        for mr in &p.spouses {
            let Some(m) = db.marriages.get(mr) else { continue };
            h.push_str("<h2>Family</h2><ul>");
            if let Some(s) = db.spouse_in(*mr, p.rin) {
                let _ = write!(h, "<li>Spouse: {}</li>", link(s));
            }
            for c in &m.children {
                let _ = write!(h, "<li>Child: {}</li>", link(*c));
            }
            h.push_str("</ul>");
        }
        std::fs::write(folder.join(format!("p{}.html", p.rin)), h)?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_report_builds() {
        let db = sample_family();
        let me = db.home.unwrap();
        assert!(!pedigree(&db, me, 4).pages.is_empty());
        assert!(!family_group(&db, db.person(me).unwrap().spouses[0]).pages.is_empty());
        assert!(!ancestry(&db, me, 5).pages.is_empty());
        assert!(!descendancy(&db, me, 4).pages.is_empty());
        assert!(!individual_summary(&db, me).pages.is_empty());
        assert!(!book(&db, me, 3).pages.is_empty());
        assert!(!calendar(&db, 2026, true).pages.is_empty());
        for k in LIST_KINDS {
            assert!(!list_report(&db, k, None).pages.is_empty());
        }
    }

    #[test]
    fn pdf_is_well_formed() {
        let db = sample_family();
        let pdf = to_pdf(&pedigree(&db, db.home.unwrap(), 4));
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("Thomas Edward Harrow"));
    }

    #[test]
    fn long_reports_paginate() {
        let mut db = Db::default();
        for i in 0..300 {
            db.add_person(Person { name: format!("Person{i} /Test/"), ..Default::default() });
        }
        let r = list_report(&db, "Individuals sorted by RIN", None);
        assert!(r.pages.len() > 2);
        for p in &r.pages {
            for op in &p.ops {
                if let Op::Text { y, .. } = op {
                    assert!(*y <= PAGE_H);
                }
            }
        }
    }

    #[test]
    fn web_hides_the_living() {
        let db = sample_family();
        let dir = std::env::temp_dir().join(format!("fk-web-{}", std::process::id()));
        let n = write_web(&db, &WebOptions { title: "T".into(), hide_living: true, include_notes: true, who: None }, &dir).unwrap();
        assert_eq!(n, db.people.len());
        let index = std::fs::read_to_string(dir.join("index.html")).unwrap();
        assert!(index.contains("Living"));
        assert!(!index.contains("Susan"));
        std::fs::remove_dir_all(dir).ok();
    }
}
