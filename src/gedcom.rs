//! GEDCOM 5.5 import and export.
use crate::calc::maybe_living;
use crate::model::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
struct Node {
    tag: String,
    xref: String,
    value: String,
    kids: Vec<Node>,
}

impl Node {
    fn child(&self, tag: &str) -> Option<&Node> {
        self.kids.iter().find(|k| k.tag == tag)
    }
    fn text(&self, tag: &str) -> String {
        self.child(tag).map(|k| k.full_text()).unwrap_or_default()
    }
    /// The value with its CONT and CONC lines joined back together.
    fn full_text(&self) -> String {
        let mut s = self.value.clone();
        for k in &self.kids {
            match k.tag.as_str() {
                "CONT" => {
                    s.push('\n');
                    s.push_str(&k.value);
                }
                "CONC" => s.push_str(&k.value),
                _ => {}
            }
        }
        s
    }
}

/// Turns the file's bytes into text. Handles UTF-8, UTF-16 (Unicode), ANSEL and ANSI.
pub fn decode(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) || bytes.starts_with(&[0xFE, 0xFF]) {
        let le = bytes[0] == 0xFF;
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| if le { u16::from_le_bytes([c[0], c[1]]) } else { u16::from_be_bytes([c[0], c[1]]) })
            .collect();
        return String::from_utf16_lossy(&units);
    }
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4000)]).to_uppercase();
    if head.contains("1 CHAR UTF-8") || head.contains("1 CHAR UTF8") {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    if head.contains("1 CHAR ANSEL") {
        return ansel_to_text(bytes);
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|b| *b as char).collect(),
    }
}

/// ANSEL puts an accent byte before its letter. This covers the marks and letters in everyday use.
fn ansel_to_text(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut marks: Vec<char> = Vec::new();
    for b in bytes {
        let mark = match *b {
            0xE1 => Some('\u{300}'),
            0xE2 => Some('\u{301}'),
            0xE3 => Some('\u{302}'),
            0xE4 => Some('\u{303}'),
            0xE5 => Some('\u{304}'),
            0xE6 => Some('\u{306}'),
            0xE7 => Some('\u{307}'),
            0xE8 => Some('\u{308}'),
            0xE9 => Some('\u{30C}'),
            0xEA => Some('\u{30A}'),
            0xED => Some('\u{315}'),
            0xEE => Some('\u{30B}'),
            0xF0 => Some('\u{327}'),
            0xF1 => Some('\u{328}'),
            0xF2 => Some('\u{323}'),
            0xF3 => Some('\u{324}'),
            0xF6 => Some('\u{332}'),
            _ => None,
        };
        if let Some(m) = mark {
            marks.push(m);
            continue;
        }
        let ch = match *b {
            0xA1 => 'Ł',
            0xA2 => 'Ø',
            0xA3 => 'Đ',
            0xA4 => 'Þ',
            0xA5 => 'Æ',
            0xA6 => 'Œ',
            0xA7 => 'ʹ',
            0xA8 => '·',
            0xA9 => '♭',
            0xAA => '®',
            0xAB => '±',
            0xB1 => 'ł',
            0xB2 => 'ø',
            0xB3 => 'đ',
            0xB4 => 'þ',
            0xB5 => 'æ',
            0xB6 => 'œ',
            0xB7 => 'ʺ',
            0xB8 => 'ı',
            0xB9 => '£',
            0xBA => 'ð',
            0xC3 => '©',
            0xC5 => '¿',
            0xC6 => '¡',
            0xCF => 'ß',
            b if b < 0x80 => b as char,
            _ => '\u{FFFD}',
        };
        out.push(compose(ch, &marks));
        if compose(ch, &marks) == ch {
            for m in &marks {
                out.push(*m);
            }
        }
        marks.clear();
    }
    out
}

fn compose(c: char, marks: &[char]) -> char {
    if marks.len() != 1 {
        return c;
    }
    let table: &[(char, &str, &str)] = &[
        ('\u{300}', "aeiouAEIOU", "àèìòùÀÈÌÒÙ"),
        ('\u{301}', "aeiouyAEIOUY", "áéíóúýÁÉÍÓÚÝ"),
        ('\u{302}', "aeiouAEIOU", "âêîôûÂÊÎÔÛ"),
        ('\u{303}', "anoANO", "ãñõÃÑÕ"),
        ('\u{308}', "aeiouyAEIOU", "äëïöüÿÄËÏÖÜ"),
        ('\u{30A}', "aAuU", "åÅůŮ"),
        ('\u{327}', "cC", "çÇ"),
    ];
    for (m, from, to) in table {
        if *m == marks[0] {
            if let Some(i) = from.chars().position(|x| x == c) {
                return to.chars().nth(i).unwrap();
            }
        }
    }
    c
}

