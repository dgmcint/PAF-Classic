//! Writing a family file out as GEDCOM.

use super::*;
use crate::rules::maybe_living;
use crate::data::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ExportOptions {
    pub notes: bool,
    pub sources: bool,
    pub media: bool,
    pub contact: bool,
    pub lds: bool,
    pub full_living: bool,
    pub names_on_living: bool,
    pub subset: Option<Vec<u32>>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            notes: true,
            sources: true,
            media: true,
            contact: true,
            lds: true,
            full_living: true,
            names_on_living: true,
            subset: None,
        }
    }
}

struct W {
    out: String,
}

impl W {
    /// Writes a value, breaking lines with CONT and long lines with CONC at GEDCOM's 248-character limit.
    fn line(&mut self, level: usize, tag: &str, value: &str) {
        let mut lines = value.split('\n');
        let first = lines.next().unwrap_or("");
        self.chunked(level, tag, first);
        for rest in lines {
            self.chunked(level + 1, "CONT", rest);
        }
    }
    fn chunked(&mut self, level: usize, tag: &str, value: &str) {
        let value = value.trim_end_matches('\r');
        const MAX: usize = 200;
        if value.chars().count() <= MAX {
            self.raw(level, tag, value);
            return;
        }
        let chars: Vec<char> = value.chars().collect();
        let mut i = 0;
        let mut tag_now = tag;
        let mut lvl = level;
        while i < chars.len() {
            let mut end = (i + MAX).min(chars.len());
            // A CONC must not start or end on a space, or the space can be lost.
            while end < chars.len() && end > i + 1 && (chars[end] == ' ' || chars[end - 1] == ' ') {
                end -= 1;
            }
            let part: String = chars[i..end].iter().collect();
            self.raw(lvl, tag_now, &part);
            // CONC sits one level under a normal tag, but level with a CONT it continues.
            if tag_now != "CONC" {
                if tag_now != "CONT" {
                    lvl += 1;
                }
                tag_now = "CONC";
            }
            i = end;
        }
    }
    fn raw(&mut self, level: usize, tag: &str, value: &str) {
        if value.is_empty() {
            self.out.push_str(&format!("{level} {tag}\r\n"));
        } else {
            self.out.push_str(&format!("{level} {tag} {value}\r\n"));
        }
    }
    fn citations(&mut self, level: usize, cites: &[Citation], o: &ExportOptions) {
        if !o.sources {
            return;
        }
        for c in cites {
            self.line(level, "SOUR", &format!("@S{}@", c.source));
            if !c.page.is_empty() {
                self.line(level + 1, "PAGE", &c.page);
            }
            if !c.actual_text.is_empty() {
                self.raw(level + 1, "DATA", "");
                self.line(level + 2, "TEXT", &c.actual_text);
            }
            if !c.comments.is_empty() {
                self.line(level + 1, "NOTE", &c.comments);
            }
        }
    }
    fn event(&mut self, tag: &str, e: &Event, o: &ExportOptions) {
        if e.is_empty() {
            return;
        }
        self.raw(1, tag, "");
        if !e.date.trim().is_empty() {
            self.line(2, "DATE", e.date.trim());
        }
        if !e.place.trim().is_empty() {
            self.line(2, "PLAC", e.place.trim());
        }
        self.citations(2, &e.citations, o);
    }
    fn ordinance(&mut self, level: usize, tag: &str, x: &Ordinance) {
        if x.is_empty() {
            return;
        }
        self.raw(level, tag, "");
        if !x.date.trim().is_empty() {
            self.line(level + 1, "DATE", x.date.trim());
        }
        if !x.temple.trim().is_empty() {
            self.line(level + 1, "TEMP", x.temple.trim());
        }
    }
    fn custom(&mut self, c: &CustomEvent) {
        match tag_for_label(&c.kind) {
            Some(t) => self.line(1, t, &c.description),
            None => {
                self.line(1, "EVEN", &c.description);
                self.line(2, "TYPE", &c.kind);
            }
        }
        if !c.date.trim().is_empty() {
            self.line(2, "DATE", c.date.trim());
        }
        if !c.place.trim().is_empty() {
            self.line(2, "PLAC", c.place.trim());
        }
    }
}

