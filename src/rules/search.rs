//! Finding possible duplicates, and Global Search and Replace.

use super::*;
use crate::data::*;

/// Pairs that look like the same person, for Match/Merge, with the original's default options.
/// Names are compared as whole surname and given name, and then these
/// checks apply: same sex, not spouses or parent and child, a surname and a date on both, dates within
/// the year range, and parents whose names agree.
pub fn duplicate_candidates(db: &Db, sound_alike: bool, year_slack: i32) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let v: Vec<&Person> = db.people.values().filter(|p| !p.name.trim().is_empty()).collect();
    let df = db.prefs.day_first_numeric;
    let same_name = |x: u32, y: u32| {
        let (Some(px), Some(py)) = (db.person(x), db.person(y)) else { return true };
        soundex(&surname_of(&px.name)) == soundex(&surname_of(&py.name)) && soundex(&given_of(&px.name)) == soundex(&given_of(&py.name))
    };
    let parents_agree = |a: &Person, b: &Person| {
        if a.parents.is_empty() || b.parents.is_empty() {
            return true;
        }
        a.parents.iter().filter_map(|l| db.marriages.get(&l.mrin)).any(|ma| {
            b.parents.iter().filter_map(|l| db.marriages.get(&l.mrin)).any(|mb| {
                let fathers = match (ma.husband, mb.husband) {
                    (Some(x), Some(y)) => same_name(x, y),
                    _ => true,
                };
                let mothers = match (ma.wife, mb.wife) {
                    (Some(x), Some(y)) => same_name(x, y),
                    _ => true,
                };
                fathers && mothers
            })
        })
    };
    for (i, a) in v.iter().enumerate() {
        for b in &v[i + 1..] {
            let (sa, sb) = (surname_of(&a.name), surname_of(&b.name));
            let (ga, gb) = (given_of(&a.name).to_lowercase(), given_of(&b.name).to_lowercase());
            let name_match = if sound_alike {
                soundex(&sa) == soundex(&sb) && soundex(&ga) == soundex(&gb)
            } else {
                sa.eq_ignore_ascii_case(&sb) && ga == gb
            };
            if !name_match || sa.trim().is_empty() || sb.trim().is_empty() {
                continue;
            }
            // The original compares the sex byte itself, so Unknown only matches Unknown.
            if a.sex != b.sex {
                continue;
            }
            // People already joined as spouses or as parent and child are never the same person.
            let linked = a.spouses.iter().any(|m| db.spouse_in(*m, a.rin) == Some(b.rin))
                || db.children_of(a.rin).contains(&b.rin)
                || db.children_of(b.rin).contains(&a.rin);
            if linked || !parents_agree(a, b) {
                continue;
            }
            // A person's christening stands in when there's no birth date, and both need a date.
            let field = |p: &Person| if p.birth.date.trim().is_empty() { p.chr.date.clone() } else { p.birth.date.clone() };
            if field(a).trim().is_empty() || field(b).trim().is_empty() {
                continue;
            }
            let (ya, yb) = (parse_date(&field(a), df), parse_date(&field(b), df));
            if let (Some(x), Some(y)) = (ya, yb) {
                if (x.year - y.year).abs() > year_slack {
                    continue;
                }
            }
            out.push((a.rin, b.rin));
        }
    }
    out
}

/// Replaces matching text in one group of fields. Returns how many fields changed.
pub fn global_replace(db: &mut Db, field: &str, find: &str, with: &str, whole: bool, case_sensitive: bool) -> usize {
    if find.is_empty() {
        return 0;
    }
    let mut n = 0;
    let mut swap = |s: &mut String| {
        let hit = if whole {
            if case_sensitive { s.trim() == find } else { s.trim().eq_ignore_ascii_case(find) }
        } else if case_sensitive {
            s.contains(find)
        } else {
            s.to_lowercase().contains(&find.to_lowercase())
        };
        if !hit {
            return;
        }
        if whole {
            *s = with.to_string();
        } else if case_sensitive {
            *s = s.replace(find, with);
        } else {
            *s = replace_ignore_case(s, find, with);
        }
        n += 1;
    };
    let people = db.people.values_mut();
    match field {
        "Places" => {
            for p in people {
                for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
                    swap(&mut e.place);
                }
                for c in p.custom.iter_mut() {
                    swap(&mut c.place);
                }
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.event.place);
                for c in m.custom.iter_mut() {
                    swap(&mut c.place);
                }
            }
        }
        "Dates" => {
            for p in people {
                for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
                    swap(&mut e.date);
                }
                swap(&mut p.baptism.date);
                swap(&mut p.endowment.date);
                for c in p.custom.iter_mut() {
                    swap(&mut c.date);
                }
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.event.date);
                swap(&mut m.sealing.date);
            }
        }
        "Full Name" => {
            for p in people {
                swap(&mut p.name);
            }
        }
        "Notes" => {
            for p in people {
                swap(&mut p.notes);
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.notes);
            }
        }
        "Temple Codes" => {
            for p in people {
                swap(&mut p.baptism.temple);
                swap(&mut p.endowment.temple);
                for l in p.parents.iter_mut() {
                    swap(&mut l.sealed.temple);
                }
            }
            for m in db.marriages.values_mut() {
                swap(&mut m.sealing.temple);
            }
        }
        "Multimedia file names" => {
            for p in people {
                for x in p.media.iter_mut() {
                    swap(&mut x.file);
                }
            }
        }
        _ => {}
    }
    n
}

fn replace_ignore_case(s: &str, find: &str, with: &str) -> String {
    let (ls, lf) = (s.to_lowercase(), find.to_lowercase());
    if ls.len() != s.len() {
        return s.replace(find, with);
    }
    let mut out = String::new();
    let mut i = 0;
    while let Some(j) = ls[i..].find(&lf) {
        out.push_str(&s[i..i + j]);
        out.push_str(with);
        i += j + lf.len();
    }
    out.push_str(&s[i..]);
    out
}