fn parse_tree(text: &str) -> (Vec<Node>, usize) {
    let mut roots: Vec<Node> = Vec::new();
    let mut stack: Vec<Node> = Vec::new();
    let mut bad = 0;
    fn attach(stack: &mut Vec<Node>, roots: &mut Vec<Node>, level: usize) {
        while stack.len() > level {
            let n = stack.pop().unwrap();
            match stack.last_mut() {
                Some(p) => p.kids.push(n),
                None => roots.push(n),
            }
        }
    }
    for raw in text.lines() {
        let line = raw.trim_start_matches('\u{feff}').trim_start();
        if line.is_empty() {
            continue;
        }
        let mut it = line.splitn(2, ' ');
        let Some(level) = it.next().and_then(|l| l.parse::<usize>().ok()) else {
            // A line without a level number continues the previous value.
            if let Some(top) = stack.last_mut() {
                top.value.push('\n');
                top.value.push_str(raw);
            } else {
                bad += 1;
            }
            continue;
        };
        let rest = it.next().unwrap_or("");
        let (xref, rest) = if rest.starts_with('@') {
            match rest[1..].find('@') {
                Some(end) => (rest[1..end + 1].to_string(), rest[end + 2..].trim_start()),
                None => (String::new(), rest),
            }
        } else {
            (String::new(), rest)
        };
        let mut parts = rest.splitn(2, ' ');
        let tag = parts.next().unwrap_or("").to_uppercase();
        let value = parts.next().unwrap_or("").trim_end_matches('\r').to_string();
        if tag.is_empty() {
            bad += 1;
            continue;
        }
        let keep = level.min(stack.len());
        attach(&mut stack, &mut roots, keep);
        stack.push(Node { tag, xref, value, kids: vec![] });
    }
    attach(&mut stack, &mut roots, 0);
    (roots, bad)
}

fn digits_of(x: &str) -> Option<u32> {
    let d: String = x.chars().filter(|c| c.is_ascii_digit()).collect();
    d.parse().ok().filter(|n| *n > 0)
}

fn pointer(v: &str) -> String {
    v.trim().trim_matches('@').to_string()
}

#[derive(Default, Debug)]
pub struct ImportReport {
    pub people: usize,
    pub marriages: usize,
    pub sources: usize,
    pub repositories: usize,
    pub skipped_tags: BTreeMap<String, usize>,
    pub bad_lines: usize,
}

impl ImportReport {
    pub fn summary(&self) -> String {
        let mut s = format!(
            "Imported {} individuals, {} marriages, {} sources and {} repositories.",
            self.people, self.marriages, self.sources, self.repositories
        );
        if self.bad_lines > 0 {
            s.push_str(&format!("\n{} lines could not be read.", self.bad_lines));
        }
        if !self.skipped_tags.is_empty() {
            let v: Vec<String> = self.skipped_tags.iter().map(|(k, n)| format!("{k} ({n})")).collect();
            s.push_str(&format!("\nNot kept: {}", v.join(", ")));
        }
        s
    }
}

