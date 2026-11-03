//! Pretend actions by the person, for testing the program from the outside with `--do`.
use eframe::egui;
use egui::{Key, Modifiers, pos2};

/// One action: a click somewhere, typed text or a key.
#[derive(Clone)]
pub enum Step {
    Click(f32, f32),
    Type(String),
    Key(Key),
}

/// Reads steps such as `dbl 250 219; wait 20; type Anna /Lee/; key Tab; click 230 580`. Each step
/// happens some frames after the one before.
pub fn parse(text: &str) -> Vec<(u32, Step)> {
    let mut out = Vec::new();
    let mut frame = 20;
    for part in text.split(';') {
        let part = part.trim();
        let (word, rest) = part.split_once(' ').unwrap_or((part, ""));
        let xy: Vec<f32> = rest.split_whitespace().filter_map(|n| n.parse().ok()).collect();
        match (word, xy.as_slice()) {
            ("wait", [n]) => frame += *n as u32,
            ("click", [x, y]) => {
                out.push((frame, Step::Click(*x, *y)));
                frame += 12;
            }
            ("dbl", [x, y]) => {
                out.push((frame, Step::Click(*x, *y)));
                out.push((frame + 2, Step::Click(*x, *y)));
                frame += 14;
            }
            ("type", _) => {
                out.push((frame, Step::Type(rest.to_string())));
                frame += 6;
            }
            ("key", _) => {
                if let Some(k) = Key::from_name(rest) {
                    out.push((frame, Step::Key(k)));
                }
                frame += 8;
            }
            _ => {}
        }
    }
    out
}

/// Adds this frame's actions to the window's input.
pub fn feed(steps: &[(u32, Step)], frame: u32, raw: &mut egui::RawInput) {
    let modifiers = Modifiers::NONE;
    for (at, step) in steps {
        match step {
            Step::Click(x, y) => {
                let pos = pos2(*x, *y);
                if *at == frame {
                    raw.events.push(egui::Event::PointerMoved(pos));
                } else if *at + 1 == frame {
                    raw.events.push(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, modifiers });
                    raw.events.push(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, modifiers });
                }
            }
            Step::Type(t) if *at == frame => raw.events.push(egui::Event::Text(t.clone())),
            Step::Key(k) if *at == frame => {
                raw.events.push(egui::Event::Key { key: *k, physical_key: None, pressed: true, repeat: false, modifiers });
                raw.events.push(egui::Event::Key { key: *k, physical_key: None, pressed: false, repeat: false, modifiers });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::*;
    use crate::dialogs::Link;
    use crate::model::*;
    use std::path::PathBuf;

    fn blank() -> App {
        App::new(Some(PathBuf::from("/nonexistent/x.png")), false, None, None, None, None)
    }

    fn sample() -> App {
        App::new(Some(PathBuf::from("/nonexistent/x.png")), true, None, None, None, None)
    }

    #[test]
    fn adding_a_child_links_them_both_ways() {
        let mut a = sample();
        let p = a.primary.unwrap();
        let before = a.db.children_of(p).len();
        let rin = a.db.add_person(Person { name: "New /Child/".into(), ..Default::default() });
        a.apply_link(Link::Child(p), rin);
        assert_eq!(a.db.children_of(p).len(), before + 1);
        assert!(!a.db.person(rin).unwrap().parents.is_empty());
        assert!(a.db.clone().check(false).is_empty());
    }

    #[test]
    fn adding_a_father_then_a_mother_makes_one_couple() {
        let mut a = blank();
        let child = a.db.add_person(Person { name: "Kid /Lee/".into(), ..Default::default() });
        a.touch();
        a.db.prefs.edit_marriage_when_created = false;
        let dad = a.db.add_person(Person { name: "Dad /Lee/".into(), ..Default::default() });
        a.apply_link(Link::Father(child), dad);
        let mum = a.db.add_person(Person { name: "Mum /Hill/".into(), ..Default::default() });
        a.apply_link(Link::Mother(child), mum);
        assert_eq!(a.db.father(child), Some(dad));
        assert_eq!(a.db.mother(child), Some(mum));
        assert_eq!(a.db.person(child).unwrap().parents.len(), 1);
        assert_eq!(a.db.person(dad).unwrap().sex, Sex::Male);
        assert_eq!(a.db.person(mum).unwrap().sex, Sex::Female);
        assert!(a.db.clone().check(false).is_empty());
    }

    #[test]
    fn a_new_spouse_never_pushes_out_the_person_already_married() {
        let mut a = blank();
        let first = a.db.add_person(Person { name: "Anna /Lee/".into(), ..Default::default() });
        let second = a.db.add_person(Person { name: "Paul /Hart/".into(), sex: Sex::Male, ..Default::default() });
        let m = a.db.add_spouse(first, second);
        let got = a.db.marriages[&m].clone();
        assert_eq!(got.husband, Some(second));
        assert_eq!(got.wife, Some(first));
        assert_eq!(a.db.spouse_in(m, first), Some(second));
        assert!(a.db.clone().check(false).is_empty());
        // A third person doesn't get into a full marriage, and gets one of their own.
        let third = a.db.add_person(Person { name: "Sam /Roe/".into(), sex: Sex::Male, ..Default::default() });
        a.db.link_spouse_to(m, third);
        assert_eq!(a.db.marriages[&m].husband, Some(second));
        assert!(!a.db.person(third).unwrap().spouses.contains(&m) || a.db.marriages[&m].husband == Some(third));
    }

    #[test]
    fn the_c_key_returns_to_the_child_who_was_primary() {
        let mut a = sample();
        let kid = a.primary.unwrap();
        let dad = a.db.father(kid).unwrap();
        a.to_primary(dad);
        a.run(Cmd::NextChild);
        assert_eq!(a.primary, Some(kid));
    }

    #[test]
    fn import_adds_to_the_open_family_without_changing_it() {
        let mut a = sample();
        let (count, marriages) = (a.db.people.len(), a.db.marriages.len());
        let first = a.db.person(1).cloned();
        a.db.absorb(sample_family());
        assert_eq!(a.db.people.len(), count * 2);
        assert_eq!(a.db.marriages.len(), marriages * 2);
        assert_eq!(a.db.person(1).cloned(), first);
        assert!(a.db.clone().check(false).is_empty());
    }

    #[test]
    fn the_settings_read_from_the_original_are_the_defaults() {
        let p = Prefs::default();
        assert_eq!((p.mother_low, p.mother_high, p.father_low, p.father_high, p.spouse_span), (16, 45, 16, 45, 15));
        assert_eq!((p.floating_info, p.locked_info, p.info_wait_tenths, p.backup_frequency, p.merge_years), (1, 2, 10, 1, 10));
        assert!(p.use_list_navigating && p.check_duplicates && p.edit_marriage_when_created && !p.allow_afn_edit && !p.verify_new_places);
    }

    #[test]
    fn a_saved_family_reads_back_the_same() {
        let path = std::env::temp_dir().join("paf-save-test.pafc");
        let mut a = sample();
        let db = a.db.clone();
        a.path = Some(path.clone());
        a.touch();
        let back: Db = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(back.people, db.people);
        assert_eq!(back.marriages, db.marriages);
    }
}
