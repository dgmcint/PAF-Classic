//! Read-only recovery of data from the original program's .paf files.
//!
//! Worked out from the file layout and checked on a real family file, so the results are still
//! checked hard and every import says it needs verifying. It never writes to
//! the .paf file.
use crate::model::*;

const BLOCK: usize = 0x2000;
const FIRST_MAP: u64 = 0x3000;
const MASTER_MAP: usize = 0x1000;
const HEADER_LEN: usize = 0x695;
const MAX_PAGES: usize = 31;

const TYPE_PERSON: u8 = 2;
const TYPE_MARRIAGE: u8 = 3;
const TYPE_LINK: u8 = 5;
const TYPE_NOTE: u8 = 4;
const TYPE_CONTACT: u8 = 0x11;

// Record sizes in bytes.
const PERSON_SIZE: usize = 221;
const MARRIAGE_SIZE: usize = 106;
const NOTE_SIZE: usize = 256;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    Current,
    OlderFive,
    Paf4,
    Paf3,
    Other,
}

pub fn identify(d: &[u8]) -> Kind {
    if d.len() < 14 {
        return Kind::Other;
    }
    let at = |o: usize, s: &[u8]| d[o..o + s.len()] == *s;
    if at(0, b"500") && at(8, b"PAF") {
        return if u16::from_le_bytes([d[12], d[13]]) == 4 { Kind::Current } else { Kind::OlderFive };
    }
    if at(0, b"PAF") {
        // Two version bytes: version 3 when both say 3, version 4 when either says 4.
        return match (d[4], d[8]) {
            (b'3', b'3') => Kind::Paf3,
            (b'3' | b'4', b'4') | (b'4', b'3') => Kind::Paf4,
            _ => Kind::Other,
        };
    }
    Kind::Other
}

struct Reader<'a> {
    d: &'a [u8],
    tables: Vec<Vec<i16>>,
}

fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}

impl<'a> Reader<'a> {
    fn new(d: &'a [u8]) -> Option<Self> {
        let mut r = Reader { d, tables: Vec::new() };
        let pages = u32_at(d, 0x12)? as usize;
        if pages == 0 || pages > MAX_PAGES {
            return None;
        }
        for page in 0..pages {
            let start = FIRST_MAP as usize + (page << 26);
            let map = d.get(start..start + BLOCK)?;
            // The first byte marks the block as a map.
            if map[0] != 0x0b {
                return None;
            }
            // Group each page's blocks by record type.
            let mut counts = [0usize; 32];
            for b in map.iter() {
                if (1..=32).contains(b) {
                    counts[*b as usize - 1] += 1;
                }
            }
            let mut table = vec![0i16; 32 + BLOCK];
            let mut at = 32usize;
            for (t, c) in counts.iter().enumerate() {
                table[t] = at as i16;
                at += c;
            }
            let mut fill = [0usize; 32];
            for (i, b) in map.iter().enumerate().skip(1) {
                if (1..=32).contains(b) {
                    let t = *b as usize - 1;
                    let pos = table[t] as usize + fill[t];
                    fill[t] += 1;
                    if pos < table.len() {
                        table[pos] = i as i16;
                    }
                }
            }
            r.tables.push(table);
        }
        Some(r)
    }

    fn master_count(&self, page: usize, t: u8) -> i64 {
        let o = MASTER_MAP + 2 * (page * 32 + t as usize - 1);
        match self.d.get(o..o + 2) {
            Some(b) => i16::from_le_bytes([b[0], b[1]]) as i64,
            None => 0,
        }
    }