pub fn import(bytes: &[u8]) -> Result<(Db, ImportReport), String> {
    let text = decode(bytes);
    let (roots, bad) = parse_tree(&text);
    if !roots.iter().any(|r| r.tag == "HEAD") {
        return Err("This is not a GEDCOM file: it has no HEAD record.".into());
    }
    if !roots.iter().any(|r| r.tag == "TRLR") {
        return Err("Invalid GEDCOM file: it has no TRLR record, so it may be incomplete.".into());
    }
    let mut db = Db::default();
    let mut report = ImportReport { bad_lines: bad, ..Default::default() };
    let mut notes: BTreeMap<String, String> = BTreeMap::new();
    let mut rins: BTreeMap<String, u32> = BTreeMap::new();
    let mut mrins: BTreeMap<String, u32> = BTreeMap::new();
    let mut srcs: BTreeMap<String, u32> = BTreeMap::new();
    let mut repos: BTreeMap<String, u32> = BTreeMap::new();

    // Keep the numbers from the file when they are free, as the original does for its own exports.
    let mut used = Vec::new();
    for r in &roots {
        match r.tag.as_str() {
            "NOTE" => {
                notes.insert(r.xref.clone(), r.full_text());
            }
            "INDI" => {
                let n = digits_of(&r.xref).filter(|n| !used.contains(n)).unwrap_or(0);
                if n != 0 {
                    used.push(n);
                }
                rins.insert(r.xref.clone(), n);
            }
            _ => {}
        }
    }
    let mut next = used.iter().max().copied().unwrap_or(0);
    for v in rins.values_mut() {
        if *v == 0 {
            next += 1;
            *v = next;
        }
    }
    let mut used_m = Vec::new();
    for r in roots.iter().filter(|r| r.tag == "FAM") {
        let n = digits_of(&r.xref).filter(|n| !used_m.contains(n)).unwrap_or(0);
        if n != 0 {
            used_m.push(n);
        }
        mrins.insert(r.xref.clone(), n);
    }
    let mut next_m = used_m.iter().max().copied().unwrap_or(0);
    for v in mrins.values_mut() {
        if *v == 0 {
            next_m += 1;
            *v = next_m;
        }
    }
    let mut id = 0;
    for r in roots.iter().filter(|r| r.tag == "REPO") {
        id += 1;
        repos.insert(r.xref.clone(), id);
        db.repositories.insert(
            id,
            Repository {
                id,
                name: r.text("NAME"),
                contact: Contact { address: r.text("ADDR"), phone: r.text("PHON"), email: r.text("EMAIL") },
            },
        );
    }
    id = 0;
    for r in roots.iter().filter(|r| r.tag == "SOUR") {
        id += 1;
        srcs.insert(r.xref.clone(), id);
        let repository = r.child("REPO").and_then(|x| repos.get(&pointer(&x.value)).copied());
        let mut comments = r.text("TEXT");
        for n in r.kids.iter().filter(|k| k.tag == "NOTE") {
            let t = note_text(n, &notes);
            if !comments.is_empty() {
                comments.push('\n');
            }
            comments.push_str(&t);
        }
        db.sources.insert(
            id,
            Source {
                id,
                title: r.text("TITL"),
                author: r.text("AUTH"),
                publication: r.text("PUBL"),
                abbreviation: r.text("ABBR"),
                repository,
                comments,
            },
        );
    }

    for r in roots.iter().filter(|r| r.tag == "INDI") {
        let rin = rins[&r.xref];
        let mut p = Person { rin, ..Default::default() };
        let mut names = 0;
        let mut sealings = Vec::new();
        for k in &r.kids {
            match k.tag.as_str() {
                "NAME" => {
                    names += 1;
                    if names == 1 {
                        p.name = k.value.clone();
                        if p.title.is_empty() {
                            p.title = k.text("NPFX");
                        }
                        p.nickname = k.text("NICK");
                        // A file may give only the parts of the name.
                        if p.name.trim().is_empty() {
                            p.name = format!("{} /{}/", k.text("GIVN"), k.text("SURN"));
                        }
                    } else if p.aka.is_empty() {
                        p.aka = plain_name(&k.value);
                    }
                }
                "SEX" => {
                    p.sex = match k.value.trim().to_uppercase().as_str() {
                        "M" => Sex::Male,
                        "F" => Sex::Female,
                        _ => Sex::Unknown,
                    }
                }
                "TITL" => p.title = k.full_text(),
                "NICK" => p.nickname = k.value.clone(),
                "BIRT" => p.birth = event_of(k, &srcs, &mut report),
                "CHR" | "BAPM" => p.chr = event_of(k, &srcs, &mut report),
                "DEAT" => p.death = event_of(k, &srcs, &mut report),
                "BURI" => p.burial = event_of(k, &srcs, &mut report),
                "BAPL" => p.baptism = ordinance_of(k),
                "ENDL" => p.endowment = ordinance_of(k),
                "AFN" => p.afn = k.value.clone(),
                "NOTE" => {
                    if !p.notes.is_empty() {
                        p.notes.push('\n');
                    }
                    p.notes.push_str(&note_text(k, &notes));
                }
                "SOUR" => p.citations.push(citation_of(k, &srcs)),
                "ADDR" => p.contact.address = k.full_text(),
                "PHON" => p.contact.phone = k.value.clone(),
                "EMAIL" | "_EMAIL" => p.contact.email = k.value.clone(),
                "RESI" => {
                    if let Some(a) = k.child("ADDR") {
                        p.contact.address = a.full_text();
                    }
                    if p.contact.phone.is_empty() {
                        p.contact.phone = k.text("PHON");
                    }
                    if p.contact.email.is_empty() {
                        p.contact.email = k.text("EMAIL");
                    }
                }
                "OBJE" => p.media.push(media_of(k)),
                "FAMS" => {
                    if let Some(m) = mrins.get(&pointer(&k.value)) {
                        if !p.spouses.contains(m) {
                            p.spouses.push(*m);
                        }
                    }
                }
                "FAMC" => {
                    if let Some(m) = mrins.get(&pointer(&k.value)) {
                        if !p.parents.iter().any(|l| l.mrin == *m) {
                            let rel = match k.text("PEDI").to_lowercase().as_str() {
                                "adopted" => "Adopted",
                                "foster" | "guardian" => "Guardian",
                                "sealing" => "LDS Sealing",
                                _ => "Biological",
                            };
                            p.parents.push(ParentLink { mrin: *m, relation: rel.into(), sealed: Ordinance::default() });
                        }
                    }
                }
                "SLGC" => {
                    let target = k.child("FAMC").and_then(|f| mrins.get(&pointer(&f.value)).copied());
                    sealings.push((target, ordinance_of(k)));
                }
                "CHAN" => p.changed = k.child("DATE").map(|d| d.value.clone()).unwrap_or_default(),
                "EVEN" => p.custom.push(custom_of(k, "Event")),
                "OCCU" | "RELI" | "EDUC" | "NATI" | "CAST" | "PROP" | "CENS" | "IMMI" | "EMIG" | "NATU" | "GRAD" | "RETI" | "PROB" | "WILL" | "ADOP"
                | "BARM" | "BASM" | "BLES" | "CHRA" | "CONF" | "FCOM" | "ORDN" | "DSCR" | "IDNO" | "NCHI" | "NMR" | "SSN" | "_UID" | "RIN" | "REFN" | "RFN" => {
                    if matches!(k.tag.as_str(), "_UID" | "RIN" | "REFN" | "RFN") {
                        continue;
                    }
                    p.custom.push(custom_of(k, event_label(&k.tag)));
                }
                other => *report.skipped_tags.entry(other.to_string()).or_default() += 1,
            }
        }
        // A sealing can come before the parent link it belongs to, so apply them last.
        for (target, ord) in sealings {
            if let Some(l) = p.parents.iter_mut().find(|l| target.is_none() || Some(l.mrin) == target) {
                l.sealed = ord;
            }
        }
        if p.name.trim() == "//" {
            p.name.clear();
        }
        db.people.insert(rin, p);
        report.people += 1;
    }

    for r in roots.iter().filter(|r| r.tag == "FAM") {
        let mrin = mrins[&r.xref];
        let mut m = Marriage { mrin, ..Default::default() };
        for k in &r.kids {
            match k.tag.as_str() {
                "HUSB" => m.husband = rins.get(&pointer(&k.value)).copied(),
                "WIFE" => m.wife = rins.get(&pointer(&k.value)).copied(),
                "CHIL" => {
                    if let Some(c) = rins.get(&pointer(&k.value)) {
                        if !m.children.contains(c) {
                            m.children.push(*c);
                        }
                    }
                }
                "MARR" => m.event = event_of(k, &srcs, &mut report),
                "SLGS" => m.sealing = ordinance_of(k),
                "DIV" => m.divorced = true,
                "NOTE" => {
                    if !m.notes.is_empty() {
                        m.notes.push('\n');
                    }
                    m.notes.push_str(&note_text(k, &notes));
                }
                "SOUR" => m.citations.push(citation_of(k, &srcs)),
                "EVEN" => m.custom.push(custom_of(k, "Event")),
                "CENS" | "ENGA" | "MARB" | "MARC" | "MARL" | "MARS" | "ANUL" | "DIVF" => m.custom.push(custom_of(k, event_label(&k.tag))),
                "_UID" | "RIN" | "REFN" | "CHAN" | "NCHI" => {}
                other => *report.skipped_tags.entry(other.to_string()).or_default() += 1,
            }
        }
        db.marriages.insert(mrin, m);
        report.marriages += 1;
    }
    report.sources = db.sources.len();
    report.repositories = db.repositories.len();
    // Mend links that a file gave in only one direction.
    db.check(true);
    db.home = db.people.keys().next().copied();
    Ok((db, report))
}

