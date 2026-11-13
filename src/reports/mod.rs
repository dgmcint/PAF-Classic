//! Printable reports: page layout shared by every report.

mod charts;
mod lists;
mod pdf;
mod web;

pub use charts::*;
pub use lists::*;
pub use pdf::*;
pub use web::*;

use crate::rules::*;
use crate::data::*;

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
pub(crate) struct Flow {
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