    /// Finds record `rec` of a type.
    fn record(&self, t: u8, size: usize, per_block: usize, rec: u32) -> Option<&'a [u8]> {
        if rec < 1 {
            return None;
        }
        let mut left = rec as i64;
        let mut page = 0usize;
        loop {
            let here = self.master_count(page, t) * per_block as i64;
            if left <= here {
                break;
            }
            left -= here;
            page += 1;
            if page >= self.tables.len() {
                return None;
            }
        }
        let table = self.tables.get(page)?;
        let index = table[t as usize - 1] as i64 + (left - 1) / per_block as i64;
        let block = *table.get(index as usize)? as i64;
        if block < 1 || block > BLOCK as i64 {
            return None;
        }
        let slot = ((left - 1) % per_block as i64) as usize;
        let offset = ((((page as u64) << 13) + block as u64) << 13) + FIRST_MAP + (slot * size) as u64;
        self.d.get(offset as usize..offset as usize + size)
    }

    /// Text kept outside the fixed records. The text starts `skip` bytes in.
    fn text(&self, offset: u32, skip: usize) -> String {
        if offset == 0 {
            return String::new();
        }
        let start = offset as usize + skip;
        let Some(tail) = self.d.get(start..(start + 0x100).min(self.d.len())) else { return String::new() };
        let end = tail.iter().position(|b| *b == 0).unwrap_or(tail.len());
        clean(&String::from_utf8_lossy(&tail[..end]))
    }

    /// Note text is a chain of records: +0 is the next one's number and 243 bytes of text start at
    /// +0xd. The bytes are joined first, because a letter can be split across records.
    fn notes(&self, first: u32) -> String {
        let (mut at, mut hops, mut bytes) = (first, 0, Vec::new());
        while at > 0 && hops < 4096 {
            let Some(rec) = self.record(TYPE_NOTE, NOTE_SIZE, 0x20, at) else { break };
            let text = &rec[0xd..];
            let end = text.iter().position(|b| *b == 0).unwrap_or(text.len());
            bytes.extend_from_slice(&text[..end]);
            at = u32_at(rec, 0).unwrap_or(0);
            hops += 1;
        }
        String::from_utf8_lossy(&bytes).replace("\r\n", "\n").trim().to_string()
    }

    /// Contact details: a record of ten text positions from +4. The original's fields
    /// are in the order of its Contact Information window.
    fn contact(&self, number: u32) -> Contact {
        let mut c = Contact::default();
        let Some(rec) = (number > 0).then(|| self.record(TYPE_CONTACT, 0x39, 0x8f, number)).flatten() else { return c };
        let field = |i: usize| self.text(u32_at(rec, 4 + i * 4).unwrap_or(0), 0xd);
        let lines: Vec<String> = (0..7).map(field).filter(|l| !l.is_empty()).collect();
        c.address = lines.join("\n");
        c.phone = field(7);
        c.email = field(8);
        c
    }
}

fn clean(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).collect::<String>().trim().to_string()
}

#[derive(Debug)]
pub struct PafImport {
    pub db: Db,
    pub summary: String,
}

