//! Carrying out a command from a menu, toolbar button or key.

use crate::i18n::tr_n;
use super::*;
use crate::dialogs::*;
use crate::formats::gedcom;
use crate::data::*;

impl App {
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
                self.message("Merge Duplicate Sources", tr_n("Merged {n} duplicate source.", "Merged {n} duplicate sources.", n, &[]));
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
}
