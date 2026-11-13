//! The windows that open over the main screen: shared types and the window loop.

mod actions;
mod find;
mod people;
mod tools;
mod widgets;

pub use actions::*;
pub use widgets::*;

use crate::app::*;
use crate::rules;
use crate::formats::gedcom;
use crate::data::*;
use crate::reports::Report;
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
            year: rules::today().0,
            include_dead: false,
            fields: vec!["RIN".into(), "Full Name".into(), "Birth Date".into(), "Death Date".into()],
            title: String::new(),
            preview: None,
            page: 0,
        }
    }
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
