//! The main window: state, menus, toolbar, file handling and commands.
use crate::i18n::tr;
use crate::dialogs::*;
use crate::gedcom;
use crate::model::*;
use crate::paffile;
use crate::views::IndividualState;
use egui::{Color32, FontData, FontDefinitions, FontFamily, Key, Modifiers, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const APP_NAME: &str = "PAF Classic";
pub const FILE_EXT: &str = "pafc";

// The colours come from the scheme chosen under View > Colors; Classic is the original's.
pub use crate::look::pal;

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
    steps: Vec<(u32, crate::script::Step)>,
    input_frames: u32,
}

// System fonts to try on each platform, in order of preference.
#[cfg(target_os = "macos")]
const PLAIN_FONTS: &[&str] = &["/System/Library/Fonts/Supplemental/Arial.ttf", "/Library/Fonts/Arial.ttf"];
#[cfg(target_os = "macos")]
const BOLD_FONTS: &[&str] = &["/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/Library/Fonts/Arial Bold.ttf"];
#[cfg(target_os = "macos")]
const WIDE_FONTS: &[&str] = &["/System/Library/Fonts/Supplemental/Arial Unicode.ttf", "/Library/Fonts/Arial Unicode.ttf"];

#[cfg(target_os = "windows")]
const PLAIN_FONTS: &[&str] = &["C:\\Windows\\Fonts\\arial.ttf", "C:\\Windows\\Fonts\\segoeui.ttf"];
#[cfg(target_os = "windows")]
const BOLD_FONTS: &[&str] = &["C:\\Windows\\Fonts\\arialbd.ttf", "C:\\Windows\\Fonts\\segoeuib.ttf"];
#[cfg(target_os = "windows")]
const WIDE_FONTS: &[&str] = &[
    "C:\\Windows\\Fonts\\ARIALUNI.TTF",
    "C:\\Windows\\Fonts\\msjh.ttc",
    "C:\\Windows\\Fonts\\YuGothM.ttc",
    "C:\\Windows\\Fonts\\msgothic.ttc",
    "C:\\Windows\\Fonts\\malgun.ttf",
    "C:\\Windows\\Fonts\\msyh.ttc",
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const PLAIN_FONTS: &[&str] = &[
    "/usr/share/fonts/truetype/msttcorefonts/Arial.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
    "/usr/share/fonts/TTF/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
];
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const BOLD_FONTS: &[&str] = &[
    "/usr/share/fonts/truetype/msttcorefonts/Arial_Bold.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/liberation-sans/LiberationSans-Bold.ttf",
    "/usr/share/fonts/TTF/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
];
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const WIDE_FONTS: &[&str] = &[
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

/// Adds the first of these fonts that exists on this computer.
fn add_first_font(fonts: &mut FontDefinitions, name: &str, paths: &[&str], first: bool) -> bool {
    paths.iter().any(|path| add_font(fonts, name, path, first))
}

/// Opens a file or web address with whatever this computer uses for it.
pub fn open_with_system(target: impl AsRef<std::ffi::OsStr>) -> std::io::Result<()> {
    let target = target.as_ref();
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("rundll32");
        c.arg("url.dll,FileProtocolHandler");
        c
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut cmd = std::process::Command::new("xdg-open");
    cmd.arg(target).spawn().map(|_| ())
}

fn add_font(fonts: &mut FontDefinitions, name: &str, path: &str, first: bool) -> bool {
    let Ok(bytes) = std::fs::read(path) else { return false };
    fonts.font_data.insert(name.into(), Arc::new(FontData::from_owned(bytes)));
    let list = fonts.families.entry(FontFamily::Proportional).or_default();
    if first {
        list.insert(0, name.into());
    } else {
        list.push(name.into());
    }
    true
}

pub fn setup_look(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    // A plain sans font like the original's, plus one that covers Cyrillic and Asian names.
    add_first_font(&mut fonts, "plain", PLAIN_FONTS, true);
    // Every wide font found is added, so names in more scripts have a font behind them.
    let mut wide = Vec::new();
    for (i, path) in WIDE_FONTS.iter().enumerate() {
        let name = format!("wide{i}");
        if add_font(&mut fonts, &name, path, false) {
            wide.push(name);
        }
    }
    if add_first_font(&mut fonts, "bold", BOLD_FONTS, false) {
        // Bold names need the same wide font behind them, or Chinese, Japanese, Korean and Cyrillic
        // names would show as empty squares in the boxes. It does the job of the original's add-on
        // font package.
        let bold_list = fonts.families.entry(FontFamily::Name("bold".into())).or_default();
        bold_list.push("bold".into());
        for name in &wide {
            bold_list.push(name.clone());
        }
    } else {
        let list = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        fonts.families.insert(FontFamily::Name("bold".into()), list);
    }
    ctx.set_fonts(fonts);
    apply_colors(ctx);
}

/// Sets the window colours from the chosen scheme. Called at start and when the scheme changes.
pub fn apply_colors(ctx: &egui::Context) {
    let p = pal();
    let mut v = if p.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.bg;
    v.window_fill = p.bg;
    v.extreme_bg_color = p.text_bg;
    v.faint_bg_color = p.faint;
    v.window_corner_radius = 2.into();
    v.menu_corner_radius = 1.into();
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = 2.into();
    }
    v.selection.bg_fill = p.select;
    v.selection.stroke = Stroke::new(1.0, p.select_ink);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.ink);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.ink);
    v.widgets.inactive.bg_fill = p.button;
    v.widgets.inactive.weak_bg_fill = p.button;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.shadow);
    v.widgets.hovered.weak_bg_fill = p.hover;
    v.widgets.active.weak_bg_fill = p.pressed;
    // Classic keeps the original's settings exactly; the other schemes set every state's colours.
    if crate::look::current() != 0 {
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.shadow);
        v.widgets.noninteractive.bg_fill = p.bg;
        v.widgets.hovered.bg_fill = p.hover;
        v.widgets.active.bg_fill = p.pressed;
        v.widgets.open.bg_fill = p.hover;
        v.widgets.open.weak_bg_fill = p.hover;
        for w in [&mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
            w.fg_stroke = Stroke::new(1.0, p.ink);
            w.bg_stroke = Stroke::new(1.0, p.light);
        }
        v.window_stroke = Stroke::new(1.0, p.shadow);
        v.hyperlink_color = p.ink;
        v.warn_fg_color = p.warn;
    }
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.set_visuals_of(egui::Theme::Light, v.clone());
    ctx.set_visuals_of(egui::Theme::Dark, v);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = vec2(6.0, 4.0);
        s.spacing.button_padding = vec2(7.0, 2.0);
    });
}

