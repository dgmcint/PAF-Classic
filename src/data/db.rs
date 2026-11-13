//! The whole family file and the lookups everything else uses.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Db {
    #[serde(default)]
    pub people: BTreeMap<u32, Person>,
    #[serde(default)]
    pub marriages: BTreeMap<u32, Marriage>,
    #[serde(default)]
    pub sources: BTreeMap<u32, Source>,
    #[serde(default)]
    pub repositories: BTreeMap<u32, Repository>,
    #[serde(default)]
    pub home: Option<u32>,
    #[serde(default)]
    pub root_person: Option<u32>,
    #[serde(default)]
    pub prefs: Prefs,
    #[serde(default)]
    pub log: Vec<String>,
    #[serde(default)]
    pub focus_lists: BTreeMap<String, Vec<u32>>,
    #[serde(default)]
    pub custom_types: Vec<String>,
}

impl Db {
    pub fn next_rin(&self) -> u32 {
        self.people.keys().next_back().copied().unwrap_or(0) + 1
    }

    pub fn next_mrin(&self) -> u32 {
        self.marriages.keys().next_back().copied().unwrap_or(0) + 1
    }

    pub fn next_source(&self) -> u32 {
        self.sources.keys().next_back().copied().unwrap_or(0) + 1
    }

    pub fn next_repository(&self) -> u32 {
        self.repositories.keys().next_back().copied().unwrap_or(0) + 1
    }

    pub fn person(&self, rin: u32) -> Option<&Person> {
        self.people.get(&rin)
    }

    pub fn name(&self, rin: u32) -> String {
        self.person(rin).map(|p| plain_name(&p.name)).unwrap_or_default()
    }

    pub fn add_person(&mut self, mut p: Person) -> u32 {
        let rin = if p.rin != 0 && !self.people.contains_key(&p.rin) { p.rin } else { self.next_rin() };
        p.rin = rin;
        self.people.insert(rin, p);
        if self.home.is_none() {
            self.home = Some(rin);
        }
        rin
    }

    pub fn spouse_in(&self, mrin: u32, rin: u32) -> Option<u32> {
        let m = self.marriages.get(&mrin)?;
        if m.husband == Some(rin) {
            m.wife
        } else if m.wife == Some(rin) {
            m.husband
        } else {
            None
        }
    }

    /// The primary parents: the first parent link.
    pub fn primary_parents(&self, rin: u32) -> Option<&Marriage> {
        let p = self.person(rin)?;
        p.parents.first().and_then(|l| self.marriages.get(&l.mrin))
    }

    pub fn father(&self, rin: u32) -> Option<u32> {
        self.primary_parents(rin).and_then(|m| m.husband)
    }

    pub fn mother(&self, rin: u32) -> Option<u32> {
        self.primary_parents(rin).and_then(|m| m.wife)
    }

    pub fn children_of(&self, rin: u32) -> Vec<u32> {
        let mut out = Vec::new();
        if let Some(p) = self.person(rin) {
            for m in &p.spouses {
                if let Some(mar) = self.marriages.get(m) {
                    out.extend(mar.children.iter().copied());
                }
            }
        }
        out
    }

    pub fn has_descendants(&self, rin: u32) -> bool {
        !self.children_of(rin).is_empty()
    }

    pub fn is_linked(&self, rin: u32) -> bool {
        self.person(rin).map(|p| !p.spouses.is_empty() || !p.parents.is_empty()).unwrap_or(false)
    }

    pub fn all_places(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for p in self.people.values() {
            for e in [&p.birth, &p.chr, &p.death, &p.burial] {
                v.push(e.place.clone());
            }
            v.extend(p.custom.iter().map(|c| c.place.clone()));
        }
        for m in self.marriages.values() {
            v.push(m.event.place.clone());
            v.extend(m.custom.iter().map(|c| c.place.clone()));
        }
        v.retain(|s| !s.trim().is_empty());
        v.sort_by_key(|s| s.to_lowercase());
        v.dedup();
        v
    }
}
