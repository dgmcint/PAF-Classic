//! Renumbering record numbers in a chosen order.

use super::*;
use std::collections::BTreeMap;

/// The orders Renumber Records can put people in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenumberOrder {
    /// The home person first, then their parents, grandparents and so on, then everyone
    /// else linked to them, family by family, then anyone not linked.
    Family,
    /// The order the records are in now, closing the gaps.
    Present,
    /// By surname, then given names. People without a name come last.
    Alphabetical,
}

impl Db {
    /// Everyone's record number in the order they would get the new numbers 1, 2, 3...
    pub fn renumber_order(&self, how: RenumberOrder) -> Vec<u32> {
        match how {
            RenumberOrder::Present => self.people.keys().copied().collect(),
            RenumberOrder::Alphabetical => {
                let mut v: Vec<u32> = self.people.keys().copied().collect();
                v.sort_by_cached_key(|r| {
                    let n = &self.people[r].name;
                    (plain_name(n).is_empty(), sort_key(n), *r)
                });
                v
            }
            RenumberOrder::Family => {
                use std::collections::{HashSet, VecDeque};
                let mut out = Vec::with_capacity(self.people.len());
                let mut seen = HashSet::new();
                let parents_of = |r: u32| -> Vec<u32> {
                    self.people[&r]
                        .parents
                        .iter()
                        .filter_map(|l| self.marriages.get(&l.mrin))
                        .flat_map(|m| [m.husband, m.wife])
                        .flatten()
                        .filter(|x| self.people.contains_key(x))
                        .collect()
                };
                let home = self.home.filter(|h| self.people.contains_key(h)).or_else(|| self.people.keys().next().copied());
                if let Some(h) = home {
                    // Ancestors generation by generation, as on a pedigree chart.
                    let mut generation = vec![h];
                    seen.insert(h);
                    out.push(h);
                    while !generation.is_empty() {
                        let mut next = Vec::new();
                        for r in generation {
                            for p in parents_of(r) {
                                if seen.insert(p) {
                                    out.push(p);
                                    next.push(p);
                                }
                            }
                        }
                        generation = next;
                    }
                }
                // Then everyone reachable, family by family: spouses, children, parents.
                let mut queue: VecDeque<u32> = out.iter().copied().collect();
                let walk = |queue: &mut VecDeque<u32>, seen: &mut HashSet<u32>, out: &mut Vec<u32>| {
                    while let Some(r) = queue.pop_front() {
                        let p = &self.people[&r];
                        let mut near = Vec::new();
                        for m in p.spouses.iter().filter_map(|m| self.marriages.get(m)) {
                            near.extend([m.husband, m.wife].into_iter().flatten());
                            near.extend(m.children.iter().copied());
                        }
                        near.extend(parents_of(r));
                        for x in near {
                            if self.people.contains_key(&x) && seen.insert(x) {
                                out.push(x);
                                queue.push_back(x);
                            }
                        }
                    }
                };
                walk(&mut queue, &mut seen, &mut out);
                for r in self.people.keys().copied().collect::<Vec<_>>() {
                    if seen.insert(r) {
                        out.push(r);
                        queue.push_back(r);
                        walk(&mut queue, &mut seen, &mut out);
                    }
                }
                out
            }
        }
    }