fn event_label(tag: &str) -> &'static str {
    match tag {
        "OCCU" => "Occupation",
        "RELI" => "Religion",
        "EDUC" => "Education",
        "NATI" => "Nationality",
        "CAST" => "Caste",
        "PROP" => "Property",
        "CENS" => "Census",
        "IMMI" => "Immigration",
        "EMIG" => "Emigration",
        "NATU" => "Naturalization",
        "GRAD" => "Graduation",
        "RETI" => "Retirement",
        "PROB" => "Probate",
        "WILL" => "Will",
        "ADOP" => "Adoption",
        "BARM" => "Bar Mitzvah",
        "BASM" => "Bas Mitzvah",
        "BLES" => "Blessing",
        "CHRA" => "Adult Christening",
        "CONF" => "Confirmation",
        "FCOM" => "First Communion",
        "ORDN" => "Ordination",
        "DSCR" => "Physical Description",
        "IDNO" => "National ID",
        "NCHI" => "Number of Children",
        "NMR" => "Number of Marriages",
        "SSN" => "Social Security Number",
        "ENGA" => "Engagement",
        "MARB" => "Marriage Banns",
        "MARC" => "Marriage Contract",
        "MARL" => "Marriage License",
        "MARS" => "Marriage Settlement",
        "ANUL" => "Annulment",
        "DIVF" => "Divorce Filed",
        _ => "Event",
    }
}

