//! Adding and removing the links between people and marriages.

use super::*;

impl Db {
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
}
