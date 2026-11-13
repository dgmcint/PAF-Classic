//! Ancestors, descendants and the relationship between two people.

use crate::data::*;
use std::collections::{BTreeMap, VecDeque};

fn ancestors_with_depth(db: &Db, start: u32) -> BTreeMap<u32, u32> {
    let mut seen = BTreeMap::new();
    let mut q = VecDeque::new();
    q.push_back((start, 0u32));
    while let Some((rin, depth)) = q.pop_front() {
        if seen.contains_key(&rin) {
            continue;
        }
        seen.insert(rin, depth);
        if let Some(p) = db.person(rin) {
            for l in &p.parents {
                if let Some(m) = db.marriages.get(&l.mrin) {
                    for par in [m.husband, m.wife].into_iter().flatten() {
                        q.push_back((par, depth + 1));
                    }
                }
            }
        }
    }
    seen
}

pub fn ordinal(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

fn gendered(sex: Sex, f: &str, m: &str) -> String {
    if sex == Sex::Female { f.to_string() } else { m.to_string() }
}

fn great(n: u32) -> String {
    if n >= 2 { format!("{} Great ", ordinal(n)) } else { "Great ".to_string() }
}

/// The word for what A is to B, with A `up_a` steps and B `up_b` steps below a common ancestor.
/// Turns generation counts into relationship words.
pub fn relationship_word(sex_a: Sex, up_a: u32, up_b: u32) -> String {
    let (f, m) = (sex_a == Sex::Female, sex_a);
    let _ = f;
    if up_a == 0 {
        return match up_b {
            0 => "Same person".into(),
            1 => gendered(m, "Mother", "Father"),
            2 => gendered(m, "Grandmother", "Grandfather"),
            n => format!("{}{}", great(n - 2), gendered(m, "Grandmother", "Grandfather")),
        };
    }
    if up_b == 0 {
        return match up_a {
            1 => gendered(m, "Daughter", "Son"),
            2 => gendered(m, "Granddaughter", "Grandson"),
            n => format!("{}{}", great(n - 2), gendered(m, "Granddaughter", "Grandson")),
        };
    }
    if up_a == 1 && up_b == 1 {
        return gendered(m, "Sister", "Brother");
    }
    if up_a == 1 {
        return match up_b {
            2 => gendered(m, "Aunt", "Uncle"),
            3 => gendered(m, "Grand Aunt", "Grand Uncle"),
            n => format!("{}{}", great(n - 3), gendered(m, "Grand Aunt", "Grand Uncle")),
        };
    }
    if up_b == 1 {
        return match up_a {
            2 => gendered(m, "Niece", "Nephew"),
            3 => gendered(m, "Grand Niece", "Grand Nephew"),
            n => format!("{}{}", great(n - 3), gendered(m, "Grand Niece", "Grand Nephew")),
        };
    }
    let degree = up_a.min(up_b) - 1;
    let removed = up_a.abs_diff(up_b);
    let mut s = format!("{} Cousin", ordinal(degree));
    if removed == 1 {
        s.push_str(" 1 Time Removed");
    } else if removed > 1 {
        s.push_str(&format!(" {removed} Times Removed"));
    }
    s
}

pub struct Relationship {
    pub words: String,
    pub ancestors: Vec<u32>,
}

/// Searches back generation by generation for ancestors the two share. Step and in-law
/// relationships are not shown, as the guide says; a spouse is named directly.
pub fn relationships(db: &Db, a: u32, b: u32) -> Vec<Relationship> {
    let mut out = Vec::new();
    let sex_a = db.person(a).map(|p| p.sex).unwrap_or_default();
    if a == b {
        out.push(Relationship { words: "Same person".into(), ancestors: vec![] });
        return out;
    }
    let is_spouse = db.person(a).map(|p| p.spouses.iter().any(|m| db.spouse_in(*m, a) == Some(b))).unwrap_or(false);
    if is_spouse {
        let word = if sex_a == Sex::Female { "Wife" } else { "Husband" };
        out.push(Relationship { words: format!("{word} of {}", db.name(b)), ancestors: vec![] });
    }
    let aa = ancestors_with_depth(db, a);
    let ab = ancestors_with_depth(db, b);
    let mut common: Vec<(u32, u32, u32)> = aa.iter().filter_map(|(r, da)| ab.get(r).map(|db_| (*r, *da, *db_))).collect();
    common.sort_by_key(|c| c.1 + c.2);
    // Only nearest shared ancestors: skip anyone who is an ancestor of another shared ancestor.
    let mut groups: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (r, da, db_) in &common {
        let shadowed = common.iter().any(|(o, oa, ob)| {
            o != r && oa + ob < da + db_ && ancestors_with_depth(db, *o).contains_key(r)
        });
        if !shadowed {
            groups.entry((*da, *db_)).or_default().push(*r);
        }
    }
    let mut seen = Vec::new();
    for ((da, db_), anc) in groups {
        let words = relationship_word(sex_a, da, db_);
        if !seen.contains(&words) {
            seen.push(words.clone());
            out.push(Relationship { words, ancestors: anc });
        }
    }
    out
}

pub fn relation_to_root(db: &Db, rin: u32) -> Option<String> {
    let root = db.root_person?;
    if root == rin {
        return Some("Root Person".into());
    }
    relationships(db, rin, root).first().map(|r| r.words.clone())
}

/// Everyone descended from the person, with generation numbers.
pub fn descendants(db: &Db, rin: u32) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let mut seen = vec![rin];
    fn walk(db: &Db, rin: u32, depth: u32, out: &mut Vec<(u32, u32)>, seen: &mut Vec<u32>) {
        for c in db.children_of(rin) {
            if seen.contains(&c) {
                continue;
            }
            seen.push(c);
            out.push((c, depth + 1));
            walk(db, c, depth + 1, out, seen);
        }
    }
    walk(db, rin, 0, &mut out, &mut seen);
    out
}

/// Everyone in the person's ancestry, with generation numbers.
pub fn ancestors(db: &Db, rin: u32) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = ancestors_with_depth(db, rin).into_iter().filter(|(r, _)| *r != rin).collect();
    v.sort_by_key(|x| x.1);
    v
}