/// The app's icon: the white tree on blue, from assets/icon.png.
pub fn app_icon() -> egui::IconData {
    const PNG: &[u8] = include_bytes!("../assets/icon.png");
    match image::load_from_memory(PNG) {
        Ok(img) => {
            let img = img.to_rgba8();
            let (width, height) = img.dimensions();
            egui::IconData { rgba: img.into_raw(), width, height }
        }
        Err(_) => egui::IconData::default(),
    }
}

/// Runs a file reader so that a fault inside it becomes a message rather than closing the app.
pub fn guarded<T>(read: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(read)).unwrap_or_else(|_| {
        Err("This file could not be read. It may use a layout this program doesn't handle yet.".into())
    })
}

/// Switches language straight away and remembers it for next time. It belongs to the program,
/// not the family file.
pub fn choose_language(index: usize) {
    crate::i18n::set(index);
    // A picture run is only a test, so it leaves the person's setting alone.
    if std::env::args().any(|a| a == "--screenshot") {
        return;
    }
    let _ = std::fs::create_dir_all(config_dir());
    let _ = std::fs::write(config_dir().join("language.txt"), crate::i18n::code());
}

/// Switches to a colour scheme and remembers it for next time.
pub fn choose_colors(ctx: &egui::Context, index: usize) {
    crate::look::set(index);
    apply_colors(ctx);
    // A picture run is only a test, so it leaves the person's setting alone.
    if std::env::args().any(|a| a == "--screenshot") {
        return;
    }
    let _ = std::fs::create_dir_all(config_dir());
    let _ = std::fs::write(config_dir().join("colors.txt"), crate::look::name());
}

pub fn config_dir() -> PathBuf {
    #[allow(unused_variables)]
    let home = || std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    #[cfg(target_os = "macos")]
    let base = home().join("Library/Application Support");
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(std::env::temp_dir)
    });
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config"));
    base.join(APP_NAME)
}