/// Reads a whole file. Refuses, with a reason, rather than return doubtful family data.
pub fn import(d: &[u8]) -> Result<PafImport, String> {
    match identify(d) {
        Kind::Current | Kind::OlderFive => {}
        Kind::Paf4 | Kind::Paf3 => {
            return Err("This .paf file was made by version 3 or 4 of the original program. This version can only read files from version 5. Export it as GEDCOM from the original program, then import that.".into());
        }
        Kind::Other => return Err("This file is not a valid family file. It may be corrupt, or an old version 2 file.".into()),
    }
    if d.len() < HEADER_LEN + BLOCK {
        return Err("This .paf file is too short to be complete.".into());
    }
    let r = Reader::new(d).ok_or("This .paf file's block map could not be read, so the file was not imported.")?;
    let people = u32_at(d, 0x16).unwrap_or(0);
    let marriages = u32_at(d, 0x82).unwrap_or(0);
    let links = u32_at(d, 0xe6).unwrap_or(0);
    if people > 2_000_000 || marriages > 2_000_000 {
        return Err("This .paf file's record counts look wrong, so it was not imported.".into());
    }
    let mut db = Db::default();
    let mut unreadable = 0usize;
    let mut unnamed = 0usize;
    for rin in 1..=people {
        let Some(rec) = r.record(TYPE_PERSON, PERSON_SIZE, 0x25, rin) else {
            unreadable += 1;
            continue;
        };
        // 'D' marks a deleted record.
        if rec[0xca] == b'D' {
            continue;
        }
        let mut p = Person { rin, ..Default::default() };
        // Names are a chain: each holds the next one's position, text at +0xf.
        let mut at = u32_at(rec, 0).unwrap_or(0);
        let mut names = Vec::new();
        let mut hops = 0;
        while at > 0 && hops < 16 {
            let Some(head) = d.get(at as usize..at as usize + 0x10) else { break };
            names.push(r.text(at, 0xf));
            at = u32_at(head, 0).unwrap_or(0);
            hops += 1;
        }
        let mut names = names.into_iter().filter(|n| !n.is_empty());
        p.name = names.next().unwrap_or_default();
        p.aka = names.next().map(|n| plain_name(&n)).unwrap_or_default();
        if p.name.is_empty() {
            unnamed += 1;
        }
        // The same byte holds M, F or D for deleted.
        p.sex = match rec[0xca] {
            b'M' => Sex::Male,
            b'F' => Sex::Female,
            _ => Sex::Unknown,
        };
        // Each event keeps a place position 0xc into its fields, and a date in front.
        for (o, ev) in [(0x40, &mut p.birth), (0x58, &mut p.chr), (0x70, &mut p.death), (0x88, &mut p.burial)] {
            ev.place = r.text(u32_at(rec, o + 0xc).unwrap_or(0), 0xd);
            ev.date = date_from(&r, &rec[o..o + 0xc]);
        }
        // Baptism and endowment have the same shape, with a temple in the place position.
        for (o, ord) in [(0xa0, &mut p.baptism), (0xb5, &mut p.endowment)] {
            ord.date = date_from(&r, &rec[o..o + 0xc]);
            ord.temple = r.text(u32_at(rec, o + 0xc).unwrap_or(0), 0xd);
        }
        p.notes = r.notes(u32_at(rec, 0xc).unwrap_or(0));
        p.custom_id = r.text(u32_at(rec, 0x28).unwrap_or(0), 0xd);
        p.contact = r.contact(u32_at(rec, 0x24).unwrap_or(0));
        db.people.insert(rin, p);
    }
    for mrin in 1..=marriages {
        let Some(rec) = r.record(TYPE_MARRIAGE, MARRIAGE_SIZE, 0x4d, mrin) else {
            unreadable += 1;
            continue;
        };
        let who = |o: usize| u32_at(rec, o).filter(|x| *x > 0 && db.people.contains_key(x));
        let mut m = Marriage { mrin, husband: who(4), wife: who(8), ..Default::default() };
        // The marriage's fields: 'Y' here means divorced.
        m.divorced = rec[0x61] == b'Y';
        m.event.date = date_from(&r, &rec[0x34..0x40]);
        m.event.place = r.text(u32_at(rec, 0x40).unwrap_or(0), 0xd);
        m.sealing.date = date_from(&r, &rec[0x4c..0x58]);
        m.sealing.temple = r.text(u32_at(rec, 0x58).unwrap_or(0), 0xd);
        m.notes = r.notes(u32_at(rec, 0x1c).unwrap_or(0));
        // Children hang off a ring of links: +4 is the next child and +0xc the child's RIN.
        let first = u32_at(rec, 0xc).unwrap_or(0);
        let mut at = first;
        let mut hops = 0;
        while at > 0 && at <= links && hops < 4096 {
            let Some(l) = r.record(TYPE_LINK, 0x2e, 0xb2, at) else { break };
            let child = u32_at(l, 0xc).unwrap_or(0);
            if db.people.contains_key(&child) && !m.children.contains(&child) {
                m.children.push(child);
            }
            at = u32_at(l, 4).unwrap_or(0);
            if at == first {
                break;
            }
            hops += 1;
        }
        db.marriages.insert(mrin, m);
    }
    // Build the links back from people to marriages. A person whose record gives no sex takes
    // it from their place in the marriage.
    let list: Vec<Marriage> = db.marriages.values().cloned().collect();
    for m in &list {
        for (who, sex) in [(m.husband, Sex::Male), (m.wife, Sex::Female)] {
            if let Some(p) = who.and_then(|w| db.people.get_mut(&w)) {
                p.spouses.push(m.mrin);
                if p.sex == Sex::Unknown {
                    p.sex = sex;
                }
            }
        }
        for c in &m.children {
            if let Some(p) = db.people.get_mut(c) {
                p.parents.push(ParentLink { mrin: m.mrin, relation: "Biological".into(), sealed: Ordinance::default() });
            }
        }
    }
    if db.people.is_empty() && people > 0 {
        return Err("No individuals could be read from this .paf file, so it was not imported.".into());
    }
    // If most people have no readable name, the format guess is wrong for this file.
    if !db.people.is_empty() && unnamed * 2 > db.people.len() {
        return Err(format!("Only {} of {} individuals had readable names, so this file was not imported. Your .paf file was not changed.", db.people.len() - unnamed, db.people.len()));
    }
    db.home = db.people.keys().next().copied();
    let mut summary = format!(
        "Read {} individuals and {} marriages from the .paf file. Your .paf file was not changed.\n\nPlease check the results against what you know. This reader was built from the original program's code. Sources, citations, custom events and photos are not read yet.",
        db.people.len(),
        db.marriages.len()
    );
    if unreadable > 0 {
        summary.push_str(&format!("\n\n{unreadable} records could not be read."));
    }
    Ok(PafImport { db, summary })
}