fn tag_for_label(label: &str) -> Option<&'static str> {
    const TAGS: [&str; 34] = [
        "OCCU", "RELI", "EDUC", "NATI", "CAST", "PROP", "CENS", "IMMI", "EMIG", "NATU", "GRAD", "RETI", "PROB", "WILL", "ADOP", "BARM", "BASM", "BLES",
        "CHRA", "CONF", "FCOM", "ORDN", "DSCR", "IDNO", "NCHI", "NMR", "SSN", "ENGA", "MARB", "MARC", "MARL", "MARS", "ANUL", "DIVF",
    ];
    TAGS.into_iter().find(|t| event_label(t) == label)
}

fn note_text(n: &Node, notes: &BTreeMap<String, String>) -> String {
    let v = n.value.trim();
    if v.starts_with('@') && v.ends_with('@') && v.len() > 2 {
        notes.get(&pointer(v)).cloned().unwrap_or_default()
    } else {
        n.full_text()
    }
}

fn citation_of(k: &Node, srcs: &BTreeMap<String, u32>) -> Citation {
    Citation {
        source: srcs.get(&pointer(&k.value)).copied().unwrap_or(0),
        page: k.text("PAGE"),
        actual_text: k.child("DATA").map(|d| d.text("TEXT")).unwrap_or_default(),
        comments: k.text("NOTE"),
    }
}

fn event_of(k: &Node, srcs: &BTreeMap<String, u32>, _r: &mut ImportReport) -> Event {
    Event {
        date: k.text("DATE"),
        place: k.text("PLAC"),
        citations: k.kids.iter().filter(|x| x.tag == "SOUR").map(|x| citation_of(x, srcs)).filter(|c| c.source != 0).collect(),
    }
}

fn ordinance_of(k: &Node) -> Ordinance {
    let mut date = k.text("DATE");
    if date.is_empty() {
        date = k.text("STAT");
    }
    Ordinance { date, temple: k.text("TEMP") }
}

fn custom_of(k: &Node, label: &str) -> CustomEvent {
    let kind = if k.tag == "EVEN" && !k.text("TYPE").is_empty() { k.text("TYPE") } else { label.to_string() };
    CustomEvent { kind, date: k.text("DATE"), place: k.text("PLAC"), description: k.full_text() }
}