pub fn export(db: &Db, o: &ExportOptions, program: &str) -> String {
    let mut w = W { out: String::new() };
    let included = |r: u32| o.subset.as_ref().map(|s| s.contains(&r)).unwrap_or(true);
    let t = crate::rules::today();
    w.raw(0, "HEAD", "");
    w.raw(1, "SOUR", program);
    w.raw(2, "VERS", env!("CARGO_PKG_VERSION"));
    w.raw(2, "NAME", program);
    w.raw(1, "DATE", &crate::rules::format_date(t.0, t.1, t.2));
    w.raw(1, "GEDC", "");
    w.raw(2, "VERS", "5.5");
    w.raw(2, "FORM", "LINEAGE-LINKED");
    w.raw(1, "CHAR", "UTF-8");
    let home = db.home.filter(|h| db.people.contains_key(h) && included(*h));
    if let Some(h) = home {
        // Read back by this program and Gilded Roots, so the home person stays the same.
        w.raw(1, "_HOME", &format!("@I{h}@"));
    }
    w.raw(1, "SUBM", "@SUB1@");
    w.raw(0, "@SUB1@ SUBM", "");
    w.line(1, "NAME", if db.prefs.prepared_name.trim().is_empty() { "Unknown" } else { db.prefs.prepared_name.trim() });

    // The home person comes first, so other programs that take the first person as home agree.
    let first = home.and_then(|h| db.people.get(&h));
    for p in first.into_iter().chain(db.people.values().filter(|p| Some(p.rin) != home)).filter(|p| included(p.rin)) {
        w.raw(0, &format!("@I{}@ INDI", p.rin), "");
        let hide = !o.full_living && maybe_living(db, p.rin);
        if hide {
            if o.names_on_living {
                w.line(1, "NAME", &p.name);
            } else {
                w.raw(1, "NAME", "LIVING");
            }
            w.raw(1, "SEX", p.sex.letter());
        } else {
            w.line(1, "NAME", &p.name);
            let (giv, sur, suf) = (given_of(&p.name), surname_of(&p.name), suffix_of(&p.name));
            if !giv.is_empty() {
                w.line(2, "GIVN", &giv);
            }
            if !sur.is_empty() {
                w.line(2, "SURN", &sur);
            }
            if !suf.is_empty() {
                w.line(2, "NSFX", &suf);
            }
            if !p.nickname.is_empty() {
                w.line(2, "NICK", &p.nickname);
            }
            if !p.aka.is_empty() {
                w.line(1, "NAME", &p.aka);
            }
            if !p.title.is_empty() {
                w.line(1, "TITL", &p.title);
            }
            w.raw(1, "SEX", p.sex.letter());
            w.event("BIRT", &p.birth, o);
            w.event("CHR", &p.chr, o);
            w.event("DEAT", &p.death, o);
            w.event("BURI", &p.burial, o);
            if o.lds {
                w.ordinance(1, "BAPL", &p.baptism);
                w.ordinance(1, "ENDL", &p.endowment);
                for l in &p.parents {
                    if !l.sealed.is_empty() {
                        w.ordinance(1, "SLGC", &l.sealed);
                        w.line(2, "FAMC", &format!("@F{}@", l.mrin));
                    }
                }
            }
            for c in &p.custom {
                w.custom(c);
            }
            if !p.afn.is_empty() {
                w.line(1, "AFN", &p.afn);
            }
            if o.contact && !p.contact.is_empty() {
                if !p.contact.address.is_empty() {
                    w.line(1, "ADDR", &p.contact.address);
                }
                if !p.contact.phone.is_empty() {
                    w.line(1, "PHON", &p.contact.phone);
                }
                if !p.contact.email.is_empty() {
                    w.line(1, "EMAIL", &p.contact.email);
                }
            }
            w.citations(1, &p.citations, o);
            if o.notes && !p.notes.trim().is_empty() {
                w.line(1, "NOTE", &p.notes);
            }
            if o.media {
                for m in &p.media {
                    w.raw(1, "OBJE", "");
                    w.line(2, "FILE", &m.file);
                    if !m.caption.is_empty() {
                        w.line(2, "TITL", &m.caption);
                    }
                    if !m.description.is_empty() {
                        w.line(2, "NOTE", &m.description);
                    }
                }
            }
        }
        for l in &p.parents {
            if db.marriages.contains_key(&l.mrin) {
                w.line(1, "FAMC", &format!("@F{}@", l.mrin));
                if l.relation != "Biological" {
                    let v = match l.relation.as_str() {
                        "Adopted" => "adopted",
                        "Guardian" => "foster",
                        "LDS Sealing" => "sealing",
                        other => other,
                    };
                    w.line(2, "PEDI", v);
                }
            }
        }
        for m in &p.spouses {
            if db.marriages.contains_key(m) {
                w.line(1, "FAMS", &format!("@F{m}@"));
            }
        }
        if !p.old_id.is_empty() {
            w.line(1, "REFN", &p.old_id);
            w.raw(2, "TYPE", "Old ID");
        }
    }

    for m in db.marriages.values() {
        let h = m.husband.filter(|r| included(*r));
        let f = m.wife.filter(|r| included(*r));
        let kids: Vec<u32> = m.children.iter().copied().filter(|r| included(*r)).collect();
        if h.is_none() && f.is_none() && kids.is_empty() {
            continue;
        }
        w.raw(0, &format!("@F{}@ FAM", m.mrin), "");
        if let Some(r) = h {
            w.line(1, "HUSB", &format!("@I{r}@"));
        }
        if let Some(r) = f {
            w.line(1, "WIFE", &format!("@I{r}@"));
        }
        for c in &kids {
            w.line(1, "CHIL", &format!("@I{c}@"));
        }
        let hide = !o.full_living && [m.husband, m.wife].into_iter().flatten().any(|r| maybe_living(db, r));
        if !hide {
            w.event("MARR", &m.event, o);
            if m.divorced {
                w.raw(1, "DIV", "Y");
            }
            if o.lds {
                w.ordinance(1, "SLGS", &m.sealing);
            }
            for c in &m.custom {
                w.custom(c);
            }
            w.citations(1, &m.citations, o);
            if o.notes && !m.notes.trim().is_empty() {
                w.line(1, "NOTE", &m.notes);
            }
        }
    }
    if o.sources {
        for s in db.sources.values() {
            w.raw(0, &format!("@S{}@ SOUR", s.id), "");
            if !s.title.is_empty() {
                w.line(1, "TITL", &s.title);
            }
            if !s.author.is_empty() {
                w.line(1, "AUTH", &s.author);
            }
            if !s.publication.is_empty() {
                w.line(1, "PUBL", &s.publication);
            }
            if !s.abbreviation.is_empty() {
                w.line(1, "ABBR", &s.abbreviation);
            }
            if let Some(r) = s.repository {
                w.line(1, "REPO", &format!("@R{r}@"));
            }
            if !s.comments.is_empty() {
                w.line(1, "NOTE", &s.comments);
            }
        }
        for r in db.repositories.values() {
            w.raw(0, &format!("@R{}@ REPO", r.id), "");
            w.line(1, "NAME", &r.name);
            if !r.contact.address.is_empty() {
                w.line(1, "ADDR", &r.contact.address);
            }
            if !r.contact.phone.is_empty() {
                w.line(1, "PHON", &r.contact.phone);
            }
            if !r.contact.email.is_empty() {
                w.line(1, "EMAIL", &r.contact.email);
            }
        }
    }
    w.raw(0, "TRLR", "");
    w.out
}
