//! The windows that open over the main screen: shared types and helpers.
use crate::i18n::tr;
use crate::app::*;
use crate::calc;
use crate::gedcom;
use crate::model::*;
use crate::reports::{self, Op, Report};
use egui::{Align2, Color32, RichText, Stroke, TextEdit, vec2};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Link {
    None,
    Spouse(u32),
    Child(u32),
    #[allow(dead_code)]
    ChildOfMarriage(u32),
    Father(u32),
    Mother(u32),
}

/// What a person picked from the Individual List is for.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PickFor {
    GoTo,
    Link(Link),
    RelationA(u64),
    RelationB(u64),
    ReportStart(u64),
    FocusStart(u64),
    Root(u64),
    Home(u64),
}

pub const INDIVIDUAL_EVENTS: [&str; 24] = [
    "Adoption", "Adult Christening", "Baptism", "Bar Mitzvah", "Bas Mitzvah", "Blessing", "Census", "Confirmation", "Cremation", "Education", "Emigration",
    "First Communion", "Graduation", "Immigration", "Military", "Naturalization", "Occupation", "Ordination", "Physical Description", "Probate", "Religion",
    "Residence", "Retirement", "Will",
];
pub const MARRIAGE_EVENTS: [&str; 8] = ["Annulment", "Census", "Divorce Filed", "Engagement", "Marriage Banns", "Marriage Contract", "Marriage License", "Marriage Settlement"];
pub const FILTER_FIELDS: [&str; 16] = [
    "Full Name", "Surname", "Given Name", "Sex", "Birth Date", "Birth Place", "Christening Date", "Christening Place", "Death Date", "Death Place", "Burial Date",
    "Burial Place", "Marriage Date", "Spouse", "AFN", "Notes",
];
pub const FILTER_OPS: [&str; 6] = ["contains", "equals", "is empty", "is not empty", "is before", "is after"];
pub const REPLACE_FIELDS: [&str; 6] = ["Multimedia file names", "Places", "Dates", "Full Name", "Notes", "Temple Codes"];
pub const REPORT_TABS: [&str; 10] = ["Pedigree", "Family Group", "Ancestry", "Descendants", "Individual Summary", "Lists", "Books", "Scrapbook", "Calendar", "Custom"];

#[derive(Clone, Debug, PartialEq)]
pub struct FilterRow {
    pub field: String,
    pub op: String,
    pub value: String,
}

pub enum Dialog {
    Message { id: u64, title: String, text: String },
    EditPerson { id: u64, draft: Box<Person>, is_new: bool, link: Link, panel: u8, cite_on: u8 },
    EditMarriage { id: u64, draft: Box<Marriage>, notes_open: bool, sources_open: bool },
    AddOrSelect { id: u64, link: Link, sex: Sex, name: String },
    Notes { id: u64, rin: u32, text: String, find: String, with: String },
    MediaList { id: u64, rin: u32, current: usize },
    Unlink { id: u64, rin: u32 },
    ConfirmDeletePerson { id: u64, rin: u32 },
    Parents { id: u64, rin: u32 },
    OrderSpouses { id: u64, rin: u32 },
    OrderChildren { id: u64, mrin: u32 },
    SourceList { id: u64, current: Option<u32> },
    RepositoryList { id: u64, current: Option<u32> },
    EventTypes { id: u64, marriage: bool, new_name: String },
    Picker { id: u64, pick: PickFor, filter: String, soundex: bool },
    MarriageList { id: u64, filter: String },
    DescendancyList { id: u64, root: u32 },
    FindRin { id: u64, text: String, marriage: bool },
    Focus { id: u64, base: usize, start: u32, gens: u32, rows: Vec<FilterRow>, any: bool, save_name: String },
    Merge { id: u64, pairs: Vec<(u32, u32)>, at: usize, sound_alike: bool, by_afn: bool, slack: i32 },
    GlobalReplace { id: u64, field: String, find: String, with: String, whole: bool, case_sensitive: bool },
    ConfirmMixedCase { id: u64 },
    Renumber { id: u64, how: u8, preview: Option<Vec<(u32, u32, String)>> },
    WebPage { id: u64, title: String, hide_living: bool, include_notes: bool, focus_only: bool },
    Notepad { id: u64, text: String, path: Option<PathBuf> },
    Relationship { id: u64, a: Option<u32>, b: Option<u32>, result: Vec<String> },
    DateCalc { id: u64, start: String, end: String, years: String, months: String, days: String, note: String },
    Soundex { id: u64, name: String },
    Preferences { id: u64, draft: Box<Prefs>, home: Option<u32>, root: Option<u32>, tab: u8 },
    About { id: u64 },
    Keyboard { id: u64 },
    Export { id: u64, options: gedcom::ExportOptions, partial: bool },
    Properties { id: u64 },
    ConfirmRestore { id: u64, from: PathBuf },
    CheckRepair { id: u64, found: Vec<String> },
    ConfirmDeleteFile { id: u64, path: PathBuf },
    Reports { id: u64, tab: usize, start: u32, gens: u32, list_kind: usize, year: i32, include_dead: bool, fields: Vec<String>, title: String, preview: Option<Report>, page: usize },
    UnsavedClose { id: u64 },
}