    /// Gives people the numbers 1, 2, 3... in `order`, and marriages 1, 2, 3... by their first
    /// member in that order. Every link follows. Each person keeps their old number as Old ID
    /// (unless they already have one). Returns old to new record numbers for people and marriages.
    pub fn renumber(&mut self, order: &[u32]) -> (BTreeMap<u32, u32>, BTreeMap<u32, u32>) {
        let mut rins = BTreeMap::new();
        for r in order.iter().chain(self.people.keys()) {
            if self.people.contains_key(r) && !rins.contains_key(r) {
                let n = rins.len() as u32 + 1;
                rins.insert(*r, n);
            }
        }
        let mut by_new: Vec<(u32, u32)> = rins.iter().map(|(o, n)| (*n, *o)).collect();
        by_new.sort();
        let mut mrins = BTreeMap::new();
        for (_, old) in &by_new {
            for m in &self.people[old].spouses {
                if self.marriages.contains_key(m) && !mrins.contains_key(m) {
                    let n = mrins.len() as u32 + 1;
                    mrins.insert(*m, n);
                }
            }
            for l in &self.people[old].parents {
                if self.marriages.contains_key(&l.mrin) && !mrins.contains_key(&l.mrin) {
                    let n = mrins.len() as u32 + 1;
                    mrins.insert(l.mrin, n);
                }
            }
        }
        for m in self.marriages.keys() {
            if !mrins.contains_key(m) {
                let n = mrins.len() as u32 + 1;
                mrins.insert(*m, n);
            }
        }
        let rp = |r: u32| rins.get(&r).copied().unwrap_or(r);
        let mp = |m: u32| mrins.get(&m).copied().unwrap_or(m);
        let people = std::mem::take(&mut self.people);
        for (_, mut p) in people {
            if p.old_id.is_empty() {
                p.old_id = p.rin.to_string();
            }
            p.rin = rp(p.rin);
            for m in p.spouses.iter_mut() {
                *m = mp(*m);
            }
            for l in p.parents.iter_mut() {
                l.mrin = mp(l.mrin);
            }
            self.people.insert(p.rin, p);
        }
        let marriages = std::mem::take(&mut self.marriages);
        for (_, mut m) in marriages {
            m.mrin = mp(m.mrin);
            m.husband = m.husband.map(rp);
            m.wife = m.wife.map(rp);
            for c in m.children.iter_mut() {
                *c = rp(*c);
            }
            self.marriages.insert(m.mrin, m);
        }
        self.home = self.home.map(rp);
        self.root_person = self.root_person.map(rp);
        for v in self.focus_lists.values_mut() {
            for r in v.iter_mut() {
                *r = rp(*r);
            }
        }
        (rins, mrins)
    }
}

#[cfg(test)]
mod renumber_tests {
    use super::*;

    /// Who is linked to whom, by name, so it can be compared before and after renumbering.
    fn shape(db: &Db) -> Vec<String> {
        let n = |r: u32| db.people[&r].name.clone();
        let mut v: Vec<String> = db
            .marriages
            .values()
            .map(|m| {
                let mut kids: Vec<String> = m.children.iter().map(|c| n(*c)).collect();
                kids.sort();
                format!("{:?}+{:?}:{:?}", m.husband.map(n), m.wife.map(n), kids)
            })
            .collect();
        for p in db.people.values() {
            let fams: Vec<String> = p.spouses.iter().chain(p.parents.iter().map(|l| &l.mrin)).map(|m| db.marriages[m].mrin.to_string()).collect();
            assert_eq!(fams.len(), p.spouses.len() + p.parents.len());
        }
        v.sort();
        v.push(format!("home={:?}", db.home.map(n)));
        v
    }

    #[test]
    fn renumbering_keeps_every_link() {
        let mut db = sample_family();
        // Spread the numbers out, as after an import from another program.
        let order: Vec<u32> = db.people.keys().rev().copied().collect();
        db.renumber(&order);
        let before = shape(&db);
        let home_name = db.people[&db.home.unwrap()].name.clone();
        db.focus_lists.insert("x".into(), vec![db.home.unwrap()]);
        for how in [RenumberOrder::Family, RenumberOrder::Alphabetical, RenumberOrder::Present] {
            let order = db.renumber_order(how);
            assert_eq!(order.len(), db.people.len());
            db.renumber(&order);
            assert_eq!(shape(&db), before, "{how:?}");
            assert_eq!(db.people.keys().copied().collect::<Vec<_>>(), (1..=db.people.len() as u32).collect::<Vec<_>>());
            assert_eq!(db.marriages.keys().copied().collect::<Vec<_>>(), (1..=db.marriages.len() as u32).collect::<Vec<_>>());
            assert!(db.check(false).is_empty(), "{:?}", db.check(false));
            assert_eq!(db.focus_lists["x"], vec![db.people.values().find(|p| p.name == home_name).unwrap().rin]);
        }
        let order = db.renumber_order(RenumberOrder::Family);
        db.renumber(&order);
        assert_eq!(db.home, Some(1));
        // Old IDs are the numbers before the first renumbering and are not overwritten.
        assert!(db.people.values().all(|p| !p.old_id.is_empty()));
    }

    #[test]
    fn family_order_starts_with_home_then_parents() {
        let db = sample_family();
        let order = db.renumber_order(RenumberOrder::Family);
        let h = db.home.unwrap();
        assert_eq!(order[0], h);
        let mut parents = vec![db.father(h).unwrap(), db.mother(h).unwrap()];
        parents.sort();
        let mut next = order[1..3].to_vec();
        next.sort();
        assert_eq!(next, parents);
    }
}
