//! The menu bar, toolbar, status bar, window title and the exit prompt.

use crate::i18n::{tr, trf};
use super::*;
use egui::{Key, Modifiers, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

impl App {
    pub(crate) fn shortcuts(&mut self, ctx: &egui::Context) {
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

    pub(crate) fn menus(&mut self, ui: &mut egui::Ui) {
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
                    for (i, (name, group, _)) in crate::app::colors::SCHEMES.iter().enumerate() {
                        if i == 0 || crate::app::colors::SCHEMES[i - 1].1 != *group {
                            if i > 0 {
                                ui.separator();
                            }
                            ui.label(egui::RichText::new(tr(group)).weak());
                        }
                        if ui.selectable_label(crate::app::colors::current() == i, tr(name)).clicked() {
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
                item(ui, &mut out, &trf("About {program}", &[("program", &APP_NAME)]), "", Cmd::About, true);
            });
        });
        self.pending.extend(out);
    }

    /// Button order and x positions follow the original's toolbar. Its button for the TempleReady export became Backup.
    pub(crate) fn toolbar(&mut self, ui: &mut egui::Ui) {
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
    pub(crate) fn status_bar(&mut self, ui: &mut egui::Ui) {
        let r = ui.max_rect();
        let p = ui.painter().clone();
        p.hline(r.left()..=r.right(), r.top(), Stroke::new(1.0, pal().shadow));
        let text = if !self.status.is_empty() {
            self.status.clone()
        } else if self.db.prefs.show_relationships {
            self.primary.and_then(|x| crate::rules::relation_to_root(&self.db, x)).map(|x| format!("Relationship: {x}")).unwrap_or_else(|| tr("For Help, press F1. Double-click on person you want to edit.").to_string())
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

    pub(crate) fn update_title(&mut self, ctx: &egui::Context) {
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

    pub(crate) fn exit_prompt(&mut self, ctx: &egui::Context) {
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
}