impl Dialog {
    pub fn new_edit_person(id: u64, draft: Person, is_new: bool, link: Link) -> Dialog {
        Dialog::EditPerson { id, draft: Box::new(draft), is_new, link, panel: 0, cite_on: 0 }
    }
    pub fn new_picker(id: u64, pick: PickFor) -> Dialog {
        Dialog::Picker { id, pick, filter: String::new(), soundex: false }
    }
    pub fn new_focus(id: u64, start: u32) -> Dialog {
        Dialog::Focus { id, base: 0, start, gens: 99, rows: vec![], any: false, save_name: String::new() }
    }
    pub fn new_merge(id: u64) -> Dialog {
        Dialog::Merge { id, pairs: vec![], at: 0, sound_alike: true, by_afn: false, slack: 10 }
    }
    pub fn new_date_calc(id: u64) -> Dialog {
        Dialog::DateCalc { id, start: String::new(), end: String::new(), years: String::new(), months: String::new(), days: String::new(), note: String::new() }
    }
    pub fn new_reports(id: u64, start: u32) -> Dialog {
        Dialog::Reports {
            id,
            tab: 0,
            start,
            gens: 5,
            list_kind: 0,
            year: calc::today().0,
            include_dead: false,
            fields: vec!["RIN".into(), "Full Name".into(), "Birth Date".into(), "Death Date".into()],
            title: String::new(),
            preview: None,
            page: 0,
        }
    }
}

pub fn window(title: &str, id: u64) -> egui::Window<'static> {
    egui::Window::new(tr(title).to_string()).id(egui::Id::unique(("dialog", id))).collapsible(false).pivot(Align2::CENTER_CENTER).default_pos(egui::pos2(512.0, 370.0))
}

pub fn text_row(ui: &mut egui::Ui, label: &str, value: &mut String, width: f32) {
    ui.label(tr(label));
    ui.add(TextEdit::singleline(value).desired_width(width));
    ui.end_row();
}

pub fn pick_combo(ui: &mut egui::Ui, id: impl egui::AsIdSalt, value: &mut String, options: &[&str], width: f32) {
    egui::ComboBox::from_id_salt(id).selected_text(tr(value.as_str()).to_string()).width(width).show_ui(ui, |ui| {
        for o in options {
            ui.selectable_value(value, o.to_string(), tr(o));
        }
    });
}

/// Citations list used on people, marriages and events.
pub fn citations_ui(ui: &mut egui::Ui, db: &Db, cites: &mut Vec<Citation>, salt: &str) {
    let mut remove = None;
    if db.sources.is_empty() {
        ui.label(tr("There are no sources in this file yet. Add them from Edit, Source List."));
    }
    for (i, c) in cites.iter_mut().enumerate() {
        ui.push_id((salt, i), |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let title = db.sources.get(&c.source).map(|s| s.title.clone()).unwrap_or_else(|| "(Select a source)".into());
                    egui::ComboBox::from_id_salt("src").selected_text(title).width(300.0).show_ui(ui, |ui| {
                        for s in db.sources.values() {
                            ui.selectable_value(&mut c.source, s.id, &s.title);
                        }
                    });
                    if ui.button(tr("Remove")).clicked() {
                        remove = Some(i);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Film/Volume/Page:"));
                    ui.add(TextEdit::singleline(&mut c.page).desired_width(280.0));
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Actual text:"));
                    ui.add(TextEdit::singleline(&mut c.actual_text).desired_width(320.0));
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Comments:"));
                    ui.add(TextEdit::singleline(&mut c.comments).desired_width(330.0));
                });
            });
        });
    }
    if let Some(i) = remove {
        cites.remove(i);
    }
    if !db.sources.is_empty() && ui.button(tr("Link a source")).clicked() {
        let first = db.sources.keys().next().copied().unwrap_or(0);
        cites.push(Citation { source: first, ..Default::default() });
    }
}

