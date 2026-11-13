//! Check/Repair: finding and fixing broken links in the file.

use crate::i18n::trf;
use super::*;

impl Db {
    /// Finds links that do not agree in both directions. With `repair` it fixes them.
    pub fn check(&mut self, repair: bool) -> Vec<String> {
        let mut out = Vec::new();
        let rins: Vec<u32> = self.people.keys().copied().collect();
        for rin in &rins {
            let p = self.people[rin].clone();
            for m in &p.spouses {
                let ok = self.marriages.get(m).map(|x| x.husband == Some(*rin) || x.wife == Some(*rin)).unwrap_or(false);
                if !ok {
                    out.push(trf("Individual {rin} points to marriage {mrin}, but the marriage does not point back.", &[("rin", &rin), ("mrin", &m)]));
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
                    out.push(trf("Individual {rin} points to parents in marriage {mrin}, but the marriage does not list them as a child.", &[("rin", &rin), ("mrin", &l.mrin)]));
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
                    out.push(trf("Husband's record (RIN {rin}) in marriage {mrin} has the wrong sex.", &[("rin", &h), ("mrin", &mrin)]));
                }
            }
            for c in &m.children {
                let ok = self.people.get(c).map(|p| p.parents.iter().any(|l| l.mrin == mrin)).unwrap_or(false);
                if !ok {
                    out.push(trf("Marriage {mrin} lists child {rin}, but that individual does not point back.", &[("mrin", &mrin), ("rin", &c)]));
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
