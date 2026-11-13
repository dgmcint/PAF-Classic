//! Reading a GEDCOM file into a family file.

use super::*;
use crate::data::*;
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

/// Record numbers for the records of one kind, in file order. The numbers in the file's own IDs
/// are kept when they look like record numbers (all different and no bigger than ten times the
/// number of records), as in files from PAF and from this program. Otherwise, as with the long
/// IDs of Ancestry and other websites, the records are numbered 1, 2, 3... in the order of the file.
fn record_numbers(xrefs: &[&str]) -> BTreeMap<String, u32> {
    let limit = (xrefs.len() as u64).saturating_mul(10).max(100);
    let mut seen = std::collections::HashSet::new();
    let found: Vec<Option<u32>> = xrefs.iter().map(|x| digits_of(x)).collect();
    let keep = found.iter().all(|n| n.is_some_and(|n| (n as u64) <= limit && seen.insert(n)));
    xrefs
        .iter()
        .enumerate()
        .map(|(i, x)| (x.to_string(), if keep { found[i].unwrap() } else { i as u32 + 1 }))
        .collect()
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
    let mut srcs: BTreeMap<String, u32> = BTreeMap::new();
    let mut repos: BTreeMap<String, u32> = BTreeMap::new();

    for r in roots.iter().filter(|r| r.tag == "NOTE") {
        notes.insert(r.xref.clone(), r.full_text());
    }
    let people: Vec<&str> = roots.iter().filter(|r| r.tag == "INDI").map(|r| r.xref.as_str()).collect();
    let fams: Vec<&str> = roots.iter().filter(|r| r.tag == "FAM").map(|r| r.xref.as_str()).collect();
    let rins = record_numbers(&people);
    let mrins = record_numbers(&fams);
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
                    if k.tag == "REFN" && k.text("TYPE").eq_ignore_ascii_case("Old ID") {
                        p.old_id = k.value.trim().to_string();
                    }
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
    // The home person: the one the file names in its header (as this program and Gilded Roots
    // write it), otherwise the first person in the file, as PAF, Ancestry and FamilySearch do.
    let head = roots.iter().find(|r| r.tag == "HEAD");
    let named = head.and_then(|h| h.child("_HOME")).and_then(|k| rins.get(&pointer(&k.value)).copied());
    db.home = named.or_else(|| people.first().and_then(|x| rins.get(*x).copied()));
    Ok((db, report))
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