/// Status words for an ordinance or date field, by their number in the file.
const STATUS: [&str; 17] = [
    "BIC", "Cancelled", "Child", "Cleared", "Completed", "Dead", "Deceased", "Done", "DNS", "DNS/CAN", "Infant", "Not Married", "Pre-1970",
    "Stillborn", "Submitted", "Uncleared", "Qualified",
];

/// Date qualifiers, by their number in the file.
const QUALIFIER: [&str; 10] = ["Abt", "Aft", "Bef", "Est", "Cal", "Sub", "In Process", "BC", "From", "To"];
/// The date word for a range: its second date prints, and the word itself doesn't.
const QUALIFIER_FROM: u8 = 9;
/// The date word for a year before the Christian era.
const QUALIFIER_BC: u8 = 8;

/// One calendar date from 3 bytes of Julian day number and a flag byte: 0x20 means the day is
/// unknown, 0x40 the month and 0x80 the year. The day count ends in the
/// standard formula's minus 32045. 0x10 marks a date the original prints inside angle
/// brackets.
fn one_date(jd: &[u8], flags: u8, bc: bool) -> Option<String> {
    let day_number = u32::from_le_bytes([jd[0], jd[1], jd[2], 0]) as i64;
    if day_number == 0 || flags & 0x80 != 0 {
        return None;
    }
    // Julian day 2440588 is 1 Jan 1970, day zero of our calendar counting.
    let (y, m, d) = crate::calc::from_serial(day_number - 2440588);
    let y = if bc && y < 1 { 1 - y } else { y };
    if !(1..=2200).contains(&y) || !(1..=12).contains(&m) {
        return None;
    }
    let month = crate::calc::MONTHS[(m - 1) as usize];
    let s = if flags & 0x40 != 0 {
        y.to_string()
    } else if flags & 0x20 != 0 {
        format!("{month} {y}")
    } else {
        format!("{d} {month} {y}")
    };
    Some(if flags & 0x10 != 0 { format!("<{s}>") } else { s })
}

