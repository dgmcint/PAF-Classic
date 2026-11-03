//! The family file: people, marriages, sources and repositories.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
    #[default]
    Unknown,
}

impl Sex {
    pub fn letter(self) -> &'static str {
        match self {
            Sex::Male => "M",
            Sex::Female => "F",
            Sex::Unknown => "U",
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Citation {
    pub source: u32,
    #[serde(default)]
    pub page: String,
    #[serde(default)]
    pub actual_text: String,
    #[serde(default)]
    pub comments: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub citations: Vec<Citation>,
}

impl Event {
    pub fn is_empty(&self) -> bool {
        self.date.trim().is_empty() && self.place.trim().is_empty() && self.citations.is_empty()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ordinance {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub temple: String,
}

impl Ordinance {
    pub fn is_empty(&self) -> bool {
        self.date.trim().is_empty() && self.temple.trim().is_empty()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomEvent {
    pub kind: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub place: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        self.address.trim().is_empty() && self.phone.trim().is_empty() && self.email.trim().is_empty()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Media {
    pub file: String,
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub in_scrapbook: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParentLink {
    pub mrin: u32,
    /// Biological, Adopted, Guardian, Challenged, Disproved, LDS Sealing or Unknown.
    #[serde(default = "default_rel")]
    pub relation: String,
    #[serde(default)]
    pub sealed: Ordinance,
}

fn default_rel() -> String {
    "Biological".into()
}

pub const PARENT_RELATIONS: [&str; 7] =
    ["Biological", "Adopted", "Guardian", "Challenged", "Disproved", "LDS Sealing", "Unknown"];

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Person {
    pub rin: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub aka: String,
    #[serde(default)]
    pub sex: Sex,
    #[serde(default)]
    pub birth: Event,
    #[serde(default)]
    pub chr: Event,
    #[serde(default)]
    pub death: Event,
    #[serde(default)]
    pub burial: Event,
    #[serde(default)]
    pub baptism: Ordinance,
    #[serde(default)]
    pub endowment: Ordinance,
    #[serde(default)]
    pub afn: String,
    #[serde(default)]
    pub custom_id: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub contact: Contact,
    #[serde(default)]
    pub media: Vec<Media>,
    #[serde(default)]
    pub custom: Vec<CustomEvent>,
    #[serde(default)]
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub spouses: Vec<u32>,
    #[serde(default)]
    pub parents: Vec<ParentLink>,
    #[serde(default)]
    pub changed: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Marriage {
    pub mrin: u32,
    #[serde(default)]
    pub husband: Option<u32>,
    #[serde(default)]
    pub wife: Option<u32>,
    #[serde(default)]
    pub children: Vec<u32>,
    #[serde(default)]
    pub event: Event,
    #[serde(default)]
    pub divorced: bool,
    #[serde(default)]
    pub sealing: Ordinance,
    #[serde(default)]
    pub custom: Vec<CustomEvent>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub citations: Vec<Citation>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Source {
    pub id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub publication: String,
    #[serde(default)]
    pub abbreviation: String,
    #[serde(default)]
    pub repository: Option<u32>,
    #[serde(default)]
    pub comments: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub contact: Contact,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Prefs {
    #[serde(default = "yes")]
    pub use_lds: bool,
    #[serde(default = "yes")]
    pub auto_slash: bool,
    #[serde(default)]
    pub show_relationships: bool,
    #[serde(default = "yes")]
    pub edit_marriage_when_created: bool,
    #[serde(default = "yes")]
    pub use_list_navigating: bool,
    #[serde(default)]
    pub verify_new_places: bool,
    #[serde(default)]
    pub day_first_numeric: bool,
    #[serde(default)]
    pub prepared_by: Contact,
    #[serde(default)]
    pub prepared_name: String,
    #[serde(default)]
    pub log_changes: bool,
    #[serde(default = "backup_default")]
    pub backup_frequency: u32,
    #[serde(default)]
    pub closes_since_backup: u32,
    #[serde(default)]
    pub changed_since_backup: bool,
    /// Show each person's number after their name on the screens. Off is an estimate:
    /// the original keeps this per file and its default wasn't found.
    #[serde(default)]
    pub show_rin: bool,
    /// 0 none, 1 dates only, 2 dates and family information.
    #[serde(default = "floating_default")]
    pub floating_info: u8,
    #[serde(default = "locked_default")]
    pub locked_info: u8,
    #[serde(default = "wait_default")]
    pub info_wait_tenths: u32,
    #[serde(default = "yes")]
    pub show_lds_reports: bool,
    #[serde(default = "yes")]
    pub check_duplicates: bool,
    #[serde(default = "yes")]
    pub use_fathers_surname: bool,
    #[serde(default)]
    pub enter_as_tab: bool,
    #[serde(default)]
    pub allow_afn_edit: bool,
    #[serde(default = "yes")]
    pub shade_reports: bool,
    #[serde(default = "age_low")]
    pub mother_low: i32,
    #[serde(default = "age_high")]
    pub mother_high: i32,
    #[serde(default = "age_low")]
    pub father_low: i32,
    #[serde(default = "age_high")]
    pub father_high: i32,
    #[serde(default = "span_default")]
    pub spouse_span: i32,
    #[serde(default = "yes")]
    pub check_lds: bool,
    #[serde(default = "yes")]
    pub check_order: bool,
    #[serde(default = "merge_years_default")]
    pub merge_years: i32,
    #[serde(default = "yes")]
    pub merge_notes: bool,
    #[serde(default = "yes")]
    pub merge_citations: bool,
    #[serde(default = "yes")]
    pub merge_media: bool,
    #[serde(default = "yes")]
    pub merge_confirm: bool,
}

fn yes() -> bool {
    true
}

fn backup_default() -> u32 {
    1
}

fn floating_default() -> u8 {
    1
}

fn locked_default() -> u8 {
    2
}

fn wait_default() -> u32 {
    10
}

fn age_low() -> i32 {
    16
}

fn age_high() -> i32 {
    45
}

fn span_default() -> i32 {
    15
}

fn merge_years_default() -> i32 {
    10
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            use_lds: true,
            auto_slash: true,
            show_relationships: false,
            edit_marriage_when_created: true,
            use_list_navigating: true,
            verify_new_places: false,
            day_first_numeric: false,
            prepared_by: Contact::default(),
            prepared_name: String::new(),
            log_changes: false,
            backup_frequency: backup_default(),
            closes_since_backup: 0,
            changed_since_backup: false,
            show_rin: false,
            floating_info: floating_default(),
            locked_info: locked_default(),
            info_wait_tenths: wait_default(),
            show_lds_reports: true,
            check_duplicates: true,
            use_fathers_surname: true,
            enter_as_tab: false,
            allow_afn_edit: false,
            shade_reports: true,
            mother_low: age_low(),
            mother_high: age_high(),
            father_low: age_low(),
            father_high: age_high(),
            spouse_span: span_default(),
            check_lds: true,
            check_order: true,
            merge_years: merge_years_default(),
            merge_notes: true,
            merge_citations: true,
            merge_media: true,
            merge_confirm: true,
        }
    }
}

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

pub fn given_of(name: &str) -> String {
    match name.find('/') {
        Some(i) => name[..i].trim().to_string(),
        None => name.trim().to_string(),
    }
}

pub fn surname_of(name: &str) -> String {
    let mut parts = name.split('/');
    parts.next();
    match parts.next() {
        Some(s) => s.trim().to_string(),
        None => String::new(),
    }
}

pub fn suffix_of(name: &str) -> String {
    let mut parts = name.split('/');
    parts.next();
    parts.next();
    parts.next().map(|s| s.trim().to_string()).unwrap_or_default()
}

/// A name with its surname marks taken out, as it appears on screen.
pub fn plain_name(name: &str) -> String {
    let joined = [given_of(name), surname_of(name), suffix_of(name)];
    joined.iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ")
}

/// Endings that aren't surnames, ignoring case.
const NAME_SUFFIXES: [&str; 19] = [
    "Jr.", "Sr.", "Jr", "Sr", "II", "III", "IV", "V", "VI", "PhD", "MD", "M.D.", "DDS", "D.D.S.", "Esq", "Esq.", "Esquire", "Second", "Third",
];

/// Words that join the surname when they come right before it.
const SURNAME_WORDS: [&str; 5] = ["de", "von", "van", "Mac", "Mc"];

/// Puts slashes around the surname when the person typed no marks. A single word has
/// an empty surname. An ending like Jr. is set aside first. The surname is then the last word,
/// with "de la" or "van der" from the fourth word on, or one of the joining words from the third.
pub fn with_surname_marks(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() || name.contains('/') {
        return name.to_string();
    }
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() == 1 {
        return format!("{name} //");
    }
    let is = |word: &str, other: &str| word.eq_ignore_ascii_case(other);
    let has_suffix = NAME_SUFFIXES.iter().any(|s| is(words[words.len() - 1], s));
    let last = if has_suffix { words.len() - 2 } else { words.len() - 1 };
    let start = if last >= 3 && ((is(words[last - 2], "de") && is(words[last - 1], "la")) || (is(words[last - 2], "van") && is(words[last - 1], "der"))) {
        last - 2
    } else if last > 1 && SURNAME_WORDS.iter().any(|w| is(words[last - 1], w)) {
        last - 1
    } else {
        last
    };
    let mut out = format!("/{}/", words[start..=last].join(" "));
    if start > 0 {
        out = format!("{} {out}", words[..start].join(" "));
    }
    if has_suffix {
        out = format!("{out} {}", words[words.len() - 1]);
    }
    out
}

/// Sorted by surname, then given name. People without a surname sort first.
pub fn sort_key(name: &str) -> (String, String, String) {
    (surname_of(name).to_lowercase(), given_of(name).to_lowercase(), suffix_of(name).to_lowercase())
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

    /// Makes a marriage with this person in the right position for their sex.
    pub fn new_marriage_for(&mut self, rin: u32) -> u32 {
        let mrin = self.next_mrin();
        let mut m = Marriage { mrin, ..Default::default() };
        let sex = self.person(rin).map(|p| p.sex).unwrap_or_default();
        if sex == Sex::Female {
            m.wife = Some(rin);
        } else {
            m.husband = Some(rin);
        }
        self.marriages.insert(mrin, m);
        if let Some(p) = self.people.get_mut(&rin) {
            p.spouses.push(mrin);
        }
        mrin
    }

    pub fn add_spouse(&mut self, rin: u32, spouse: u32) -> u32 {
        // Fill an open place in an existing marriage first, so children stay with the couple.
        let open = self.person(rin).and_then(|p| {
            p.spouses.iter().copied().find(|m| {
                self.marriages.get(m).map(|x| x.husband.is_none() || x.wife.is_none()).unwrap_or(false)
            })
        });
        let mrin = open.unwrap_or_else(|| self.new_marriage_for(rin));
        self.link_spouse_to(mrin, spouse);
        mrin
    }

    pub fn link_spouse_to(&mut self, mrin: u32, spouse: u32) {
        let sex = self.person(spouse).map(|p| p.sex).unwrap_or_default();
        if let Some(m) = self.marriages.get_mut(&mrin) {
            let as_husband = match sex {
                Sex::Male => true,
                Sex::Female => false,
                Sex::Unknown => m.husband.is_none(),
            };
            // Nobody already in the marriage may be pushed out. Someone in the wanted place moves
            // to the other one, and a full marriage is left as it is.
            let (wanted, other) = if as_husband { (m.husband, m.wife) } else { (m.wife, m.husband) };
            match (wanted, other) {
                (Some(w), _) if w == spouse => {}
                (_, Some(o)) if o == spouse => {}
                (Some(_), Some(_)) => return,
                (Some(w), None) => {
                    m.husband = Some(if as_husband { spouse } else { w });
                    m.wife = Some(if as_husband { w } else { spouse });
                }
                (None, _) => {
                    if as_husband {
                        m.husband = Some(spouse);
                    } else {
                        m.wife = Some(spouse);
                    }
                }
            }
        }
        if let Some(p) = self.people.get_mut(&spouse) {
            if !p.spouses.contains(&mrin) {
                p.spouses.push(mrin);
            }
        }
    }

    pub fn add_child(&mut self, mrin: u32, child: u32) {
        if let Some(m) = self.marriages.get_mut(&mrin) {
            if !m.children.contains(&child) {
                m.children.push(child);
            }
        }
        if let Some(p) = self.people.get_mut(&child) {
            if !p.parents.iter().any(|l| l.mrin == mrin) {
                p.parents.push(ParentLink { mrin, relation: default_rel(), sealed: Ordinance::default() });
            }
        }
    }

    /// Unlinks a person from one set of parents.
    pub fn unlink_child(&mut self, mrin: u32, child: u32) {
        if let Some(m) = self.marriages.get_mut(&mrin) {
            m.children.retain(|c| *c != child);
        }
        if let Some(p) = self.people.get_mut(&child) {
            p.parents.retain(|l| l.mrin != mrin);
        }
        self.drop_if_empty(mrin);
    }

    /// Unlinks a person from a spouse. The marriage stays with the other spouse and the children.
    pub fn unlink_spouse(&mut self, mrin: u32, rin: u32) {
        if let Some(m) = self.marriages.get_mut(&mrin) {
            if m.husband == Some(rin) {
                m.husband = None;
            }
            if m.wife == Some(rin) {
                m.wife = None;
            }
        }
        if let Some(p) = self.people.get_mut(&rin) {
            p.spouses.retain(|x| *x != mrin);
        }
        self.drop_if_empty(mrin);
    }

    fn drop_if_empty(&mut self, mrin: u32) {
        let empty = self
            .marriages
            .get(&mrin)
            .map(|m| m.husband.is_none() && m.wife.is_none() && m.children.is_empty())
            .unwrap_or(false);
        if empty {
            self.marriages.remove(&mrin);
        }
    }

    pub fn delete_marriage(&mut self, mrin: u32) {
        if let Some(m) = self.marriages.remove(&mrin) {
            for r in [m.husband, m.wife].into_iter().flatten() {
                if let Some(p) = self.people.get_mut(&r) {
                    p.spouses.retain(|x| *x != mrin);
                }
            }
            for c in m.children {
                if let Some(p) = self.people.get_mut(&c) {
                    p.parents.retain(|l| l.mrin != mrin);
                }
            }
        }
    }

    pub fn delete_person(&mut self, rin: u32) {
        let (sp, par) = match self.person(rin) {
            Some(p) => (p.spouses.clone(), p.parents.iter().map(|l| l.mrin).collect::<Vec<_>>()),
            None => return,
        };
        for m in sp {
            self.unlink_spouse(m, rin);
        }
        for m in par {
            self.unlink_child(m, rin);
        }
        self.people.remove(&rin);
        if self.home == Some(rin) {
            self.home = self.people.keys().next().copied();
        }
        if self.root_person == Some(rin) {
            self.root_person = None;
        }
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

    /// Finds links that do not agree in both directions. With `repair` it fixes them.
    pub fn check(&mut self, repair: bool) -> Vec<String> {
        let mut out = Vec::new();
        let rins: Vec<u32> = self.people.keys().copied().collect();
        for rin in &rins {
            let p = self.people[rin].clone();
            for m in &p.spouses {
                let ok = self.marriages.get(m).map(|x| x.husband == Some(*rin) || x.wife == Some(*rin)).unwrap_or(false);
                if !ok {
                    out.push(format!("Individual {rin} points to marriage {m}, but the marriage does not point back."));
                    if repair {
                        if let Some(x) = self.people.get_mut(rin) {
                            x.spouses.retain(|v| v != m);
                        }
                    }
                }
            }
            for l in &p.parents {
                let ok = self.marriages.get(&l.mrin).map(|x| x.children.contains(rin)).unwrap_or(false);
                if !ok {
                    out.push(format!("Individual {rin} points to parents in marriage {}, but the marriage does not list them as a child.", l.mrin));
                    if repair {
                        if let Some(x) = self.people.get_mut(rin) {
                            x.parents.retain(|v| v.mrin != l.mrin);
                        }
                    }
                }
            }
        }
        let mrins: Vec<u32> = self.marriages.keys().copied().collect();
        for mrin in mrins {
            let m = self.marriages[&mrin].clone();
            for (label, who) in [("husband", m.husband), ("wife", m.wife)] {
                if let Some(r) = who {
                    let ok = self.people.get(&r).map(|p| p.spouses.contains(&mrin)).unwrap_or(false);
                    if !ok {
                        out.push(format!("Marriage {mrin} points to {label} {r}, but that individual does not point back."));
                        if repair {
                            if self.people.contains_key(&r) {
                                self.people.get_mut(&r).unwrap().spouses.push(mrin);
                            } else if let Some(x) = self.marriages.get_mut(&mrin) {
                                if label == "husband" {
                                    x.husband = None;
                                } else {
                                    x.wife = None;
                                }
                            }
                        }
                    }
                }
            }
            if let (Some(h), Some(_)) = (m.husband, m.wife) {
                if self.people.get(&h).map(|p| p.sex == Sex::Female).unwrap_or(false) {
                    out.push(format!("Husband's record (RIN {h}) in marriage {mrin} has the wrong sex."));
                }
            }
            for c in &m.children {
                let ok = self.people.get(c).map(|p| p.parents.iter().any(|l| l.mrin == mrin)).unwrap_or(false);
                if !ok {
                    out.push(format!("Marriage {mrin} lists child {c}, but that individual does not point back."));
                    if repair {
                        if self.people.contains_key(c) {
                            self.people.get_mut(c).unwrap().parents.push(ParentLink {
                                mrin,
                                relation: default_rel(),
                                sealed: Ordinance::default(),
                            });
                        } else if let Some(x) = self.marriages.get_mut(&mrin) {
                            x.children.retain(|v| v != c);
                        }
                    }
                }
            }
        }
        out
    }
}

/// A small made-up family used for the Help menu's sample and for tests.
pub fn sample_family() -> Db {
    let mut db = Db::default();
    let add = |db: &mut Db, name: &str, sex: Sex, b: (&str, &str), d: (&str, &str)| {
        let mut p = Person { name: name.into(), sex, ..Default::default() };
        p.birth.date = b.0.into();
        p.birth.place = b.1.into();
        p.death.date = d.0.into();
        p.death.place = d.1.into();
        db.add_person(p)
    };
    let me = add(&mut db, "Thomas Edward /Harrow/", Sex::Male, ("14 Mar 1921", "Boise, Ada, Idaho, USA"), ("2 Oct 1998", "Boise, Ada, Idaho, USA"));
    let wife = add(&mut db, "Clara Louise /Whitcomb/", Sex::Female, ("8 Jun 1924", "Ogden, Weber, Utah, USA"), ("30 Jan 2003", "Boise, Ada, Idaho, USA"));
    let dad = add(&mut db, "Edward James /Harrow/", Sex::Male, ("Abt 1890", "York, Yorkshire, England"), ("12 Feb 1960", "Boise, Ada, Idaho, USA"));
    let mum = add(&mut db, "Ida May /Larkin/", Sex::Female, ("3 Sep 1894", "Provo, Utah, Utah, USA"), ("7 Jul 1971", "Boise, Ada, Idaho, USA"));
    let gdad = add(&mut db, "William /Harrow/", Sex::Male, ("1862", "York, Yorkshire, England"), ("1931", "York, Yorkshire, England"));
    let gmum = add(&mut db, "Hannah /Booth/", Sex::Female, ("1865", "Leeds, Yorkshire, England"), ("1940", "York, Yorkshire, England"));
    let c1 = add(&mut db, "Robert Thomas /Harrow/", Sex::Male, ("22 Apr 1947", "Boise, Ada, Idaho, USA"), ("", ""));
    let c2 = add(&mut db, "Margaret Ann /Harrow/", Sex::Female, ("5 Nov 1949", "Boise, Ada, Idaho, USA"), ("", ""));
    let c3 = add(&mut db, "Paul Edward /Harrow/", Sex::Male, ("19 Aug 1953", "Boise, Ada, Idaho, USA"), ("", ""));
    let gc = add(&mut db, "Susan /Harrow/", Sex::Female, ("11 Jan 1975", "Boise, Ada, Idaho, USA"), ("", ""));
    let inlaw = add(&mut db, "Janet /Moore/", Sex::Female, ("1948", "Nampa, Canyon, Idaho, USA"), ("", ""));
    let m1 = db.new_marriage_for(me);
    db.link_spouse_to(m1, wife);
    db.marriages.get_mut(&m1).unwrap().event = Event { date: "18 May 1946".into(), place: "Boise, Ada, Idaho, USA".into(), citations: vec![] };
    for c in [c1, c2, c3] {
        db.add_child(m1, c);
    }
    let m2 = db.new_marriage_for(dad);
    db.link_spouse_to(m2, mum);
    db.marriages.get_mut(&m2).unwrap().event = Event { date: "2 Jun 1918".into(), place: "Provo, Utah, Utah, USA".into(), citations: vec![] };
    db.add_child(m2, me);
    let m3 = db.new_marriage_for(gdad);
    db.link_spouse_to(m3, gmum);
    db.add_child(m3, dad);
    let m4 = db.new_marriage_for(c1);
    db.link_spouse_to(m4, inlaw);
    db.add_child(m4, gc);
    db.people.get_mut(&me).unwrap().notes = "Sample family made up for trying out the program.".into();
    db.home = Some(me);
    db
}
