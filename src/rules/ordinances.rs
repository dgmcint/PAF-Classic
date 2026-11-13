//! The ordinance status letters shown beside each person.

use super::*;
use crate::data::*;

/// What an ordinance field means for its letter.
#[derive(Clone, Copy, PartialEq)]
enum Mark {
    Blank,
    Dash,
    Done,
    Pending,
}

#[derive(Clone, Copy, PartialEq)]
enum Rite {
    Baptism,
    Endowment,
    SealParents,
    SealSpouse,
}

fn ordinance_mark(field: &str, rite: Rite) -> Mark {
    let t = field.trim().to_lowercase();
    let is = |w: &str| t == w || t.starts_with(&format!("{w} "));
    if t.is_empty() {
        Mark::Blank
    } else if is("bic") {
        if rite == Rite::SealParents { Mark::Done } else { Mark::Blank }
    } else if is("cancelled") || is("completed") || is("done") || is("pre-1970") {
        Mark::Done
    } else if is("child") || is("infant") {
        if matches!(rite, Rite::Baptism | Rite::Endowment) { Mark::Dash } else { Mark::Blank }
    } else if is("cleared") || is("submitted") || is("sub") {
        Mark::Pending
    } else if is("dns") || is("dns/can") || is("uncleared") {
        Mark::Dash
    } else if is("stillborn") {
        if rite == Rite::SealSpouse { Mark::Blank } else { Mark::Dash }
    } else if parse_date(&t, false).is_some() {
        Mark::Done
    } else {
        Mark::Blank
    }
}

fn mark_letter(mark: Mark, letter: char) -> char {
    match mark {
        Mark::Blank => ' ',
        Mark::Dash => '-',
        Mark::Done => letter,
        Mark::Pending => letter.to_ascii_lowercase(),
    }
}

/// The ordinance letters on the Family screen, in the original's order: B, E, P, S.
/// With no marriages the last one is a dash. A woman shows S from any one sealed marriage, and a
/// man only when all are sealed.
pub fn ordinance_flags(db: &Db, rin: u32) -> String {
    let Some(p) = db.person(rin) else { return String::new() };
    let par = p.parents.first().map(|l| l.sealed.date.as_str()).unwrap_or("");
    let marks: Vec<Mark> = p.spouses.iter().filter_map(|m| db.marriages.get(m)).map(|m| ordinance_mark(&m.sealing.date, Rite::SealSpouse)).collect();
    let spouse = if marks.is_empty() {
        '-'
    } else if p.sex == Sex::Female {
        let mut shown = '-';
        for m in &marks {
            match m {
                Mark::Done => {
                    shown = 'S';
                    break;
                }
                Mark::Pending => {
                    shown = 's';
                    break;
                }
                Mark::Blank => shown = ' ',
                Mark::Dash => {}
            }
        }
        shown
    } else {
        let mut shown = 'S';
        for m in &marks {
            if shown == ' ' {
                break;
            }
            shown = match m {
                Mark::Blank => ' ',
                Mark::Dash => '-',
                Mark::Pending => shown.to_ascii_lowercase(),
                Mark::Done => shown,
            };
        }
        shown
    };
    // The parents' letter shows only when done or pending, never as a dash.
    let parents = match ordinance_mark(par, Rite::SealParents) {
        Mark::Dash => ' ',
        m => mark_letter(m, 'P'),
    };
    format!(
        "{} {} {} {}",
        mark_letter(ordinance_mark(&p.baptism.date, Rite::Baptism), 'B'),
        mark_letter(ordinance_mark(&p.endowment.date, Rite::Endowment), 'E'),
        parents,
        spouse
    )
}
