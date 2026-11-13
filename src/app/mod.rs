//! The main window: its state, the commands it understands, and the frame-by-frame loop.

pub mod colors;
mod commands;
mod files;
mod icons;
mod menus;
mod navigate;
mod screenshot;
pub mod script;
mod style;

pub use files::*;
pub use icons::*;
pub use style::*;

use crate::i18n::tr;
use crate::dialogs::*;
use crate::data::*;
use crate::screens::IndividualState;
use egui::Stroke;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

pub const APP_NAME: &str = "PAF Classic";

pub const FILE_EXT: &str = "pafc";

// The colours come from the scheme chosen under View > Colors; Classic is the original's.
pub use colors::pal;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    Family,
    Pedigree,
    Individual,
}

/// Which place on the screen is highlighted. It can be an empty box.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    Primary,
    Marriage,
    Spouse,
    Father,
    Mother,
    Child(usize),
    /// Pedigree position: 1 is the primary person, 2 and 3 their parents, and so on.
    Tree(usize),
    Row,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cmd {
    New,
    Open,
    OpenPath(PathBuf),
    Close,
    SaveAs,
    Import,
    Export,
    Properties,
    Backup,
    Restore,
    CheckRepair,
    DeleteFile,
    Reports,
    Exit,
    Edit,
    Notes,
    Media,
    Unlink,
    DeletePerson,
    EditMarriage,
    EditParents,
    ParentsMarriage,
    OrderSpouses,
    OrderChildren,
    SourceList,
    RepositoryList,
    PersonEvents,
    MarriageEvents,
    AddIndividual,
    AddFamily,
    AddSpouse,
    AddChild,
    AddFather,
    AddMother,
    AddOtherParents,
    IndividualList,
    MarriageList,
    DescendancyList,
    FindByRin,
    Focus,
    Home,
    WebSearch(&'static str),
    MatchMerge,
    MergeSources,
    GlobalReplace,
    MixedCase,
    Renumber,
    SetHome,
    WebPage,
    Notepad,
    Relationship,
    DateCalc,
    Soundex,
    Preferences,
    About,
    Keyboard,
    Sample,
    ToPrimary(u32),
    #[allow(dead_code)]
    Select(u32),
    SetView(View),
    ShowMarriage(usize),
    ShowParents(usize),
    SelectSlot(Slot, Option<u32>),
    GettingStarted,
    GoFather,
    GoMother,
    NextChild,
}

pub struct App {
    pub db: Db,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub view: View,
    pub primary: Option<u32>,
    pub selected: Option<u32>,
    pub slot: Slot,
    pub individual: IndividualState,
    /// Which spouse and which parents are showing for each person.
    pub shown_marriage: BTreeMap<u32, usize>,
    pub shown_parents: BTreeMap<u32, usize>,
    /// The last child moved to primary from each parent, for the C key.
    pub last_child: BTreeMap<u32, u32>,
    pub dialogs: Vec<Dialog>,
    pub picks: Vec<(PickFor, u32)>,
    pub next_id: u64,
    pub pending: Vec<Cmd>,
    pub status: String,
    pub recent: Vec<PathBuf>,
    pub focus: Option<Vec<u32>>,
    pub show_toolbar: bool,
    pub show_status: bool,
    pub confirm_exit: bool,
    pub really_exit: bool,
    pub textures: BTreeMap<String, Option<egui::TextureHandle>>,
    last_title: String,
    screenshot: Option<PathBuf>,
    frames: u32,
    screenshot_requested: bool,
    shot_done: Arc<std::sync::atomic::AtomicBool>,
    steps: Vec<(u32, crate::app::script::Step)>,
    input_frames: u32,
}

impl App {
    /// `view` and `open` start on a given screen or window, for trying each one by hand.
    pub fn new(screenshot: Option<PathBuf>, sample: bool, file: Option<PathBuf>, view: Option<String>, open: Option<String>, steps: Option<String>) -> Self {
        let recent = std::fs::read(recent_file()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        let mut app = App {
            db: Db::default(),
            path: None,
            dirty: false,
            view: View::Family,
            primary: None,
            selected: None,
            slot: Slot::Primary,
            individual: IndividualState::default(),
            shown_marriage: BTreeMap::new(),
            shown_parents: BTreeMap::new(),
            last_child: BTreeMap::new(),
            dialogs: Vec::new(),
            picks: Vec::new(),
            next_id: 1,
            pending: Vec::new(),
            status: String::new(),
            recent,
            focus: None,
            show_toolbar: true,
            show_status: true,
            confirm_exit: false,
            really_exit: false,
            textures: BTreeMap::new(),
            last_title: String::new(),
            screenshot,
            frames: 0,
            screenshot_requested: false,
            shot_done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            steps: steps.as_deref().map(crate::app::script::parse).unwrap_or_default(),
            input_frames: 0,
        };
        if sample {
            app.load_db(sample_family(), None);
        }
        if let Some(f) = file {
            app.pending.push(Cmd::OpenPath(f));
        } else if app.screenshot.is_none() && !sample {
            // The original reopens the file in use when it last closed. A picture run always
            // starts on the blank family so it stays comparable.
            if let Some(last) = app.recent.first().filter(|p| p.exists()).cloned() {
                app.pending.push(Cmd::OpenPath(last));
            }
        }
        match view.as_deref() {
            Some("pedigree") => app.view = View::Pedigree,
            Some("individual") => app.view = View::Individual,
            _ => {}
        }
        app.slot = app.home_slot();
        let start = match open.as_deref() {
            Some("edit") => Some(Cmd::Edit),
            Some("marriage") => Some(Cmd::EditMarriage),
            Some("parents") => Some(Cmd::EditParents),
            Some("add") => Some(Cmd::AddChild),
            Some("list") => Some(Cmd::IndividualList),
            Some("marriages") => Some(Cmd::MarriageList),
            Some("descendants") => Some(Cmd::DescendancyList),
            Some("reports") => Some(Cmd::Reports),
            Some("prefs") => Some(Cmd::Preferences),
            Some("relation") => Some(Cmd::Relationship),
            Some("date") => Some(Cmd::DateCalc),
            Some("export") => Some(Cmd::Export),
            Some("merge") => Some(Cmd::MatchMerge),
            Some("sources") => Some(Cmd::SourceList),
            _ => None,
        };
        app.pending.extend(start);
        app
    }

    fn home_slot(&self) -> Slot {
        match self.view {
            View::Family => Slot::Primary,
            View::Pedigree => Slot::Tree(1),
            View::Individual => Slot::Row,
        }
    }

    pub fn dialog_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    pub fn push(&mut self, make: impl FnOnce(u64) -> Dialog) {
        let id = self.dialog_id();
        self.dialogs.push(make(id));
    }

    pub fn message(&mut self, title: &str, text: impl Into<String>) {
        let text: String = text.into();
        let (title, text) = (tr(title).to_string(), tr(&text).to_string());
        self.push(|id| Dialog::Message { id, title, text });
    }

    pub fn load_db(&mut self, db: Db, path: Option<PathBuf>) {
        self.primary = db.home.or_else(|| db.people.keys().next().copied());
        self.selected = self.primary;
        self.slot = self.home_slot();
        self.db = db;
        self.path = path;
        self.dirty = false;
        self.shown_marriage.clear();
        self.shown_parents.clear();
        self.last_child.clear();
        self.focus = None;
        self.dialogs.clear();
    }

    /// Saves after each change, as the original writes each change to its file straight away.
    pub fn touch(&mut self) {
        self.dirty = true;
        self.db.prefs.changed_since_backup = true;
        if self.path.is_some() {
            self.save_now();
        }
        if self.primary.is_none_or(|r| !self.db.people.contains_key(&r)) {
            self.primary = self.db.home.filter(|r| self.db.people.contains_key(r)).or_else(|| self.db.people.keys().next().copied());
        }
        if self.selected.is_none_or(|r| !self.db.people.contains_key(&r)) {
            self.selected = self.primary;
        }
    }

    pub fn log(&mut self, what: &str, id: u32) {
        if !self.db.prefs.log_changes {
            return;
        }
        let t = crate::rules::today();
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        self.db.log.push(format!("{}-{}-{} {:02}:{:02} {what} {id}", t.1, t.2, t.0 % 100, (secs / 3600) % 24, (secs / 60) % 60));
    }

    /// After Renumber Records: follow the people on screen to their new numbers and forget
    /// what was remembered under the old ones.
    pub fn after_renumber(&mut self, rins: &std::collections::BTreeMap<u32, u32>) {
        let map = |r: Option<u32>| r.and_then(|r| rins.get(&r).copied());
        self.primary = map(self.primary).or(self.db.home);
        self.selected = map(self.selected).or(self.primary);
        self.shown_marriage.clear();
        self.shown_parents.clear();
        self.last_child.clear();
        self.focus = None;
        self.slot = self.home_slot();
        self.touch();
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        if !self.steps.is_empty() {
            self.input_frames += 1;
            crate::app::script::feed(&self.steps, self.input_frames, raw);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.screenshot_step(&ctx);
        self.update_title(&ctx);
        self.exit_prompt(&ctx);
        self.shortcuts(&ctx);
        // Strip heights measured from the person's pictures, with their title bar left out.
        let panel = egui::Frame::new().fill(pal().bg);
        egui::Panel::top("menu").exact_size(20.0).frame(panel).show_separator_line(false).show(ui, |ui| self.menus(ui));
        if self.show_toolbar {
            egui::Panel::top("tools").exact_size(30.0).frame(panel).show_separator_line(false).show(ui, |ui| {
                let r = ui.max_rect();
                ui.painter().hline(r.left()..=r.right(), r.top(), Stroke::new(1.0, pal().shadow));
                ui.painter().hline(r.left()..=r.right(), r.bottom() - 1.0, Stroke::new(1.0, pal().shadow));
                self.toolbar(ui);
            });
        }
        egui::Panel::top("tabs").exact_size(24.0).frame(panel).show_separator_line(false).show(ui, |ui| self.tabs(ui));
        if self.show_status {
            egui::Panel::bottom("status").exact_size(22.0).frame(panel).show_separator_line(false).show(ui, |ui| self.status_bar(ui));
        }
        egui::CentralPanel::default().frame(panel).show(ui, |ui| match self.view {
            View::Family => self.family_view(ui),
            View::Pedigree => self.pedigree_view(ui),
            View::Individual => self.individual_view(ui),
        });
        self.show_dialogs(&ctx);
        let list = std::mem::take(&mut self.pending);
        for c in list {
            self.status.clear();
            self.run(c);
        }
    }
}
