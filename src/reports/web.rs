//! Making a simple family web page.

use super::*;
use crate::rules::*;
use crate::data::*;
use std::fmt::Write as _;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[derive(Clone, Debug)]
pub struct WebOptions {
    pub title: String,
    pub hide_living: bool,
    pub include_notes: bool,
    pub who: Option<Vec<u32>>,
}

/// Creates a folder of plain web pages: an index, plus one page for each person.
pub fn write_web(db: &Db, o: &WebOptions, folder: &std::path::Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(folder)?;
    let chosen = |r: u32| o.who.as_ref().map(|l| l.contains(&r)).unwrap_or(true);
    let hidden = |r: u32| o.hide_living && maybe_living(db, r);
    let name_of = |r: u32| if hidden(r) { "Living".to_string() } else { db.name(r) };
    let style = "<style>body{font-family:Georgia,serif;max-width:46em;margin:2em auto;padding:0 1em;line-height:1.5}h1{font-size:1.5em}a{color:#34386a}li{margin:.15em 0}</style>";
    let mut people: Vec<&Person> = db.people.values().filter(|p| chosen(p.rin)).collect();
    people.sort_by_key(|p| sort_key(&p.name));
    let mut index = String::new();
    let _ = write!(index, "<!doctype html><meta charset=\"utf-8\"><title>{0}</title>{style}<h1>{0}</h1><p>{1} people</p><ul>", esc(&o.title), people.len());
    for p in &people {
        let _ = write!(index, "<li><a href=\"p{}.html\">{}</a></li>", p.rin, esc(&name_of(p.rin)));
    }
    index.push_str("</ul>");
    std::fs::write(folder.join("index.html"), index)?;
    let link = |r: u32| -> String {
        if chosen(r) {
            format!("<a href=\"p{r}.html\">{}</a>", esc(&name_of(r)))
        } else {
            esc(&name_of(r))
        }
    };
    let mut count = 0;
    for p in &people {
        let mut h = String::new();
        let _ = write!(h, "<!doctype html><meta charset=\"utf-8\"><title>{0}</title>{style}<p><a href=\"index.html\">Index</a></p><h1>{0}</h1>", esc(&name_of(p.rin)));
        if hidden(p.rin) {
            h.push_str("<p>Details are hidden because this person may still be living.</p>");
        } else {
            for (l, e) in [("Born", &p.birth), ("Christened", &p.chr), ("Died", &p.death), ("Buried", &p.burial)] {
                if let Some(s) = event_line(l, e) {
                    let _ = write!(h, "<p>{}</p>", esc(&s));
                }
            }
            if o.include_notes && !p.notes.trim().is_empty() {
                let _ = write!(h, "<h2>Notes</h2><p>{}</p>", esc(&p.notes).replace('\n', "<br>"));
            }
        }
        if let Some(m) = db.primary_parents(p.rin) {
            h.push_str("<h2>Parents</h2><ul>");
            for r in [m.husband, m.wife].into_iter().flatten() {
                let _ = write!(h, "<li>{}</li>", link(r));
            }
            h.push_str("</ul>");
        }
        for mr in &p.spouses {
            let Some(m) = db.marriages.get(mr) else { continue };
            h.push_str("<h2>Family</h2><ul>");
            if let Some(s) = db.spouse_in(*mr, p.rin) {
                let _ = write!(h, "<li>Spouse: {}</li>", link(s));
            }
            for c in &m.children {
                let _ = write!(h, "<li>Child: {}</li>", link(*c));
            }
            h.push_str("</ul>");
        }
        std::fs::write(folder.join(format!("p{}.html", p.rin)), h)?;
        count += 1;
    }
    Ok(count)
}