fn recent_file() -> PathBuf {
    config_dir().join("recent.json")
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
            steps: steps.as_deref().map(crate::script::parse).unwrap_or_default(),
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
        let t = crate::calc::today();
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

    /// A copy of the family file as it is now, saved beside it, before a big change.
    pub fn safety_copy(&mut self, why: &str) -> Option<PathBuf> {
        let path = self.path.clone()?;
        let stem = path.file_stem()?.to_str()?.to_string();
        let t = crate::calc::today();
        let copy = path.with_file_name(format!("{stem} before {why} {:04}-{:02}-{:02}.{FILE_EXT}", t.0, t.1, t.2));
        let bytes = serde_json::to_vec_pretty(&self.db).ok()?;
        std::fs::write(&copy, bytes).ok()?;
        Some(copy)
    }

    fn save_now(&mut self) -> bool {
        let Some(path) = self.path.clone() else { return false };
        match serde_json::to_vec_pretty(&self.db) {
            Ok(bytes) => {
                // Write beside the file, then swap, so a crash cannot leave half a file.
                let tmp = path.with_extension("pafc.saving");
                if let Err(e) = std::fs::write(&tmp, bytes).and_then(|_| std::fs::rename(&tmp, &path)) {
                    self.message("Could not save", format!("Could not save {}:\n{e}", path.display()));
                    return false;
                }
                self.dirty = false;
                self.remember_recent(&path);
                true
            }
            Err(e) => {
                self.message("Could not save", e.to_string());
                false
            }
        }
    }

    fn remember_recent(&mut self, path: &Path) {
        // The automated tests save scratch files, which must never join the person's list.
        if cfg!(test) || self.recent.first().map(|p| p == path).unwrap_or(false) {
            return;
        }
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_path_buf());
        self.recent.truncate(4);
        let _ = std::fs::create_dir_all(config_dir());
        if let Ok(b) = serde_json::to_vec(&self.recent) {
            let _ = std::fs::write(recent_file(), b);
        }
    }

    fn pick_family_filter() -> rfd::FileDialog {
        rfd::FileDialog::new()
            .add_filter("Family files", &[FILE_EXT, "paf", "ged"])
            .add_filter("All files", &["*"])
    }

    pub fn open_path(&mut self, path: PathBuf) {
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.message("Could not open", format!("Could not open {}:\n{e}", path.display()));
                return;
            }
        };
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "paf" => match guarded(|| paffile::import(&bytes)) {
                Ok(got) => {
                    self.load_db(got.db, None);
                    self.dirty = true;
                    self.message("Family file read", format!("{}\n\nUse File, Save As to keep this as a {APP_NAME} family file.", got.summary));
                }
                Err(e) => self.message("Could not read this .paf file", e),
            },
            "ged" => self.import_gedcom(&bytes),
            _ => match serde_json::from_slice::<Db>(&bytes) {
                Ok(db) => {
                    self.load_db(db, Some(path.clone()));
                    self.remember_recent(&path);
                }
                Err(_) => {
                    if paffile::identify(&bytes) != paffile::Kind::Other {
                        match guarded(|| paffile::import(&bytes)) {
                            Ok(got) => {
                                self.load_db(got.db, None);
                                self.dirty = true;
                                self.message("Family file read", got.summary);
                            }
                            Err(e) => self.message("Could not read this .paf file", e),
                        }
                    } else if bytes.starts_with(b"0 HEAD") || bytes.windows(6).take(16).any(|w| w == b"0 HEAD") {
                        self.import_gedcom(&bytes);
                    } else {
                        self.message("Could not open", "This file is not a valid family file. It may be corrupt.");
                    }
                }
            },
        }
    }

    fn import_gedcom(&mut self, bytes: &[u8]) {
        match guarded(|| gedcom::import(bytes)) {
            Ok((db, report)) => {
                self.load_db(db, None);
                self.dirty = true;
                self.message("GEDCOM import", format!("{}\n\nUse File, Save As to keep this as a {APP_NAME} family file.", report.summary()));
            }
            Err(e) => self.message("GEDCOM import", e),
        }
    }

    /// The guide's way to combine files: Import adds to the open family, and an empty family
    /// simply fills up. The records get the next free numbers, so nothing already here changes.
    fn import_into_current(&mut self, path: &Path) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                self.message("Could not open", format!("Could not open {}:\n{e}", path.display()));
                return;
            }
        };
        let is_paf = path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("paf"));
        let got = guarded(|| if is_paf { paffile::import(&bytes).map(|g| (g.db, g.summary)) } else { gedcom::import(&bytes).map(|(d, r)| (d, r.summary())) });
        match got {
            Ok((db, summary)) => {
                self.db.absorb(db);
                self.touch();
                self.selected = self.primary;
                self.message("Import", summary);
            }
            Err(e) => self.message("Import", e),
        }
    }

    fn save_as_dialog(&mut self) {
        let mut d = rfd::FileDialog::new().add_filter("Family files", &[FILE_EXT]).set_file_name(format!("Family.{FILE_EXT}"));
        if let Some(p) = &self.path {
            if let Some(folder) = p.parent() {
                d = d.set_directory(folder);
            }
        }
        if let Some(mut p) = d.save_file() {
            if p.extension().is_none() {
                p.set_extension(FILE_EXT);
            }
            self.path = Some(p);
            self.save_now();
        }
    }

    fn write_backup(&mut self) {
        let name = self.path.as_ref().and_then(|p| p.file_stem()).and_then(|s| s.to_str()).unwrap_or("Family").to_string();
        let t = crate::calc::today();
        let file = format!("{name} backup {:04}-{:02}-{:02}.{FILE_EXT}", t.0, t.1, t.2);
        if let Some(p) = rfd::FileDialog::new().add_filter("Family files", &[FILE_EXT]).set_file_name(file).save_file() {
            match serde_json::to_vec_pretty(&self.db).map_err(|e| e.to_string()).and_then(|b| std::fs::write(&p, b).map_err(|e| e.to_string())) {
                Ok(()) => {
                    self.db.prefs.closes_since_backup = 0;
                    self.touch();
                    self.db.prefs.changed_since_backup = false;
                    if self.path.is_some() {
                        self.save_now();
                    }
                    self.message("Backup", format!("Backup saved to {}.", p.display()));
                }
                Err(e) => self.message("Backup", format!("The backup could not be saved:\n{e}")),
            }
        }
    }

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
    fn c_key(&mut self, rin: u32) {
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

    pub fn run(&mut self, cmd: Cmd) {
        let sel = self.selected.or(self.primary);
        match cmd {
            Cmd::New => {
                if let Some(mut p) = rfd::FileDialog::new().add_filter("Family files", &[FILE_EXT]).set_file_name(format!("New Family.{FILE_EXT}")).save_file() {
                    if p.extension().is_none() {
                        p.set_extension(FILE_EXT);
                    }
                    self.load_db(Db::default(), Some(p));
                    self.save_now();
                }
            }
            Cmd::Open => {
                if let Some(p) = Self::pick_family_filter().pick_file() {
                    self.open_path(p);
                }
            }
            Cmd::OpenPath(p) => self.open_path(p),
            Cmd::Close => {
                // The guide: remind only when the file changed since its last backup. 0 never reminds.
                let changed = self.db.prefs.changed_since_backup;
                if changed {
                    self.db.prefs.closes_since_backup += 1;
                }
                let freq = self.db.prefs.backup_frequency;
                let remind = changed && freq > 0 && self.db.prefs.closes_since_backup >= freq;
                if self.path.is_some() {
                    self.save_now();
                }
                let keep_unsaved = self.dirty && self.path.is_none();
                if keep_unsaved {
                    self.push(|id| Dialog::UnsavedClose { id });
                } else {
                    if remind {
                        self.message("Backup reminder", "It has been a while since this family file was backed up. Use File, Backup after opening it again.");
                    }
                    self.load_db(Db::default(), None);
                }
            }
            Cmd::SaveAs => self.save_as_dialog(),
            Cmd::Import => {
                if let Some(p) = rfd::FileDialog::new().add_filter("GEDCOM or .paf files", &["ged", "paf"]).pick_file() {
                    self.import_into_current(&p);
                }
            }
            Cmd::Export => self.push(|id| Dialog::Export { id, options: gedcom::ExportOptions::default(), partial: false }),
            Cmd::Properties => self.push(|id| Dialog::Properties { id }),
            Cmd::Backup => self.write_backup(),
            Cmd::Restore => {
                if let Some(p) = rfd::FileDialog::new().add_filter("Family files", &[FILE_EXT]).pick_file() {
                    self.push(|id| Dialog::ConfirmRestore { id, from: p });
                }
            }
            Cmd::CheckRepair => {
                let mut copy = self.db.clone();
                let found = copy.check(false);
                self.push(|id| Dialog::CheckRepair { id, found });
            }
            Cmd::DeleteFile => {
                if let Some(p) = rfd::FileDialog::new().add_filter("Family files", &[FILE_EXT]).pick_file() {
                    self.push(|id| Dialog::ConfirmDeleteFile { id, path: p });
                }
            }
            Cmd::Reports => {
                let start = sel.unwrap_or(0);
                self.push(|id| Dialog::new_reports(id, start));
            }
            Cmd::Exit => self.confirm_exit = true,
            Cmd::Edit => {
                if let Some(r) = sel {
                    self.open_edit_person(r);
                }
            }
            Cmd::Notes => {
                if let Some(r) = sel {
                    if let Some(p) = self.db.person(r) {
                        let text = p.notes.clone();
                        self.push(|id| Dialog::Notes { id, rin: r, text, find: String::new(), with: String::new() });
                    }
                }
            }
            Cmd::Media => {
                if let Some(r) = sel {
                    self.push(|id| Dialog::MediaList { id, rin: r, current: 0 });
                }
            }
            Cmd::Unlink => {
                if let Some(r) = sel {
                    self.push(|id| Dialog::Unlink { id, rin: r });
                }
            }
            Cmd::DeletePerson => {
                if let Some(r) = sel {
                    self.push(|id| Dialog::ConfirmDeletePerson { id, rin: r });
                }
            }
            Cmd::EditMarriage => {
                // Someone off in another family means their marriage, otherwise the one on screen.
                let who = if matches!(self.slot, Slot::Primary | Slot::Marriage | Slot::Spouse) { self.primary } else { sel };
                let m = who.and_then(|p| self.current_marriage(p));
                match m.and_then(|m| self.db.marriages.get(&m).cloned()) {
                    Some(draft) => self.push(|id| Dialog::EditMarriage { id, draft: Box::new(draft), notes_open: false, sources_open: false }),
                    None => self.message("Marriage", "This individual has no marriage yet. Add a spouse first, or add one from the Add menu."),
                }
            }
            Cmd::EditParents => {
                if let Some(r) = sel {
                    self.push(|id| Dialog::Parents { id, rin: r });
                }
            }
            Cmd::ParentsMarriage => {
                let m = sel.and_then(|r| self.current_parents(r));
                match m.and_then(|m| self.db.marriages.get(&m).cloned()) {
                    Some(draft) => self.push(|id| Dialog::EditMarriage { id, draft: Box::new(draft), notes_open: false, sources_open: false }),
                    None => self.message("Parents", "This individual has no parents linked yet."),
                }
            }
            Cmd::OrderSpouses => {
                if let Some(r) = self.primary {
                    self.push(|id| Dialog::OrderSpouses { id, rin: r });
                }
            }
            Cmd::OrderChildren => match self.primary.and_then(|r| self.current_marriage(r)) {
                Some(m) => self.push(|id| Dialog::OrderChildren { id, mrin: m }),
                None => self.message("Order Children", "This individual has no marriage to order children in."),
            },
            Cmd::SourceList => self.push(|id| Dialog::SourceList { id, current: None }),
            Cmd::RepositoryList => self.push(|id| Dialog::RepositoryList { id, current: None }),
            Cmd::PersonEvents => self.push(|id| Dialog::EventTypes { id, marriage: false, new_name: String::new() }),
            Cmd::MarriageEvents => self.push(|id| Dialog::EventTypes { id, marriage: true, new_name: String::new() }),
            Cmd::AddIndividual => self.start_add(Link::None),
            Cmd::AddFamily => {
                let draft = Person { sex: Sex::Male, ..Default::default() };
                self.push(|id| Dialog::new_edit_person(id, draft, true, Link::None));
                self.status = "Add the husband first, then use Add Spouse and Add Child.".into();
            }
            Cmd::AddSpouse => {
                if let Some(r) = sel {
                    self.start_add(Link::Spouse(r));
                }
            }
            Cmd::AddChild => {
                if let Some(r) = sel {
                    self.start_add(Link::Child(r));
                }
            }
            Cmd::AddFather => {
                if let Some(r) = sel {
                    self.start_add(Link::Father(r));
                }
            }
            Cmd::AddMother => {
                if let Some(r) = sel {
                    self.start_add(Link::Mother(r));
                }
            }
            Cmd::AddOtherParents => {
                if let Some(r) = sel {
                    // A new set of parents: a fresh parent marriage with this person as the child.
                    let mrin = self.db.next_mrin();
                    self.db.marriages.insert(mrin, Marriage { mrin, ..Default::default() });
                    self.db.add_child(mrin, r);
                    let idx = self.db.person(r).map(|p| p.parents.len().saturating_sub(1)).unwrap_or(0);
                    self.shown_parents.insert(r, idx);
                    self.touch();
                    self.start_add(Link::Father(r));
                }
            }
            Cmd::IndividualList => {
                let pick = PickFor::GoTo;
                self.push(|id| Dialog::new_picker(id, pick));
            }
            Cmd::MarriageList => self.push(|id| Dialog::MarriageList { id, filter: String::new() }),
            Cmd::DescendancyList => {
                if let Some(r) = sel {
                    self.push(|id| Dialog::DescendancyList { id, root: r });
                }
            }
            Cmd::FindByRin => self.push(|id| Dialog::FindRin { id, text: String::new(), marriage: false }),
            Cmd::Focus => self.push(|id| Dialog::new_focus(id, sel.unwrap_or(0))),
            Cmd::Home => {
                if let Some(h) = self.db.home {
                    self.to_primary(h);
                }
            }
            Cmd::WebSearch(site) => self.web_search(site),
            Cmd::MatchMerge => self.push(|id| Dialog::new_merge(id)),
            Cmd::MergeSources => {
                let n = merge_duplicate_sources(&mut self.db);
                self.touch();
                self.message("Merge Duplicate Sources", format!("Merged {n} duplicate sources."));
            }
            Cmd::GlobalReplace => self.push(|id| Dialog::GlobalReplace { id, field: "Places".into(), find: String::new(), with: String::new(), whole: false, case_sensitive: false }),
            Cmd::MixedCase => self.push(|id| Dialog::ConfirmMixedCase { id }),
            Cmd::Renumber => self.push(|id| Dialog::Renumber { id, how: 0, preview: None }),
            Cmd::SetHome => {
                if let Some(r) = sel.filter(|r| self.db.people.contains_key(r)) {
                    self.db.home = Some(r);
                    self.touch();
                    self.message("Home Person", format!("{} is now the home person.", self.db.name(r)));
                }
            }
            Cmd::WebPage => {
                let title = self.path.as_ref().and_then(|p| p.file_stem()).and_then(|s| s.to_str()).unwrap_or("Family History").to_string();
                self.push(|id| Dialog::WebPage { id, title, hide_living: true, include_notes: true, focus_only: false });
            }
            Cmd::Notepad => self.push(|id| Dialog::Notepad { id, text: String::new(), path: None }),
            Cmd::Relationship => {
                let a = sel;
                let b = self.db.root_person.or(self.db.home);
                self.push(|id| Dialog::Relationship { id, a, b, result: Vec::new() });
            }
            Cmd::DateCalc => self.push(|id| Dialog::new_date_calc(id)),
            Cmd::Soundex => self.push(|id| Dialog::Soundex { id, name: String::new() }),
            Cmd::Preferences => {
                let draft = self.db.prefs.clone();
                let (home, root) = (self.db.home, self.db.root_person);
                self.push(|id| Dialog::Preferences { id, draft: Box::new(draft), home, root, tab: 0 });
            }
            Cmd::About => self.push(|id| Dialog::About { id }),
            Cmd::Keyboard => self.push(|id| Dialog::Keyboard { id }),
            Cmd::Sample => self.load_db(sample_family(), None),
            Cmd::ToPrimary(r) => self.to_primary(r),
            Cmd::Select(r) => self.selected = Some(r),
            Cmd::SetView(v) => {
                // Switching view makes the selected individual primary, as the guide says.
                if v != View::Individual {
                    if let Some(s) = self.selected {
                        self.to_primary(s);
                    }
                }
                self.view = v;
                self.slot = self.home_slot();
            }
            Cmd::ShowMarriage(i) => {
                if let Some(r) = self.primary {
                    self.shown_marriage.insert(r, i);
                }
            }
            Cmd::ShowParents(i) => {
                if let Some(r) = self.primary {
                    self.shown_parents.insert(r, i);
                }
            }
            Cmd::GettingStarted => self.message(
                "Getting Started",
                "1. Double-click the dark box on the Family screen and type your own name, with slashes around the surname: Anna Marie /Lee/.\n\n\
                 2. Double-click an empty box to add a spouse or a parent, or the Children area to add a child.\n\n\
                 3. Click the triangle beside a person to move them into the main position. The F, M and C keys go to the father, mother or a child.\n\n\
                 4. Use the Pedigree tab to see five generations, and the Individual tab to see everyone in a list.\n\n\
                 5. Right-click anyone for more choices, such as Notes and Multimedia.\n\n\
                 6. Use File, Save As to name your family file. Changes save by themselves after that.\n\n\
                 7. Use File, Print Reports for charts and lists, and File, Backup to keep a safe copy.",
            ),
            Cmd::SelectSlot(s, r) => {
                self.slot = s;
                self.selected = r;
            }
            Cmd::GoFather => {
                let (f, _, _, _) = self.primary.map(|r| self.family_of(r)).unwrap_or_default();
                if let Some(f) = f {
                    self.to_primary(f);
                }
            }
            Cmd::GoMother => {
                let (_, m, _, _) = self.primary.map(|r| self.family_of(r)).unwrap_or_default();
                if let Some(m) = m {
                    self.to_primary(m);
                }
            }
            Cmd::NextChild => {
                if let Some(r) = self.primary {
                    self.c_key(r);
                }
            }
        }
    }

    /// Opens the person's name in a browser. Nothing is sent until the person picks this command.
    fn web_search(&mut self, site: &str) {
        let Some(p) = self.selected.or(self.primary).and_then(|r| self.db.person(r)) else {
            self.message("Search", "Select an individual first.");
            return;
        };
        let enc = |s: &str| -> String {
            s.bytes()
                .map(|b| match b {
                    b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' => (b as char).to_string(),
                    _ => format!("%{b:02X}"),
                })
                .collect()
        };
        let (g, s) = (enc(&given_of(&p.name)), enc(&surname_of(&p.name)));
        let url = match site {
            "ancestry" => format!("https://www.ancestry.com/search/?name={g}_{s}"),
            "genealogy" => format!("https://www.google.com/search?q=site%3Agenealogy.com+%22{g}%22+%22{s}%22"),
            "rootsweb" => format!("https://www.google.com/search?q=site%3Arootsweb.com+%22{g}%22+%22{s}%22"),
            _ => format!("https://www.familysearch.org/search/record/results?q.givenName={g}&q.surname={s}"),
        };
        let _ = open_with_system(&url);
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if !self.dialogs.is_empty() || ctx.memory(|m| m.focused().is_some()) {
            return;
        }
        let cmd = |ctx: &egui::Context, m: Modifiers, k: Key, c: Cmd, list: &mut Vec<Cmd>| {
            if ctx.input_mut(|i| i.consume_key(m, k)) {
                list.push(c);
            }
        };
        let mut list = Vec::new();
        let ctrl = Modifiers::COMMAND;
        let shift_ctrl = Modifiers { shift: true, ..Modifiers::COMMAND };
        cmd(ctx, shift_ctrl, Key::P, Cmd::Preferences, &mut list);
        cmd(ctx, ctrl, Key::O, Cmd::Open, &mut list);
        cmd(ctx, ctrl, Key::P, Cmd::Reports, &mut list);
        cmd(ctx, ctrl, Key::I, Cmd::Edit, &mut list);
        cmd(ctx, ctrl, Key::N, Cmd::Notes, &mut list);
        cmd(ctx, ctrl, Key::M, Cmd::Media, &mut list);
        cmd(ctx, ctrl, Key::A, Cmd::AddIndividual, &mut list);
        cmd(ctx, ctrl, Key::U, Cmd::AddSpouse, &mut list);
        cmd(ctx, ctrl, Key::L, Cmd::AddChild, &mut list);
        cmd(ctx, ctrl, Key::T, Cmd::AddFather, &mut list);
        cmd(ctx, ctrl, Key::H, Cmd::AddMother, &mut list);
        cmd(ctx, ctrl, Key::F, Cmd::IndividualList, &mut list);
        cmd(ctx, ctrl, Key::D, Cmd::DescendancyList, &mut list);
        cmd(ctx, ctrl, Key::R, Cmd::FindByRin, &mut list);
        cmd(ctx, ctrl, Key::Home, Cmd::Home, &mut list);
        cmd(ctx, Modifiers::NONE, Key::F1, Cmd::Keyboard, &mut list);
        if ctx.input_mut(|i| i.consume_key(ctrl, Key::S)) {
            let next = match self.view {
                View::Family => View::Pedigree,
                View::Pedigree => View::Individual,
                View::Individual => View::Family,
            };
            list.push(Cmd::SetView(next));
        }
        self.pending.extend(list);
        self.view_keys(ctx);
    }

    fn menus(&mut self, ui: &mut egui::Ui) {
        let open = !self.db.people.is_empty() || self.path.is_some();
        let has = self.selected.or(self.primary).is_some();
        let mut out: Vec<Cmd> = Vec::new();
        let item = |ui: &mut egui::Ui, out: &mut Vec<Cmd>, text: &str, keys: &str, cmd: Cmd, on: bool| {
            let mut b = egui::Button::new(tr(text));
            if !keys.is_empty() {
                b = b.shortcut_text(keys);
            }
            if ui.add_enabled(on, b).clicked() {
                out.push(cmd);
                ui.close();
            }
        };
        egui::MenuBar::new().ui(ui, |ui| {
            ui.spacing_mut().button_padding = vec2(6.0, 2.0);
            ui.add_space(20.0);
            ui.menu_button(tr("File"), |ui| {
                item(ui, &mut out, "New...", "", Cmd::New, true);
                item(ui, &mut out, "Open...", "⌘O", Cmd::Open, true);
                item(ui, &mut out, "Close", "", Cmd::Close, open);
                item(ui, &mut out, "Save As...", "", Cmd::SaveAs, true);
                ui.separator();
                item(ui, &mut out, "Import...", "", Cmd::Import, true);
                item(ui, &mut out, "Export...", "", Cmd::Export, open);
                ui.separator();
                item(ui, &mut out, "Properties", "", Cmd::Properties, open);
                item(ui, &mut out, "Backup...", "", Cmd::Backup, open);
                item(ui, &mut out, "Restore...", "", Cmd::Restore, true);
                item(ui, &mut out, "Check/Repair...", "", Cmd::CheckRepair, open);
                item(ui, &mut out, "Delete...", "", Cmd::DeleteFile, true);
                ui.separator();
                item(ui, &mut out, "Print Reports...", "⌘P", Cmd::Reports, has);
                if !self.recent.is_empty() {
                    ui.separator();
                    for (i, p) in self.recent.clone().into_iter().enumerate() {
                        let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        item(ui, &mut out, &format!("{} {n}", i + 1), "", Cmd::OpenPath(p), true);
                    }
                }
                ui.separator();
                item(ui, &mut out, "Exit", "", Cmd::Exit, true);
            });
            ui.menu_button(tr("Edit"), |ui| {
                item(ui, &mut out, "Individual", "⌘I", Cmd::Edit, has);
                item(ui, &mut out, "Notes", "⌘N", Cmd::Notes, has);
                item(ui, &mut out, "Multimedia", "⌘M", Cmd::Media, has);
                ui.separator();
                item(ui, &mut out, "Unlink Individual", "", Cmd::Unlink, has);
                item(ui, &mut out, "Delete Individual", "", Cmd::DeletePerson, has);
                ui.separator();
                item(ui, &mut out, "Marriage", "", Cmd::EditMarriage, has);
                item(ui, &mut out, "Parents", "", Cmd::EditParents, has);
                item(ui, &mut out, "Parents' Marriage", "", Cmd::ParentsMarriage, has);
                ui.separator();
                item(ui, &mut out, "Order Spouses...", "", Cmd::OrderSpouses, has);
                item(ui, &mut out, "Order Children...", "", Cmd::OrderChildren, has);
                ui.separator();
                item(ui, &mut out, "Source List", "", Cmd::SourceList, true);
                item(ui, &mut out, "Repository List", "", Cmd::RepositoryList, true);
                item(ui, &mut out, "Individual Event List", "", Cmd::PersonEvents, true);
                item(ui, &mut out, "Marriage Event List", "", Cmd::MarriageEvents, true);
            });
            ui.menu_button(tr("Add"), |ui| {
                item(ui, &mut out, "Individual...", "⌘A", Cmd::AddIndividual, true);
                item(ui, &mut out, "Family...", "", Cmd::AddFamily, true);
                item(ui, &mut out, "Spouse...", "⌘U", Cmd::AddSpouse, has);
                item(ui, &mut out, "Child...", "⌘L", Cmd::AddChild, has);
                item(ui, &mut out, "Father...", "⌘T", Cmd::AddFather, has);
                item(ui, &mut out, "Mother...", "⌘H", Cmd::AddMother, has);
                item(ui, &mut out, "Other Parents...", "", Cmd::AddOtherParents, has);
            });
            ui.menu_button(tr("View"), |ui| {
                item(ui, &mut out, "Family", "", Cmd::SetView(View::Family), true);
                item(ui, &mut out, "Pedigree", "", Cmd::SetView(View::Pedigree), true);
                item(ui, &mut out, "Individual", "", Cmd::SetView(View::Individual), true);
                ui.separator();
                ui.checkbox(&mut self.show_toolbar, tr("Toolbar"));
                ui.checkbox(&mut self.show_status, tr("Status Bar"));
                ui.separator();
                ui.menu_button(tr("Colors"), |ui| {
                    for (i, (name, group, _)) in crate::look::SCHEMES.iter().enumerate() {
                        if i == 0 || crate::look::SCHEMES[i - 1].1 != *group {
                            if i > 0 {
                                ui.separator();
                            }
                            ui.label(egui::RichText::new(tr(group)).weak());
                        }
                        if ui.selectable_label(crate::look::current() == i, tr(name)).clicked() {
                            choose_colors(ui.ctx(), i);
                            ui.close();
                        }
                    }
                });
                ui.separator();
                // Each language is listed under its own name, so it can be found from any language.
                ui.label(egui::RichText::new(tr("Language")).weak());
                for (i, name) in crate::i18n::LANGUAGES.iter().enumerate() {
                    if ui.selectable_label(crate::i18n::current() == i, *name).clicked() {
                        choose_language(i);
                        ui.close();
                    }
                }
            });
            ui.menu_button(tr("Search"), |ui| {
                item(ui, &mut out, "Individual List...", "⌘F", Cmd::IndividualList, open);
                item(ui, &mut out, "Marriage List...", "", Cmd::MarriageList, open);
                item(ui, &mut out, "Descendancy List...", "⌘D", Cmd::DescendancyList, has);
                item(ui, &mut out, "Find by RIN/MRIN...", "⌘R", Cmd::FindByRin, open);
                item(ui, &mut out, "Advanced Focus/Filter...", "", Cmd::Focus, open);
                ui.separator();
                item(ui, &mut out, "Home Person", "⌘Home", Cmd::Home, has);
                item(ui, &mut out, "Make Selected the Home Person", "", Cmd::SetHome, has);
                ui.separator();
                item(ui, &mut out, "On FamilySearch", "", Cmd::WebSearch("familysearch"), has);
                item(ui, &mut out, "On Ancestry", "", Cmd::WebSearch("ancestry"), has);
                item(ui, &mut out, "On Genealogy.com", "", Cmd::WebSearch("genealogy"), has);
                item(ui, &mut out, "On RootsWeb", "", Cmd::WebSearch("rootsweb"), has);
            });
            ui.menu_button(tr("Tools"), |ui| {
                item(ui, &mut out, "Match/Merge...", "", Cmd::MatchMerge, open);
                item(ui, &mut out, "Merge Duplicate Sources", "", Cmd::MergeSources, open);
                ui.separator();
                item(ui, &mut out, "Global Search and Replace...", "", Cmd::GlobalReplace, open);
                item(ui, &mut out, "Change Names to Mixed Case", "", Cmd::MixedCase, open);
                item(ui, &mut out, "Renumber Records...", "", Cmd::Renumber, has);
                item(ui, &mut out, "Create Web Page...", "", Cmd::WebPage, open);
                item(ui, &mut out, "Notepad", "", Cmd::Notepad, true);
                ui.separator();
                item(ui, &mut out, "Relationship Calculator", "", Cmd::Relationship, open);
                item(ui, &mut out, "Date Calculator", "", Cmd::DateCalc, true);
                item(ui, &mut out, "Soundex Calculator", "", Cmd::Soundex, true);
                ui.separator();
                item(ui, &mut out, "Preferences", "⇧⌘P", Cmd::Preferences, true);
            });
            ui.menu_button(tr("Window"), |ui| {
                ui.label(tr("This version keeps one family window."));
            });
            ui.menu_button(tr("Help"), |ui| {
                item(ui, &mut out, "Getting Started", "", Cmd::GettingStarted, true);
                item(ui, &mut out, "Keyboard", "", Cmd::Keyboard, true);
                item(ui, &mut out, "Open Sample Family", "", Cmd::Sample, true);
                ui.separator();
                item(ui, &mut out, &format!("About {APP_NAME}"), "", Cmd::About, true);
            });
        });
        self.pending.extend(out);
    }

    /// Button order and x positions follow the original's toolbar. Its button for the TempleReady export became Backup.
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let has = self.selected.or(self.primary).is_some();
        let open = !self.db.people.is_empty() || self.path.is_some();
        let r = ui.max_rect();
        let tools: [(f32, Icon, &str, Cmd, bool); 18] = [
            (22.0, Icon::Folder, "Open family file (Ctrl+O)", Cmd::Open, true),
            (49.0, Icon::Disk, "Import a GEDCOM or .paf file", Cmd::Import, true),
            (75.0, Icon::Page, "Export a GEDCOM file", Cmd::Export, open),
            (102.0, Icon::Disk, "Back up this family file", Cmd::Backup, open),
            (129.0, Icon::Printer, "Print reports (Ctrl+P)", Cmd::Reports, has),
            (162.0, Icon::Pencil, "Edit individual (Ctrl+I)", Cmd::Edit, has),
            (189.0, Icon::Page, "Edit notes (Ctrl+N)", Cmd::Notes, has),
            (216.0, Icon::Camera, "Multimedia (Ctrl+M)", Cmd::Media, has),
            (243.0, Icon::PersonAdd, "Add individual (Ctrl+A)", Cmd::AddIndividual, true),
            (270.0, Icon::CoupleAdd, "Add family", Cmd::AddFamily, true),
            (304.0, Icon::Find, "Individual list (Ctrl+F)", Cmd::IndividualList, open),
            (331.0, Icon::FindCouple, "Find by RIN or MRIN (Ctrl+R)", Cmd::FindByRin, open),
            (358.0, Icon::Tree, "Descendancy list (Ctrl+D)", Cmd::DescendancyList, has),
            (385.0, Icon::Home, "Go to the home person (Ctrl+Home)", Cmd::Home, has),
            (417.0, Icon::Merge, "Match/Merge", Cmd::MatchMerge, open),
            (444.0, Icon::Calc, "Create a web page", Cmd::WebPage, open),
            (478.0, Icon::Calc, "Relationship calculator", Cmd::Relationship, open),
            (505.0, Icon::Help, "Keyboard help (F1)", Cmd::Keyboard, true),
        ];
        let p = ui.painter().clone();
        for x in [147.0, 288.0, 401.0, 461.0] {
            p.vline(r.left() + x, (r.top() + 3.0)..=(r.bottom() - 4.0), Stroke::new(1.0, pal().shadow));
            p.vline(r.left() + x + 1.0, (r.top() + 3.0)..=(r.bottom() - 4.0), Stroke::new(1.0, pal().light));
        }
        let mut out = Vec::new();
        for (x, icon, tip, cmd, on) in tools {
            let rect = Rect::from_center_size(pos2(r.left() + x, r.center().y), vec2(24.0, 24.0));
            let resp = ui.allocate_rect(rect, if on { Sense::click() } else { Sense::hover() });
            if on && resp.hovered() {
                p.rect(rect, 1.0, pal().hover, Stroke::new(1.0, pal().shadow), StrokeKind::Inside);
            }
            draw_icon(&p, rect.shrink(4.0), icon, on);
            if resp.on_hover_text(tr(tip)).clicked() {
                out.push(cmd);
            }
        }
        self.pending.extend(out);
    }

    /// The hint is the original's own wording. The panes keep the positions in the person's
    /// pictures, and show what the highlighted person's numbers and the family's size are.
    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let r = ui.max_rect();
        let p = ui.painter().clone();
        p.hline(r.left()..=r.right(), r.top(), Stroke::new(1.0, pal().shadow));
        let text = if !self.status.is_empty() {
            self.status.clone()
        } else if self.db.prefs.show_relationships {
            self.primary.and_then(|x| crate::calc::relation_to_root(&self.db, x)).map(|x| format!("Relationship: {x}")).unwrap_or_else(|| tr("For Help, press F1. Double-click on person you want to edit.").to_string())
        } else {
            tr("For Help, press F1. Double-click on person you want to edit.").to_string()
        };
        p.text(pos2(r.left() + 3.0, r.center().y), egui::Align2::LEFT_CENTER, text, egui::FontId::proportional(12.0), pal().ink);
        let right = r.right();
        let rin = self.selected.map(|x| format!("RIN {x}")).unwrap_or_default();
        let mrin = self.primary.and_then(|x| self.current_marriage(x)).map(|m| format!("MRIN {m}")).unwrap_or_default();
        let count = if self.db.people.is_empty() { String::new() } else { self.db.people.len().to_string() };
        for (x0, x1, t) in [(683.0, 781.0, rin), (782.0, 880.0, mrin), (881.0, 917.0, count), (918.0, 954.0, String::new()), (955.0, 993.0, String::new()), (994.0, 1022.0, String::new())] {
            let rc = Rect::from_min_max(pos2(right - 1024.0 + x0, r.top() + 2.0), pos2(right - 1024.0 + x1, r.bottom() - 2.0));
            p.rect_stroke(rc, 0.0, Stroke::new(1.0, pal().shadow), StrokeKind::Inside);
            p.text(rc.center(), egui::Align2::CENTER_CENTER, t, egui::FontId::proportional(11.5), pal().ink);
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let name = match &self.path {
            Some(p) => p.display().to_string(),
            None if self.dirty => tr("Not saved yet").to_string(),
            None => tr("No file").to_string(),
        };
        let title = format!("{APP_NAME} - [{name}]");
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }

    fn exit_prompt(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.really_exit && self.dirty && self.path.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_exit = true;
        }
        if !self.confirm_exit {
            return;
        }
        if !(self.dirty && self.path.is_none()) {
            self.really_exit = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            self.confirm_exit = false;
            return;
        }
        egui::Modal::new(egui::Id::unique("exit")).show(ctx, |ui| {
            ui.label(tr("This family has not been saved to a file. Save it before you exit?"));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(tr("Save As...")).clicked() {
                    self.save_as_dialog();
                    self.confirm_exit = false;
                }
                if ui.button(tr("Exit without saving")).clicked() {
                    self.really_exit = true;
                    self.confirm_exit = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button(tr("Cancel")).clicked() {
                    self.confirm_exit = false;
                }
            });
        });
    }

    fn screenshot_step(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot.clone() else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.shot_done.load(std::sync::atomic::Ordering::SeqCst) {
            std::process::exit(0);
        }
        let last_step = self.steps.iter().map(|s| s.0).max().unwrap_or(0);
        if self.frames >= 120.max(last_step + 30) && !self.screenshot_requested {
            self.screenshot_requested = true;
            let done = self.shot_done.clone();
            ctx.request_screenshot(move |shot: Arc<egui::ColorImage>| {
                let [w, h] = shot.size;
                let mut rgba = Vec::with_capacity(w * h * 4);
                for px in &shot.pixels {
                    rgba.extend_from_slice(&px.to_srgba_unmultiplied());
                }
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = image::save_buffer(&path, &rgba, w as u32, h as u32, image::ColorType::Rgba8) {
                    eprintln!("Could not save the picture: {e}");
                }
                done.store(true, std::sync::atomic::Ordering::SeqCst);
            });
        }
        // A window that never finishes must not hang whoever asked for the picture.
        if self.frames > 6000 {
            eprintln!("The window picture never arrived.");
            std::process::exit(1);
        }
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        if !self.steps.is_empty() {
            self.input_frames += 1;
            crate::script::feed(&self.steps, self.input_frames, raw);
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

#[derive(Clone, Copy)]
pub enum Icon {
    Folder,
    Disk,
    Pencil,
    Page,
    Printer,
    #[allow(dead_code)]
    Couple,
    Camera,
    PersonAdd,
    CoupleAdd,
    Find,
    FindCouple,
    Tree,
    Home,
    Merge,
    Calc,
    Help,
}

/// Small drawn icons, made for this program rather than taken from the original.
pub fn draw_icon(p: &egui::Painter, r: Rect, icon: Icon, on: bool) {
    let ink = if on { Color32::from_rgb(40, 40, 40) } else { Color32::from_rgb(150, 148, 135) };
    let fill = |c: Color32| if on { c } else { Color32::from_rgb(200, 197, 182) };
    let s = Stroke::new(1.0, ink);
    let at = |x: f32, y: f32| pos2(r.left() + x * r.width(), r.top() + y * r.height());
    let box_ = |x1: f32, y1: f32, x2: f32, y2: f32| Rect::from_two_pos(at(x1, y1), at(x2, y2));
    let person = |x: f32, c: Color32| {
        p.circle(at(x, 0.28), r.width() * 0.17, fill(c), s);
        p.rect(box_(x - 0.2, 0.5, x + 0.2, 0.95), 2.0, fill(c), s, StrokeKind::Inside);
    };
    let plus = |x: f32, y: f32| {
        let green = Stroke::new(2.0, if on { Color32::from_rgb(30, 130, 50) } else { ink });
        p.line_segment([at(x - 0.14, y), at(x + 0.14, y)], green);
        p.line_segment([at(x, y - 0.14), at(x, y + 0.14)], green);
    };
    let lens = |x: f32, y: f32| {
        p.circle(at(x, y), r.width() * 0.2, fill(Color32::from_rgb(190, 220, 240)), Stroke::new(1.5, ink));
        p.line_segment([at(x + 0.14, y + 0.14), at(x + 0.34, y + 0.34)], Stroke::new(2.0, ink));
    };
    match icon {
        Icon::Folder => {
            p.rect(box_(0.0, 0.28, 0.4, 0.45), 1.0, fill(Color32::from_rgb(240, 200, 90)), s, StrokeKind::Inside);
            p.rect(box_(0.0, 0.4, 1.0, 0.92), 1.0, fill(Color32::from_rgb(245, 210, 110)), s, StrokeKind::Inside);
        }
        Icon::Disk => {
            p.rect(box_(0.05, 0.05, 0.95, 0.95), 1.0, fill(Color32::from_rgb(70, 90, 160)), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.05, 0.75, 0.4), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.58, 0.75, 0.95), 0.0, fill(Color32::from_rgb(210, 210, 220)), s, StrokeKind::Inside);
        }
        Icon::Pencil => {
            p.rect(box_(0.1, 0.05, 0.8, 0.95), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            p.line_segment([at(0.35, 0.8), at(0.95, 0.2)], Stroke::new(2.5, if on { Color32::from_rgb(200, 120, 30) } else { ink }));
        }
        Icon::Page => {
            p.rect(box_(0.15, 0.03, 0.85, 0.97), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            for y in [0.25, 0.45, 0.65, 0.82] {
                p.line_segment([at(0.27, y), at(0.73, y)], s);
            }
        }
        Icon::Printer => {
            p.rect(box_(0.25, 0.05, 0.75, 0.4), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            p.rect(box_(0.03, 0.38, 0.97, 0.75), 2.0, fill(Color32::from_rgb(190, 190, 195)), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.62, 0.75, 0.97), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
        }
        Icon::Couple => {
            person(0.3, Color32::from_rgb(120, 150, 210));
            person(0.72, Color32::from_rgb(220, 130, 150));
        }
        Icon::Camera => {
            p.rect(box_(0.03, 0.3, 0.97, 0.9), 2.0, fill(Color32::from_rgb(120, 120, 130)), s, StrokeKind::Inside);
            p.rect(box_(0.33, 0.15, 0.67, 0.32), 0.0, fill(Color32::from_rgb(120, 120, 130)), s, StrokeKind::Inside);
            p.circle(at(0.5, 0.6), r.width() * 0.2, fill(Color32::from_rgb(190, 220, 240)), s);
        }
        Icon::PersonAdd => {
            person(0.36, Color32::from_rgb(120, 150, 210));
            plus(0.82, 0.25);
        }
        Icon::CoupleAdd => {
            person(0.24, Color32::from_rgb(120, 150, 210));
            person(0.62, Color32::from_rgb(220, 130, 150));
            plus(0.86, 0.2);
        }
        Icon::Find => lens(0.42, 0.42),
        Icon::FindCouple => {
            person(0.25, Color32::from_rgb(120, 150, 210));
            lens(0.6, 0.55);
        }
        Icon::Tree => {
            p.line_segment([at(0.15, 0.1), at(0.15, 0.85)], s);
            for y in [0.3, 0.6, 0.85] {
                p.line_segment([at(0.15, y), at(0.45, y)], s);
                p.rect(box_(0.45, y - 0.09, 0.95, y + 0.09), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            }
        }
        Icon::Home => {
            let roof = vec![at(0.5, 0.05), at(1.0, 0.5), at(0.0, 0.5)];
            p.add(egui::Shape::convex_polygon(roof, fill(Color32::from_rgb(180, 70, 60)), s));
            p.rect(box_(0.15, 0.5, 0.85, 0.97), 0.0, fill(Color32::from_rgb(235, 225, 190)), s, StrokeKind::Inside);
        }
        Icon::Merge => {
            p.line_segment([at(0.05, 0.15), at(0.5, 0.5)], Stroke::new(2.0, ink));
            p.line_segment([at(0.05, 0.85), at(0.5, 0.5)], Stroke::new(2.0, ink));
            p.line_segment([at(0.5, 0.5), at(0.97, 0.5)], Stroke::new(2.0, ink));
        }
        Icon::Calc => {
            p.rect(box_(0.15, 0.03, 0.85, 0.97), 1.0, fill(Color32::from_rgb(210, 210, 215)), s, StrokeKind::Inside);
            p.rect(box_(0.25, 0.12, 0.75, 0.33), 0.0, fill(Color32::WHITE), s, StrokeKind::Inside);
            for y in [0.48, 0.65, 0.82] {
                for x in [0.33, 0.5, 0.67] {
                    p.circle_filled(at(x, y), 1.3, ink);
                }
            }
        }
        Icon::Help => {
            p.text(r.center(), egui::Align2::CENTER_CENTER, tr("?"), egui::FontId::new(r.height() * 1.1, FontFamily::Name("bold".into())), if on { Color32::from_rgb(40, 80, 170) } else { ink });
        }
    }
}