pub fn custom_ui(ui: &mut egui::Ui, list: &mut Vec<CustomEvent>, kinds: &[String], salt: &str) {
    let mut remove = None;
    egui::Grid::new((salt, "custom")).num_columns(5).spacing(vec2(6.0, 4.0)).show(ui, |ui| {
        for (i, c) in list.iter_mut().enumerate() {
            ui.push_id((salt, i), |ui| {
                egui::ComboBox::from_id_salt("kind").selected_text(tr(c.kind.as_str()).to_string()).width(130.0).show_ui(ui, |ui| {
                    for k in kinds {
                        ui.selectable_value(&mut c.kind, k.clone(), k);
                    }
                });
            });
            ui.add(TextEdit::singleline(&mut c.description).desired_width(120.0).hint_text(tr("Description")));
            ui.add(TextEdit::singleline(&mut c.date).desired_width(90.0).hint_text(tr("Date")));
            ui.add(TextEdit::singleline(&mut c.place).desired_width(150.0).hint_text(tr("Place")));
            if ui.small_button(tr("Remove")).clicked() {
                remove = Some(i);
            }
            ui.end_row();
        }
    });
    if let Some(i) = remove {
        list.remove(i);
    }
    if ui.button(tr("New Event/Attribute")).clicked() {
        list.push(CustomEvent { kind: kinds.first().cloned().unwrap_or_else(|| "Event".into()), ..Default::default() });
    }
}

pub fn date_warning(ui: &mut egui::Ui, text: &str, day_first: bool) {
    if !calc::is_standard_date(text, day_first) {
        ui.label(RichText::new(tr("The date you typed is not standard.")).color(crate::look::pal().warn).size(11.0));
    }
}

pub fn merge_duplicate_sources(db: &mut Db) -> usize {
    let mut map: Vec<(u32, u32)> = Vec::new();
    let list: Vec<Source> = db.sources.values().cloned().collect();
    for (i, a) in list.iter().enumerate() {
        if map.iter().any(|(old, _)| *old == a.id) {
            continue;
        }
        for b in &list[i + 1..] {
            if a.title.trim().eq_ignore_ascii_case(b.title.trim()) && a.author.trim().eq_ignore_ascii_case(b.author.trim()) && a.publication.trim().eq_ignore_ascii_case(b.publication.trim()) {
                map.push((b.id, a.id));
            }
        }
    }
    let swap = |list: &mut Vec<Citation>, map: &[(u32, u32)]| {
        for c in list.iter_mut() {
            if let Some((_, new)) = map.iter().find(|(old, _)| *old == c.source) {
                c.source = *new;
            }
        }
        let mut seen: Vec<Citation> = Vec::new();
        list.retain(|c| {
            if seen.contains(c) {
                false
            } else {
                seen.push(c.clone());
                true
            }
        });
    };
    for p in db.people.values_mut() {
        swap(&mut p.citations, &map);
        for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
            swap(&mut e.citations, &map);
        }
    }
    for m in db.marriages.values_mut() {
        swap(&mut m.citations, &map);
        swap(&mut m.event.citations, &map);
    }
    for (old, _) in &map {
        db.sources.remove(old);
    }
    map.len()
}

pub fn remove_source_citations(db: &mut Db, source: u32) {
    let clean = |l: &mut Vec<Citation>| l.retain(|c| c.source != source);
    for p in db.people.values_mut() {
        clean(&mut p.citations);
        for e in [&mut p.birth, &mut p.chr, &mut p.death, &mut p.burial] {
            clean(&mut e.citations);
        }
    }
    for m in db.marriages.values_mut() {
        clean(&mut m.citations);
        clean(&mut m.event.citations);
    }
}