/// A date field in a .paf file. The low 4 bits of the first byte give
/// the kind: 1 is a status word and 2 to 4 hold dates. The next 2 bits say how: 1 is a day number
/// and 2 a position of date text, which is how most dates like "Abt 1850" are kept.
fn date_from(r: &Reader, b: &[u8]) -> String {
    let kind = b[0] & 0x0f;
    if kind == 1 {
        return (b[1] as usize).checked_sub(1).and_then(|i| STATUS.get(i)).map(|s| s.to_string()).unwrap_or_default();
    }
    if !(2..=4).contains(&kind) {
        return String::new();
    }
    match (b[0] >> 4) & 3 {
        2 => r.text(u32_at(b, 2).unwrap_or(0), 0xd),
        1 => {
            let Some(first) = one_date(&b[4..7], b[3], b[1] == QUALIFIER_BC) else { return String::new() };
            if b[1] == QUALIFIER_FROM && b[0] >> 6 == 1 && (kind == 3 || kind == 4) {
                if let Some(second) = one_date(&b[9..12], b[8], false) {
                    return format!("{first}{}{second}", crate::calc::RANGE_SEPARATOR);
                }
            }
            match (b[1] as usize).checked_sub(1).and_then(|i| QUALIFIER.get(i)) {
                Some(w) => format!("{w} {first}"),
                None => first,
            }
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A check against a real family file, run by hand with its path in FK_PAF. It prints only
    /// counts, so nothing about the family ends up in a log or in this project.
    #[test]
    #[ignore]
    fn real_file_report() {
        let path = std::env::var("FK_PAF").expect("set FK_PAF to a .paf file");
        let d = std::fs::read(path).unwrap();
        println!("kind {:?}, bytes {}", identify(&d), d.len());
        println!("header people {:?} marriages {:?} links {:?}", u32_at(&d, 0x16), u32_at(&d, 0x82), u32_at(&d, 0xe6));
        let got = import(&d).unwrap();
        let db = got.db;
        let mut copy = db.clone();
        let problems = copy.check(false);
        let (mut named, mut born, mut born_place, mut died, mut slash, mut lower) = (0, 0, 0, 0, 0, 0);
        for p in db.people.values() {
            named += !p.name.is_empty() as usize;
            born += !p.birth.date.is_empty() as usize;
            born_place += !p.birth.place.is_empty() as usize;
            died += !p.death.date.is_empty() as usize;
            slash += p.name.contains('/') as usize;
            lower += p.birth.place.chars().any(|c| c.is_lowercase()) as usize;
        }
        println!("read people {} marriages {} named {named} with surname marks {slash}", db.people.len(), db.marriages.len());
        println!("birth dates {born} birth places {born_place} (readable text {lower}) death dates {died}");
        println!("links that disagree {}", problems.len());
        // Sense check: children should be born after their parents, by a believable gap.
        let year = |p: &Person| crate::calc::parse_date(&p.birth.date, false).map(|x| x.year);
        let (mut pairs, mut good) = (0, 0);
        for p in db.people.values() {
            let Some(y) = year(p) else { continue };
            for parent in [db.father(p.rin), db.mother(p.rin)].into_iter().flatten() {
                let Some(py) = db.person(parent).and_then(year) else { continue };
                pairs += 1;
                good += (10..=70).contains(&(y - py)) as usize;
            }
        }
        println!("parent and child pairs with both birth years {pairs}, believable gap {good}");
        let years: Vec<i32> = db.people.values().filter_map(year).collect();
        println!("birth years from {:?} to {:?}", years.iter().min(), years.iter().max());
        let (mut m, mut c) = (0, 0);
        for x in db.marriages.values() {
            m += !x.event.date.is_empty() as usize;
            c += x.children.len();
        }
        let (mut sexes, mut notes, mut divorced) = ([0usize; 3], 0, 0);
        for p in db.people.values() {
            sexes[match p.sex { Sex::Male => 0, Sex::Female => 1, Sex::Unknown => 2 }] += 1;
            notes += !p.notes.is_empty() as usize;
        }
        for x in db.marriages.values() {
            notes += !x.notes.is_empty() as usize;
            divorced += x.divorced as usize;
        }
        println!("marriage dates {m}, children links {c}, male/female/unknown {sexes:?}, with notes {notes}, divorced {divorced}");
        let wrong = db.marriages.values().filter(|x| {
            x.husband.and_then(|r| db.person(r)).is_some_and(|p| p.sex == Sex::Female) || x.wife.and_then(|r| db.person(r)).is_some_and(|p| p.sex == Sex::Male)
        });
        println!("marriages where a spouse's sex disagrees with their role {}", wrong.count());
        let standard = db.people.values().filter(|p| !p.birth.date.is_empty() && crate::calc::parse_date(&p.birth.date, false).is_some()).count();
        println!("birth dates that parse as standard dates {standard}");
    }

    #[test]
    fn the_date_field_decoder_follows_the_original() {
        let mut d = vec![0u8; 0x3000 + BLOCK * 8];
        d[0x12] = 1;
        d[0x3000] = 0x0b;
        d[0x5000 + 0xd..0x5000 + 0xd + 8].copy_from_slice(b"Abt 1850");
        let r = Reader::new(&d).unwrap();
        // Text form: first byte kind 2, way 2, then the text's position.
        let mut b = [0u8; 12];
        b[0] = 0x22;
        b[2..6].copy_from_slice(&0x5000u32.to_le_bytes());
        assert_eq!(date_from(&r, &b), "Abt 1850");
        // Status form: kind 1 with the word's number.
        let mut b = [0u8; 12];
        b[0] = 0x01;
        b[1] = 15;
        assert_eq!(date_from(&r, &b), "Submitted");
        // Day number with a qualifier, month unknown.
        let mut b = [0u8; 12];
        b[0] = 0x12;
        b[1] = 1;
        b[3] = 0x40;
        b[4..7].copy_from_slice(&2415021u32.to_le_bytes()[..3]);
        assert_eq!(date_from(&r, &b), "Abt 1900");
    }

    /// Builds a tiny file in the layout worked out from the code. This only proves the reader
    /// matches that reading of the code. It says nothing about real files.
    fn fake_file() -> Vec<u8> {
        let mut d = vec![0u8; 0x3000 + BLOCK * 8 + 0x4000];
        d[0..3].copy_from_slice(b"500");
        d[4..7].copy_from_slice(b"500");
        d[8..11].copy_from_slice(b"PAF");
        d[12] = 4;
        d[0x12] = 1;
        d[0x16] = 3; // people
        d[0x82] = 1; // marriages
        d[0xe6] = 2; // links
        d[0x3000] = 0x0b;
        d[0x3001] = TYPE_PERSON;
        d[0x3002] = TYPE_MARRIAGE;
        d[0x3003] = TYPE_LINK;
        d[MASTER_MAP + 2 * (TYPE_PERSON as usize - 1)] = 1;
        d[MASTER_MAP + 2 * (TYPE_MARRIAGE as usize - 1)] = 1;
        d[MASTER_MAP + 2 * (TYPE_LINK as usize - 1)] = 1;
        let block = |n: usize| 0x3000 + n * BLOCK;
        let pool = 0x3000 + BLOCK * 8;
        let put = |d: &mut Vec<u8>, at: usize, s: &str, skip: usize| {
            d[at + skip..at + skip + s.len()].copy_from_slice(s.as_bytes());
        };
        for (i, n) in ["Ann /Lee/", "Joe /Lee/", "Kid /Lee/"].iter().enumerate() {
            let at = pool + i * 0x100;
            put(&mut d, at, n, 0xf);
            let rec = block(1) + i * 0xdd;
            d[rec..rec + 4].copy_from_slice(&(at as u32).to_le_bytes());
        }
        // Birth place for the first person.
        let place = pool + 0x800;
        put(&mut d, place, "Boise, Idaho", 0xd);
        let first = block(1);
        d[first + 0x40 + 0xc..first + 0x40 + 0x10].copy_from_slice(&(place as u32).to_le_bytes());
        // Birth date for the first person: 1 Jan 1900 is Julian day 2415021.
        d[first + 0x40] = 0x12;
        d[first + 0x40 + 4..first + 0x40 + 7].copy_from_slice(&2415021u32.to_le_bytes()[..3]);
        let m = block(2);
        d[m + 4] = 2; // husband
        d[m + 8] = 1; // wife
        d[m + 0xc] = 1; // first child link
        let l = block(3);
        d[l + 4] = 1; // ring back to itself
        d[l + 0xc] = 3; // child RIN
        d
    }

    #[test]
    fn reads_the_layout_from_the_code() {
        let got = import(&fake_file()).unwrap();
        let db = got.db;
        assert_eq!(db.people.len(), 3);
        assert_eq!(db.person(1).unwrap().name, "Ann /Lee/");
        assert_eq!(db.person(1).unwrap().birth.place, "Boise, Idaho");
        assert_eq!(db.person(1).unwrap().birth.date, "1 Jan 1900");
        assert_eq!(db.person(1).unwrap().sex, Sex::Female);
        assert_eq!(db.father(3), Some(2));
        assert_eq!(db.mother(3), Some(1));
    }

    #[test]
    fn refuses_what_it_cannot_read() {
        assert!(import(b"not a family file at all").is_err());
        let mut old = vec![0u8; 100];
        old[0..3].copy_from_slice(b"PAF");
        old[4] = b'4';
        old[8] = b'4';
        assert_eq!(identify(&old), Kind::Paf4);
        assert!(import(&old).unwrap_err().contains("version 3 or 4"));
        // A damaged map must fail cleanly and not panic.
        let mut bad = fake_file();
        bad[0x3000] = 0;
        assert!(import(&bad).is_err());
        let mut short = fake_file();
        short.truncate(0x3100);
        assert!(import(&short).is_err());
    }

    #[test]
    fn nonsense_never_panics() {
        let mut d = fake_file();
        for i in (0..d.len()).step_by(7) {
            d[i] = (i * 31 % 251) as u8;
        }
        d[0..3].copy_from_slice(b"500");
        d[8..11].copy_from_slice(b"PAF");
        d[12] = 4;
        d[13] = 0;
        let _ = import(&d);
    }
}
