//! Merging two people, and bringing another file's records into this one.

use super::*;

impl Db {
    /// Keeps the first person and fills their empty fields from the second, then removes the second.
    pub fn merge_people(&mut self, keep: u32, other: u32) {
        if keep == other {
            return;
        }
        let Some(b) = self.people.get(&other).cloned() else { return };
        // Repoint marriages and children.
        for m in b.spouses.clone() {
            if let Some(x) = self.marriages.get_mut(&m) {
                if x.husband == Some(other) {
                    x.husband = Some(keep);
                }
                if x.wife == Some(other) {
                    x.wife = Some(keep);
                }
            }
        }
        for l in b.parents.clone() {
            if let Some(x) = self.marriages.get_mut(&l.mrin) {
                for c in x.children.iter_mut() {
                    if *c == other {
                        *c = keep;
                    }
                }
                let mut seen = Vec::new();
                x.children.retain(|c| {
                    if seen.contains(c) {
                        false
                    } else {
                        seen.push(*c);
                        true
                    }
                });
            }
        }
        if let Some(a) = self.people.get_mut(&keep) {
            fn fill(a: &mut String, b: &str) {
                if a.trim().is_empty() {
                    *a = b.to_string();
                }
            }
            fn fill_ev(a: &mut Event, b: &Event) {
                fill(&mut a.date, &b.date);
                fill(&mut a.place, &b.place);
                a.citations.extend(b.citations.iter().cloned());
            }
            fill(&mut a.name, &b.name);
            fill(&mut a.title, &b.title);
            fill(&mut a.nickname, &b.nickname);
            fill(&mut a.aka, &b.aka);
            fill(&mut a.afn, &b.afn);
            if a.sex == Sex::Unknown {
                a.sex = b.sex;
            }
            fill_ev(&mut a.birth, &b.birth);
            fill_ev(&mut a.chr, &b.chr);
            fill_ev(&mut a.death, &b.death);
            fill_ev(&mut a.burial, &b.burial);
            fill(&mut a.baptism.date, &b.baptism.date);
            fill(&mut a.baptism.temple, &b.baptism.temple);
            fill(&mut a.endowment.date, &b.endowment.date);
            fill(&mut a.endowment.temple, &b.endowment.temple);
            if !b.notes.trim().is_empty() {
                if a.notes.trim().is_empty() {
                    a.notes = b.notes.clone();
                } else if !a.notes.contains(b.notes.trim()) {
                    a.notes = format!("{}\n{}", a.notes, b.notes);
                }
            }
            if a.contact.is_empty() {
                a.contact = b.contact.clone();
            }
            a.media.extend(b.media.iter().cloned());
            a.custom.extend(b.custom.iter().cloned());
            a.citations.extend(b.citations.iter().cloned());
            for m in b.spouses.iter() {
                if !a.spouses.contains(m) {
                    a.spouses.push(*m);
                }
            }
            for l in b.parents.iter() {
                if !a.parents.iter().any(|x| x.mrin == l.mrin) {
                    a.parents.push(l.clone());
                }
            }
        }
        self.people.remove(&other);
        if self.home == Some(other) {
            self.home = Some(keep);
        }
        if self.root_person == Some(other) {
            self.root_person = Some(keep);
        }
    }

    /// Adds everything from another family file, giving each record the next free number.
    /// Returns how many individuals came in.
    pub fn absorb(&mut self, other: Db) -> usize {
        use std::collections::HashMap;
        let (mut rins, mut mrins, mut srcs, mut reps): (HashMap<u32, u32>, HashMap<u32, u32>, HashMap<u32, u32>, HashMap<u32, u32>) = Default::default();
        let (mut nr, mut nm, mut ns, mut nd) = (self.next_rin(), self.next_mrin(), self.next_source(), self.next_repository());
        for k in other.people.keys() {
            rins.insert(*k, nr);
            nr += 1;
        }
        for k in other.marriages.keys() {
            mrins.insert(*k, nm);
            nm += 1;
        }
        for k in other.sources.keys() {
            srcs.insert(*k, ns);
            ns += 1;
        }
        for k in other.repositories.keys() {
            reps.insert(*k, nd);
            nd += 1;
        }
        let cite = |list: &mut Vec<Citation>, srcs: &HashMap<u32, u32>| {
            for c in list.iter_mut() {
                c.source = srcs.get(&c.source).copied().unwrap_or(c.source);
            }
        };
        let count = other.people.len();
        for (_, mut p) in other.people {
            p.rin = rins[&p.rin];
            p.spouses = p.spouses.iter().filter_map(|m| mrins.get(m).copied()).collect();
            for l in p.parents.iter_mut() {
                l.mrin = mrins.get(&l.mrin).copied().unwrap_or(l.mrin);
            }
            for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
                cite(&mut e.citations, &srcs);
            }
            cite(&mut p.citations, &srcs);
            self.people.insert(p.rin, p);
        }
        for (_, mut m) in other.marriages {
            m.mrin = mrins[&m.mrin];
            m.husband = m.husband.and_then(|r| rins.get(&r).copied());
            m.wife = m.wife.and_then(|r| rins.get(&r).copied());
            m.children = m.children.iter().filter_map(|r| rins.get(r).copied()).collect();
            cite(&mut m.event.citations, &srcs);
            cite(&mut m.citations, &srcs);
            self.marriages.insert(m.mrin, m);
        }
        for (_, mut s) in other.sources {
            s.id = srcs[&s.id];
            s.repository = s.repository.and_then(|r| reps.get(&r).copied());
            self.sources.insert(s.id, s);
        }
        for (_, mut r) in other.repositories {
            r.id = reps[&r.id];
            self.repositories.insert(r.id, r);
        }
        if self.home.is_none() {
            self.home = other.home.and_then(|r| rins.get(&r).copied());
        }
        if self.home.is_none() {
            self.home = self.people.keys().next().copied();
        }
        count
    }
}