pub fn filter_people(db: &Db, base: usize, start: u32, gens: u32, rows: &[FilterRow], any: bool) -> Vec<u32> {
    let mut who: Vec<u32> = match base {
        1 => calc::ancestors(db, start).into_iter().filter(|x| x.1 < gens).map(|x| x.0).collect(),
        2 => calc::descendants(db, start).into_iter().filter(|x| x.1 < gens).map(|x| x.0).collect(),
        _ => db.people.keys().copied().collect(),
    };
    if base != 0 && db.people.contains_key(&start) {
        who.insert(0, start);
    }
    if rows.is_empty() {
        return who;
    }
    let df = db.prefs.day_first_numeric;
    who.retain(|r| {
        let Some(p) = db.person(*r) else { return false };
        let test = |f: &FilterRow| {
            let v = reports::field_value(db, p, &f.field);
            let want = f.value.trim();
            match f.op.as_str() {
                "contains" => v.to_lowercase().contains(&want.to_lowercase()),
                "equals" => v.trim().eq_ignore_ascii_case(want),
                "is empty" => v.trim().is_empty(),
                "is not empty" => !v.trim().is_empty(),
                "is before" | "is after" => match (calc::date_value(&v, df), calc::date_value(want, df)) {
                    (Some(a), Some(b)) => {
                        if f.op == "is before" {
                            a < b
                        } else {
                            a > b
                        }
                    }
                    _ => false,
                },
                _ => false,
            }
        };
        if any { rows.iter().any(test) } else { rows.iter().all(test) }
    });
    who
}

pub fn parse_ymd(text: &str, day_first: bool) -> Option<(i32, u32, u32)> {
    let d = calc::parse_date(text, day_first)?;
    if d.month == 0 || d.day == 0 || d.day > calc::days_in_month(d.year, d.month) {
        return None;
    }
    Some((d.year, d.month, d.day))
}

impl App {
    pub fn show_dialogs(&mut self, ctx: &egui::Context) {
        let list = std::mem::take(&mut self.dialogs);
        let mut keep = Vec::new();
        let top = list.len();
        for (i, mut d) in list.into_iter().enumerate() {
            let is_top = i + 1 == top;
            if self.show_one(ctx, &mut d, is_top) {
                keep.push(d);
            }
        }
        // Windows opened while drawing go on top of those still open.
        keep.append(&mut self.dialogs);
        self.dialogs = keep;
        // A pick goes to its window once all the windows are back in the list.
        for (pick, r) in std::mem::take(&mut self.picks) {
            self.picked(pick, r);
        }
    }

    /// Draws one window. Returns false when it should close.
    fn show_one(&mut self, ctx: &egui::Context, dlg: &mut Dialog, is_top: bool) -> bool {
        match dlg {
            Dialog::Message { .. }
            | Dialog::EditPerson { .. }
            | Dialog::EditMarriage { .. }
            | Dialog::AddOrSelect { .. }
            | Dialog::Notes { .. }
            | Dialog::MediaList { .. }
            | Dialog::Unlink { .. }
            | Dialog::ConfirmDeletePerson { .. }
            | Dialog::Parents { .. }
            | Dialog::OrderSpouses { .. }
            | Dialog::OrderChildren { .. } => self.show_people_dialog(ctx, dlg, is_top),
            Dialog::SourceList { .. }
            | Dialog::RepositoryList { .. }
            | Dialog::EventTypes { .. }
            | Dialog::Picker { .. }
            | Dialog::MarriageList { .. }
            | Dialog::DescendancyList { .. }
            | Dialog::FindRin { .. }
            | Dialog::Focus { .. }
            | Dialog::Merge { .. }
            | Dialog::GlobalReplace { .. }
            | Dialog::ConfirmMixedCase { .. } => self.show_find_dialog(ctx, dlg, is_top),
            _ => self.show_tool_dialog(ctx, dlg, is_top),
        }
    }

