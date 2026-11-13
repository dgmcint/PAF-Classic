//! Moving around the family: who is shown, and linking new people into it.

use super::*;
use crate::dialogs::*;
use crate::data::*;

impl App {
    pub fn current_marriage(&self, rin: u32) -> Option<u32> {
        let p = self.db.person(rin)?;
        let idx = self.shown_marriage.get(&rin).copied().unwrap_or(0).min(p.spouses.len().saturating_sub(1));
        p.spouses.get(idx).copied()
    }

    pub fn current_parents(&self, rin: u32) -> Option<u32> {
        let p = self.db.person(rin)?;
        let idx = self.shown_parents.get(&rin).copied().unwrap_or(0).min(p.parents.len().saturating_sub(1));
        p.parents.get(idx).map(|l| l.mrin)
    }

    /// Who the family screen shows: (father, mother, spouse, children).
    pub fn family_of(&self, rin: u32) -> (Option<u32>, Option<u32>, Option<u32>, Vec<u32>) {
        let parents = self.current_parents(rin).and_then(|m| self.db.marriages.get(&m));
        let (f, mo) = parents.map(|m| (m.husband, m.wife)).unwrap_or((None, None));
        let m = self.current_marriage(rin);
        let spouse = m.and_then(|m| self.db.spouse_in(m, rin));
        let kids = m.and_then(|m| self.db.marriages.get(&m)).map(|m| m.children.clone()).unwrap_or_default();
        (f, mo, spouse, kids)
    }

    /// Whichever way a parent and child swap places, the child counts as having been primary.
    pub fn to_primary(&mut self, rin: u32) {
        if let Some(old) = self.primary {
            if self.db.children_of(old).contains(&rin) {
                self.last_child.insert(old, rin);
            } else if self.db.children_of(rin).contains(&old) {
                self.last_child.insert(rin, old);
            }
        }
        self.primary = Some(rin);
        self.selected = Some(rin);
        self.slot = self.home_slot();
    }

    /// The C key's rule from the guide, page 26: the last child who was primary. Otherwise the
    /// Descendancy List when Use List When Navigating is on, or the oldest child with
    /// descendants, then the youngest child.
    pub(crate) fn c_key(&mut self, rin: u32) {
        let kids = self.db.children_of(rin);
        if kids.is_empty() {
            return;
        }
        if let Some(c) = self.last_child.get(&rin).copied().filter(|c| kids.contains(c)) {
            self.to_primary(c);
        } else if self.db.prefs.use_list_navigating {
            self.push(|id| Dialog::DescendancyList { id, root: rin });
        } else if let Some(c) = kids.iter().copied().find(|c| self.db.has_descendants(*c)).or(kids.last().copied()) {
            self.to_primary(c);
        }
    }

    pub fn apply_link(&mut self, link: Link, new_rin: u32) {
        match link {
            Link::None => {}
            Link::Spouse(of) => {
                let m = self.db.add_spouse(of, new_rin);
                let idx = self.db.person(of).and_then(|p| p.spouses.iter().position(|x| *x == m)).unwrap_or(0);
                self.shown_marriage.insert(of, idx);
                self.log("Add Sp", new_rin);
                self.maybe_edit_marriage(m);
            }
            Link::Child(of) => {
                let m = match self.current_marriage(of) {
                    Some(m) => m,
                    None => self.db.new_marriage_for(of),
                };
                self.db.add_child(m, new_rin);
                self.log("Add Ch", new_rin);
            }
            Link::ChildOfMarriage(m) => {
                self.db.add_child(m, new_rin);
                self.log("Add Ch", new_rin);
            }
            Link::Father(of) | Link::Mother(of) => {
                let want_father = matches!(link, Link::Father(_));
                let open = self.current_parents(of).filter(|m| {
                    self.db.marriages.get(m).map(|x| if want_father { x.husband.is_none() } else { x.wife.is_none() }).unwrap_or(false)
                });
                let m = match open {
                    Some(m) => m,
                    None => {
                        let m = self.db.new_marriage_for(new_rin);
                        if let Some(x) = self.db.marriages.get_mut(&m) {
                            x.husband = None;
                            x.wife = None;
                            if want_father {
                                x.husband = Some(new_rin);
                            } else {
                                x.wife = Some(new_rin);
                            }
                        }
                        self.db.add_child(m, of);
                        m
                    }
                };
                if let Some(x) = self.db.marriages.get_mut(&m) {
                    if want_father {
                        x.husband = Some(new_rin);
                    } else {
                        x.wife = Some(new_rin);
                    }
                }
                if let Some(p) = self.db.people.get_mut(&new_rin) {
                    if !p.spouses.contains(&m) {
                        p.spouses.push(m);
                    }
                    if p.sex == Sex::Unknown {
                        p.sex = if want_father { Sex::Male } else { Sex::Female };
                    }
                }
                self.log("Add Pa", new_rin);
                self.maybe_edit_marriage(m);
            }
        }
        self.touch();
    }

    fn maybe_edit_marriage(&mut self, m: u32) {
        if self.db.prefs.edit_marriage_when_created {
            if let Some(draft) = self.db.marriages.get(&m).cloned() {
                self.push(|id| Dialog::EditMarriage { id, draft: Box::new(draft), notes_open: false, sources_open: false });
            }
        }
    }

    pub fn start_add(&mut self, link: Link) {
        let sex = match link {
            Link::Father(_) => Sex::Male,
            Link::Mother(_) => Sex::Female,
            Link::Spouse(of) => match self.db.person(of).map(|p| p.sex) {
                Some(Sex::Male) => Sex::Female,
                Some(Sex::Female) => Sex::Male,
                _ => Sex::Unknown,
            },
            _ => Sex::Unknown,
        };
        // Children take the family name, as a starting point the person can change.
        let surname = match link {
            Link::Father(of) | Link::Child(of) => self.db.person(of).map(|p| surname_of(&p.name)).unwrap_or_default(),
            Link::ChildOfMarriage(m) => self.db.marriages.get(&m).and_then(|x| x.husband).and_then(|h| self.db.person(h)).map(|p| surname_of(&p.name)).unwrap_or_default(),
            _ => String::new(),
        };
        let name = if surname.is_empty() { String::new() } else { format!(" /{surname}/") };
        if matches!(link, Link::None) {
            self.open_new_person(Link::None, sex, name);
        } else {
            self.push(|id| Dialog::AddOrSelect { id, link, sex, name });
        }
    }

    pub fn open_new_person(&mut self, link: Link, sex: Sex, name: String) {
        let draft = Person { name, sex, ..Default::default() };
        self.push(|id| Dialog::new_edit_person(id, draft, true, link));
    }

    pub fn open_edit_person(&mut self, rin: u32) {
        if let Some(p) = self.db.person(rin).cloned() {
            self.push(|id| Dialog::new_edit_person(id, p, false, Link::None));
        }
    }
}