fn media_of(k: &Node) -> Media {
    let file = k.child("FILE");
    Media {
        file: k.text("FILE"),
        caption: file.map(|f| f.text("TITL")).filter(|t| !t.is_empty()).unwrap_or_else(|| k.text("TITL")),
        description: k.text("NOTE"),
        in_scrapbook: false,
    }
}

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
    let t = crate::calc::today();
    w.raw(0, "HEAD", "");
    w.raw(1, "SOUR", program);
    w.raw(2, "VERS", env!("CARGO_PKG_VERSION"));
    w.raw(2, "NAME", program);
    w.raw(1, "DATE", &crate::calc::format_date(t.0, t.1, t.2));
    w.raw(1, "GEDC", "");
    w.raw(2, "VERS", "5.5");
    w.raw(2, "FORM", "LINEAGE-LINKED");
    w.raw(1, "CHAR", "UTF-8");
    w.raw(1, "SUBM", "@SUB1@");
    w.raw(0, "@SUB1@ SUBM", "");
    w.line(1, "NAME", if db.prefs.prepared_name.trim().is_empty() { "Unknown" } else { db.prefs.prepared_name.trim() });

    for p in db.people.values().filter(|p| included(p.rin)) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_then_import_keeps_everything() {
        let mut db = sample_family();
        let me = db.home.unwrap();
        db.sources.insert(1, Source { id: 1, title: "Parish register".into(), repository: Some(1), ..Default::default() });
        db.repositories.insert(1, Repository { id: 1, name: "County archive".into(), contact: Contact::default() });
        {
            let p = db.people.get_mut(&me).unwrap();
            p.birth.citations.push(Citation { source: 1, page: "p. 12".into(), ..Default::default() });
            p.notes = "First line\nSecond line ".to_string() + &"long ".repeat(100);
            p.custom.push(CustomEvent { kind: "Occupation".into(), date: "1950".into(), place: "Boise".into(), description: "Teacher".into() });
            p.custom.push(CustomEvent { kind: "Elected mayor".into(), date: "1961".into(), place: String::new(), description: "Town hall".into() });
            p.baptism = Ordinance { date: "1 Jan 1930".into(), temple: "SLAKE".into() };
            p.parents[0].relation = "Adopted".into();
            p.parents[0].sealed = Ordinance { date: "2 Feb 1931".into(), temple: "LOGAN".into() };
            p.media.push(Media { file: "photo.jpg".into(), caption: "Portrait".into(), ..Default::default() });
        }
        let text = export(&db, &ExportOptions::default(), "PAF Classic");
        let (back, report) = import(text.as_bytes()).unwrap();
        assert_eq!(report.people, db.people.len());
        assert_eq!(report.marriages, db.marriages.len());
        assert!(report.skipped_tags.is_empty(), "{:?}", report.skipped_tags);
        assert_eq!(back.marriages, db.marriages);
        for (rin, a) in &db.people {
            let b = &back.people[rin];
            assert_eq!(a.name, b.name);
            assert_eq!(a.sex, b.sex);
            assert_eq!(a.birth, b.birth);
            assert_eq!(a.death, b.death);
            assert_eq!(a.spouses, b.spouses);
            assert_eq!(a.parents, b.parents);
            assert_eq!(a.notes, b.notes);
            assert_eq!(a.custom, b.custom);
            assert_eq!(a.media, b.media);
        }
        assert_eq!(back.sources, db.sources);
    }

    #[test]
    fn reads_other_programs_files() {
        let text = "0 HEAD\n1 CHAR ANSEL\n0 @P1@ INDI\n1 NAME Ann /Lee/\n1 SEX F\n1 BIRT\n2 DATE 1 JAN 1900\n1 FAMC @X@\n1 _ODD thing\n0 @P2@ INDI\n1 NAME Joe /Lee/\n0 @X@ FAM\n1 HUSB @P2@\n1 CHIL @P1@\n0 TRLR\n";
        let (db, r) = import(text.as_bytes()).unwrap();
        assert_eq!(db.people.len(), 2);
        assert_eq!(db.father(1), Some(2));
        // Joe's FAMS line was missing, and the import mends it.
        assert_eq!(db.person(2).unwrap().spouses.len(), 1);
        assert_eq!(r.skipped_tags.get("_ODD"), Some(&1));
    }

    #[test]
    fn ansel_accents() {
        let bytes = [b'J', 0xE2, b'o', b's', 0xE2, b'e'];
        assert_eq!(ansel_to_text(&bytes), "Jósé");
    }

    #[test]
    fn privacy_hides_living() {
        let db = sample_family();
        let opts = ExportOptions { full_living: false, names_on_living: false, ..Default::default() };
        let text = export(&db, &opts, "PAF Classic");
        assert!(text.contains("1 NAME LIVING"));
        assert!(!text.contains("Susan"));
        assert!(text.contains("Thomas Edward"));
    }

    #[test]
    fn rejects_incomplete() {
        assert!(import(b"0 HEAD\n").is_err());
        assert!(import(b"hello\n").is_err());
    }
}