    fn picked(&mut self, pick: PickFor, r: u32) {
        match pick {
            PickFor::GoTo => {
                self.to_primary(r);
                if self.view == View::Individual {
                    self.selected = Some(r);
                }
            }
            PickFor::Link(link) => self.apply_link(link, r),
            PickFor::RelationA(d) | PickFor::RelationB(d) | PickFor::ReportStart(d) | PickFor::FocusStart(d) | PickFor::Root(d) | PickFor::Home(d) => {
                for dlg in self.dialogs.iter_mut() {
                    match (dlg, pick) {
                        (Dialog::Relationship { id, a, result, .. }, PickFor::RelationA(_)) if *id == d => {
                            *a = Some(r);
                            result.clear();
                        }
                        (Dialog::Relationship { id, b, result, .. }, PickFor::RelationB(_)) if *id == d => {
                            *b = Some(r);
                            result.clear();
                        }
                        (Dialog::Reports { id, start, preview, .. }, PickFor::ReportStart(_)) if *id == d => {
                            *start = r;
                            *preview = None;
                        }
                        (Dialog::Focus { id, start, .. }, PickFor::FocusStart(_)) if *id == d => *start = r,
                        (Dialog::Preferences { id, root, .. }, PickFor::Root(_)) if *id == d => *root = Some(r),
                        (Dialog::Preferences { id, home, .. }, PickFor::Home(_)) if *id == d => *home = Some(r),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn texture_for(&mut self, ctx: &egui::Context, file: &str) -> Option<egui::TextureHandle> {
        if let Some(t) = self.textures.get(file) {
            return t.clone();
        }
        let tex = image::open(file).ok().map(|img| {
            let rgba = img.thumbnail(1024, 1024).to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            ctx.load_texture(file, egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()), Default::default())
        });
        self.textures.insert(file.to_string(), tex.clone());
        tex
    }
}

pub fn order_list<T: Copy>(ui: &mut egui::Ui, list: &mut Vec<T>, label: impl Fn(&T) -> String) -> bool {
    let mut swap = None;
    let n = list.len();
    for (i, x) in list.iter().enumerate() {
        ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(i > 0, egui::Button::new(tr("Up"))).clicked() {
                    swap = Some((i, i - 1));
                }
                if ui.add_enabled(i + 1 < n, egui::Button::new(tr("Down"))).clicked() {
                    swap = Some((i, i + 1));
                }
                ui.label(format!("{}. {}", i + 1, label(x)));
            });
        });
    }
    if let Some((a, b)) = swap {
        list.swap(a, b);
        return true;
    }
    false
}

pub fn person_card(ui: &mut egui::Ui, db: &Db, rin: u32) {
    let Some(p) = db.person(rin) else { return };
    ui.group(|ui| {
        ui.label(RichText::new(plain_name(&p.name)).strong());
        ui.label(format!("RIN {}   Sex {}", p.rin, p.sex.letter()));
        for (l, e) in [("Born", &p.birth), ("Christened", &p.chr), ("Died", &p.death), ("Buried", &p.burial)] {
            ui.label(format!("{l}: {} {}", e.date, e.place));
        }
        let parents = db.primary_parents(rin).map(|m| format!("{} and {}", m.husband.map(|r| db.name(r)).unwrap_or_default(), m.wife.map(|r| db.name(r)).unwrap_or_default()));
        ui.label(format!("Parents: {}", parents.unwrap_or_default()));
        for m in &p.spouses {
            if let Some(s) = db.spouse_in(*m, rin) {
                ui.label(format!("Spouse: {}", db.name(s)));
            }
        }
        if !p.afn.is_empty() {
            ui.label(format!("AFN: {}", p.afn));
        }
    });
}

pub fn save_report(r: &Report, print: bool) -> Result<(), String> {
    let pdf = reports::to_pdf(r);
    let name: String = r.title.chars().map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' }).collect();
    let target = if print {
        Some(std::env::temp_dir().join(format!("{name}.pdf")))
    } else {
        rfd::FileDialog::new().add_filter("PDF", &["pdf"]).set_file_name(format!("{name}.pdf")).save_file()
    };
    let Some(p) = target else { return Ok(()) };
    std::fs::write(&p, pdf).map_err(|e| e.to_string())?;
    if print {
        crate::app::open_with_system(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn draw_page(ui: &mut egui::Ui, page: &reports::Page, scale: f32) {
    let size = vec2(reports::PAGE_W * scale, reports::PAGE_H * scale);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = ui.painter();
    p.rect(rect, 0.0, Color32::WHITE, Stroke::new(1.0, pal().shadow), egui::StrokeKind::Inside);
    let at = |x: f32, y: f32| rect.min + vec2(x * scale, y * scale);
    for op in &page.ops {
        match op {
            Op::Text { x, y, size, bold, text } => {
                let font = if *bold { egui::FontId::new(*size * scale, egui::FontFamily::Name("bold".into())) } else { egui::FontId::proportional(*size * scale) };
                p.text(at(*x, *y), Align2::LEFT_TOP, text, font, Color32::BLACK);
            }
            Op::Line { x1, y1, x2, y2 } => {
                p.line_segment([at(*x1, *y1), at(*x2, *y2)], Stroke::new(0.5, Color32::BLACK));
            }
            Op::Rect { x, y, w, h, shade } => {
                let r = egui::Rect::from_min_size(at(*x, *y), vec2(*w * scale, *h * scale));
                let fill = if *shade { Color32::from_gray(230) } else { Color32::WHITE };
                p.rect(r, 0.0, fill, Stroke::new(0.5, Color32::BLACK), egui::StrokeKind::Inside);
            }
        }
    }
}
