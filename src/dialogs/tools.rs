//! Windows for reports, calculators, preferences and file tasks.

//! Windows for reports, calculators, preferences and file tasks.
use crate::i18n::{tr, tr_n, trf};
use super::*;
use crate::app::*;
use crate::rules;
use crate::formats::gedcom;
use crate::data::*;
use crate::reports::{self, Report};
use egui::{Key, RichText, TextEdit};

impl App {
    pub fn show_tool_dialog(&mut self, ctx: &egui::Context, dlg: &mut Dialog, is_top: bool) -> bool {
        let mut open = true;
        let escape = is_top && ctx.input(|i| i.key_pressed(Key::Escape));
        match dlg {
            Dialog::WebPage { id, title, hide_living, include_notes, focus_only } => {
                let mut go = false;
                window("Create Web Page", *id).resizable(false).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tr("Title:"));
                        ui.add(TextEdit::singleline(title).desired_width(320.0));
                    });
                    ui.checkbox(hide_living, tr("Hide details of people who may still be living"));
                    ui.checkbox(include_notes, tr("Include notes"));
                    ui.add_enabled_ui(self.focus.is_some(), |ui| ui.checkbox(focus_only, tr("Only the current focus list")));
                    ui.label(RichText::new(tr("Pages are saved in a folder you choose. Nothing is uploaded.")).size(12.0));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Create")).clicked() {
                            go = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if go {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        let o = reports::WebOptions { title: title.clone(), hide_living: *hide_living, include_notes: *include_notes, who: if *focus_only { self.focus.clone() } else { None } };
                        match reports::write_web(&self.db, &o, &folder) {
                            Ok(n) => self.message("Create Web Page", tr_n("Web page creation complete: {n} page in {folder}.", "Web page creation complete: {n} pages in {folder}.", n, &[("folder", &folder.display())])),
                            Err(e) => self.message("Create Web Page", trf("The pages could not be saved:\n{error}", &[("error", &e)])),
                        }
                        open = false;
                    }
                }
            }
            Dialog::Notepad { id, text, path } => {
                let title = match path {
                    Some(p) => trf("Notepad: {file}", &[("file", &p.display())]),
                    None => "Notepad".to_string(),
                };
                let mut err = None;
                window(&title, *id).default_width(640.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(tr("Open...")).clicked() {
                            if let Some(p) = rfd::FileDialog::new().add_filter("Text", &["txt"]).add_filter("All files", &["*"]).pick_file() {
                                match std::fs::read(&p) {
                                    Ok(b) => {
                                        *text = String::from_utf8_lossy(&b).into_owned();
                                        *path = Some(p);
                                    }
                                    Err(e) => err = Some(e.to_string()),
                                }
                            }
                        }
                        if ui.button(tr("Save...")).clicked() {
                            let target = path.clone().or_else(|| rfd::FileDialog::new().add_filter("Text", &["txt"]).set_file_name("Notes.txt").save_file());
                            if let Some(p) = target {
                                match std::fs::write(&p, text.as_bytes()) {
                                    Ok(()) => *path = Some(p),
                                    Err(e) => err = Some(e.to_string()),
                                }
                            }
                        }
                        if ui.button(tr("Close")).clicked() {
                            open = false;
                        }
                    });
                    egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                        ui.add(TextEdit::multiline(text).desired_width(620.0).desired_rows(22));
                    });
                });
                if let Some(e) = err {
                    self.message("Notepad", e);
                }
            }
            Dialog::Relationship { id, a, b, result } => {
                let dialog_id = *id;
                let (mut pick_a, mut pick_b) = (false, false);
                window("Relationship Calculator", *id).default_width(460.0).show(ctx, |ui| {
                    ui.group(|ui| {
                        ui.set_min_width(420.0);
                        ui.label(RichText::new(tr("First Person")).strong());
                        ui.horizontal(|ui| {
                            ui.label(a.map(|r| self.db.name(r)).unwrap_or_default());
                            if ui.button(tr("Search")).clicked() {
                                pick_a = true;
                            }
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_width(420.0);
                        ui.label(RichText::new(tr("Second Person")).strong());
                        ui.horizontal(|ui| {
                            ui.label(b.map(|r| self.db.name(r)).unwrap_or_default());
                            if ui.button(tr("Search")).clicked() {
                                pick_b = true;
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        if ui.add_enabled(a.is_some() && b.is_some(), egui::Button::new(tr("Calculate"))).clicked() {
                            let found = rules::relationships(&self.db, a.unwrap(), b.unwrap());
                            *result = if found.is_empty() {
                                vec!["No relationship was found through shared ancestors.".to_string()]
                            } else {
                                found
                                    .iter()
                                    .flat_map(|r| {
                                        let mut v = vec![r.words.clone()];
                                        let names: Vec<String> = r.ancestors.iter().map(|x| self.db.name(*x)).collect();
                                        match names.len() {
                                            0 => {}
                                            1 => v.push(trf("Their common ancestor is {name}", &[("name", &names[0])])),
                                            _ => v.push(trf("Their common ancestors are {name} and {other}", &[("name", &names[0]), ("other", &names[1])])),
                                        }
                                        v
                                    })
                                    .collect()
                            };
                        }
                        if ui.button(tr("Close")).clicked() || escape {
                            open = false;
                        }
                    });
                    for r in result.iter() {
                        ui.label(r);
                    }
                });
                if pick_a {
                    self.push(|id| Dialog::new_picker(id, PickFor::RelationA(dialog_id)));
                }
                if pick_b {
                    self.push(|id| Dialog::new_picker(id, PickFor::RelationB(dialog_id)));
                }
            }
            Dialog::DateCalc { id, start, end, years, months, days, note } => {
                let df = self.db.prefs.day_first_numeric;
                window("Date Calculator", *id).resizable(false).show(ctx, |ui| {
                    egui::Grid::new("dc").num_columns(2).show(ui, |ui| {
                        text_row(ui, "Start date:", start, 150.0);
                        text_row(ui, "Years:", years, 60.0);
                        text_row(ui, "Months:", months, 60.0);
                        text_row(ui, "Days:", days, 60.0);
                        text_row(ui, "End date:", end, 150.0);
                    });
                    ui.label(RichText::new(tr("Fill in any two of start, end and elapsed time. Works for the Gregorian calendar only, from October 1752.")).size(12.0));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Calculate")).clicked() {
                            let a = parse_ymd(start, df);
                            let b = parse_ymd(end, df);
                            let num = |s: &String| s.trim().parse::<i32>().unwrap_or(0);
                            let elapsed = !years.trim().is_empty() || !months.trim().is_empty() || !days.trim().is_empty();
                            let in_range = |d: (i32, u32, u32)| rules::date_in_calendar(d.0, d.1);
                            *note = String::new();
                            if [a, b].into_iter().flatten().any(|d| !in_range(d)) {
                                *note = "Dates before October 1752 cannot be calculated: some places had not yet changed to the Gregorian calendar.".into();
                            } else if let (Some(x), Some(y)) = (a, b) {
                                if rules::serial(x.0, x.1, x.2) > rules::serial(y.0, y.1, y.2) {
                                    *note = "Start date must be before end date.".into();
                                } else {
                                    let (yy, mm, dd) = rules::date_difference(x, y);
                                    *years = yy.to_string();
                                    *months = mm.to_string();
                                    *days = dd.to_string();
                                }
                            } else if let (Some(x), true) = (a, elapsed) {
                                let r = rules::date_add(x, num(years), num(months), num(days));
                                *end = rules::format_date(r.0, r.1, r.2);
                            } else if let (Some(y), true) = (b, elapsed) {
                                let r = rules::date_add(y, -num(years), -num(months), -num(days));
                                if in_range((r.0, r.1, r.2)) {
                                    *start = rules::format_date(r.0, r.1, r.2);
                                } else {
                                    *note = "The answer falls before October 1752, which cannot be calculated.".into();
                                }
                            } else {
                                *note = "Type full dates, such as 23 Mar 1842, and any elapsed time.".into();
                            }
                        }
                        if ui.button(tr("Clear Fields")).clicked() {
                            for f in [&mut *start, &mut *end, &mut *years, &mut *months, &mut *days, &mut *note] {
                                f.clear();
                            }
                        }
                        if ui.button(tr("Close")).clicked() || escape {
                            open = false;
                        }
                    });
                    if !note.is_empty() {
                        ui.label(RichText::new(note.as_str()).color(crate::app::colors::pal().warn));
                    }
                });
            }
            Dialog::Renumber { id, how, preview } => {
                let mut apply = false;
                window("Renumber Records", *id).resizable(false).show(ctx, |ui| {
                    ui.set_max_width(520.0);
                    ui.label(tr("Gives everyone new record numbers (RINs) 1, 2, 3... with no gaps, and numbers the marriages (MRINs) to match. Every link between people stays the same."));
                    ui.add_space(4.0);
                    let names = ["Family order: the home person first, then parents, grandparents and the rest of the family", "Present order, closing the gaps", "Alphabetical by surname"];
                    for (i, n) in names.iter().enumerate() {
                        if ui.radio_value(how, i as u8, tr(n)).changed() {
                            *preview = None;
                        }
                    }
                    ui.add_space(4.0);
                    ui.label(RichText::new(tr("Each person keeps their old number as an Old ID, shown when editing them and written to exported GEDCOM files, so notes and printouts that use the old numbers can still be matched. A copy of the file as it is now is saved beside it first.")).size(12.0));
                    let order = match *how {
                        0 => RenumberOrder::Family,
                        1 => RenumberOrder::Present,
                        _ => RenumberOrder::Alphabetical,
                    };
                    if preview.is_none() {
                        let list = self.db.renumber_order(order);
                        *preview = Some(list.iter().take(200).enumerate().map(|(i, r)| (*r, i as u32 + 1, self.db.name(*r))).collect());
                    }
                    if let Some(rows) = preview.as_ref() {
                        let changed = rows.iter().filter(|(o, n, _)| o != n).count();
                        ui.label(RichText::new(format!("{} {}", tr("Preview, first people:"), if changed == 0 && rows.len() == self.db.people.len() { tr("nothing would change.") } else { "" })).strong());
                        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                            egui::Grid::new("renum").striped(true).num_columns(3).show(ui, |ui| {
                                ui.label(RichText::new(tr("New RIN")).weak());
                                ui.label(RichText::new(tr("Old RIN")).weak());
                                ui.label(RichText::new(tr("Name")).weak());
                                ui.end_row();
                                for (old, new, name) in rows {
                                    ui.label(new.to_string());
                                    ui.label(old.to_string());
                                    ui.label(name);
                                    ui.end_row();
                                }
                            });
                        });
                    }
                    ui.horizontal(|ui| {
                        if ui.button(tr("Renumber")).clicked() {
                            apply = true;
                            open = false;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if apply {
                    let order = self.db.renumber_order(match *how {
                        0 => RenumberOrder::Family,
                        1 => RenumberOrder::Present,
                        _ => RenumberOrder::Alphabetical,
                    });
                    let copy = self.safety_copy("renumbering");
                    let (rins, _) = self.db.renumber(&order);
                    self.after_renumber(&rins);
                    let mut text = format!("{} {}", tr_n("Renumbered {n} individual", "Renumbered {n} individuals", self.db.people.len(), &[]), tr_n("and {n} marriage.", "and {n} marriages.", self.db.marriages.len(), &[]));
                    if let Some(c) = copy {
                        text.push_str(&format!("\nThe file as it was before is saved as {}.", c.display()));
                    }
                    self.message("Renumber Records", text);
                }
            }
            Dialog::Soundex { id, name } => {
                window("Soundex Code Calculator", *id).resizable(false).show(ctx, |ui| {
                    egui::Grid::new("sx").num_columns(2).show(ui, |ui| {
                        ui.label(tr("Name:"));
                        let r = ui.add(TextEdit::singleline(name).desired_width(170.0));
                        if is_top && name.is_empty() {
                            r.request_focus();
                        }
                        ui.end_row();
                        ui.label(tr("Soundex code:"));
                        ui.label(RichText::new(rules::soundex(name)).strong());
                        ui.end_row();
                    });
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
            }
            Dialog::Preferences { id, draft, home, root, tab } => {
                let dialog_id = *id;
                let (mut save, mut pick_home, mut pick_root, mut flush) = (false, false, false, false);
                window("Preferences", *id).default_width(520.0).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        for (n, t) in [(0u8, "General"), (1, "File"), (2, "Names"), (3, "Prepared by"), (4, "Formats"), (5, "InfoBox"), (6, "Problems")] {
                            ui.selectable_value(tab, n, tr(t));
                        }
                    });
                    ui.separator();
                    match *tab {
                        0 => {
                            // The language belongs to the program, not the family file.
                            ui.horizontal(|ui| {
                                ui.label(tr("Language"));
                                let mut language = crate::i18n::current();
                                egui::ComboBox::from_id_salt("language").selected_text(crate::i18n::LANGUAGES[language]).show_ui(ui, |ui| {
                                    for (i, name) in crate::i18n::LANGUAGES.iter().enumerate() {
                                        ui.selectable_value(&mut language, i, *name);
                                    }
                                });
                                if language != crate::i18n::current() {
                                    choose_language(language);
                                }
                            });
                            ui.separator();
                            ui.checkbox(&mut draft.use_lds, tr("Use LDS data"));
                            ui.checkbox(&mut draft.show_rin, tr("Show RIN numbers after names"));
                            ui.checkbox(&mut draft.verify_new_places, tr("Verify new places"));
                            ui.checkbox(&mut draft.edit_marriage_when_created, tr("Edit marriage when created"));
                            ui.checkbox(&mut draft.use_list_navigating, tr("Use list when navigating"));
                            ui.checkbox(&mut draft.allow_afn_edit, tr("Allow AFN edit"));
                            ui.checkbox(&mut draft.check_duplicates, tr("Check for duplicates when adding individuals"));
                            ui.horizontal(|ui| {
                                ui.label(tr("Backup frequency:"));
                                ui.add(egui::DragValue::new(&mut draft.backup_frequency).range(0..=100));
                                ui.label(tr("closes between reminders (0 for never)"));
                            });
                        }
                        1 => {
                            ui.horizontal(|ui| {
                                ui.label(trf("Home person: {name}", &[("name", &home.map(|r| self.db.name(r)).unwrap_or_default())]));
                                if ui.button(tr("Search")).clicked() {
                                    pick_home = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(trf("Root person for relationships: {name}", &[("name", &root.map(|r| self.db.name(r)).unwrap_or_default())]));
                                if ui.button(tr("Search")).clicked() {
                                    pick_root = true;
                                }
                            });
                            ui.checkbox(&mut draft.show_relationships, tr("Show relationships on the status bar"));
                            ui.checkbox(&mut draft.log_changes, tr("Log changes"));
                            ui.horizontal(|ui| {
                                ui.label(format!("{} log entries", self.db.log.len()));
                                if ui.button(tr("Flush")).clicked() {
                                    flush = true;
                                }
                            });
                            if !self.db.log.is_empty() {
                                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                                    for l in self.db.log.iter().rev() {
                                        ui.label(RichText::new(l).monospace().size(11.0));
                                    }
                                });
                            }
                        }
                        2 => {
                            ui.checkbox(&mut draft.auto_slash, tr("Add surname slashes automatically"));
                            ui.label(RichText::new(tr("With this on, the last word of a name without slashes becomes the surname.")).size(12.0));
                        }
                        3 => {
                            egui::Grid::new("prep").num_columns(2).show(ui, |ui| {
                                text_row(ui, "Name:", &mut draft.prepared_name, 340.0);
                                ui.label(tr("Address:"));
                                ui.add(TextEdit::multiline(&mut draft.prepared_by.address).desired_width(340.0).desired_rows(3));
                                ui.end_row();
                                text_row(ui, "Phone:", &mut draft.prepared_by.phone, 340.0);
                                text_row(ui, "Email:", &mut draft.prepared_by.email, 340.0);
                            });
                        }
                        5 => {
                            for (label, value) in [("Floating", &mut draft.floating_info), ("Locked", &mut draft.locked_info)] {
                                ui.label(RichText::new(label).strong());
                                ui.radio_value(value, 0u8, tr("None"));
                                ui.radio_value(value, 1u8, tr("Dates only"));
                                ui.radio_value(value, 2u8, tr("Dates and family information"));
                                ui.add_space(4.0);
                            }
                            ui.horizontal(|ui| {
                                ui.label(tr("Wait time:"));
                                ui.add(egui::DragValue::new(&mut draft.info_wait_tenths).range(0..=100));
                                ui.label(tr("1/10 seconds"));
                            });
                        }
                        6 => {
                            ui.label(tr("Used by the Possible Problems list."));
                            ui.horizontal(|ui| {
                                ui.label(tr("Mother's age should be between"));
                                ui.add(egui::DragValue::new(&mut draft.mother_low).range(0..=120));
                                ui.label(tr("and"));
                                ui.add(egui::DragValue::new(&mut draft.mother_high).range(0..=120));
                            });
                            ui.horizontal(|ui| {
                                ui.label(tr("Father's age should be between"));
                                ui.add(egui::DragValue::new(&mut draft.father_low).range(0..=120));
                                ui.label(tr("and"));
                                ui.add(egui::DragValue::new(&mut draft.father_high).range(0..=120));
                            });
                            ui.horizontal(|ui| {
                                ui.label(tr("Age difference between husband and wife:"));
                                ui.add(egui::DragValue::new(&mut draft.spouse_span).range(0..=120));
                                ui.label(tr("years"));
                            });
                            ui.checkbox(&mut draft.check_lds, tr("Check LDS data"));
                            ui.checkbox(&mut draft.check_order, tr("Check marriage and children order"));
                        }
                        _ => {
                            ui.label(tr("Dates typed as all numbers, such as 3/11/1955, mean:"));
                            ui.radio_value(&mut draft.day_first_numeric, false, tr("Month first: 11 March 1955"));
                            ui.radio_value(&mut draft.day_first_numeric, true, tr("Day first: 3 November 1955"));
                        }
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(tr("OK")).clicked() {
                            save = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if pick_home {
                    self.push(|id| Dialog::new_picker(id, PickFor::Home(dialog_id)));
                }
                if pick_root {
                    self.push(|id| Dialog::new_picker(id, PickFor::Root(dialog_id)));
                }
                if flush {
                    self.db.log.clear();
                    self.touch();
                }
                if save {
                    self.db.prefs = (**draft).clone();
                    self.db.home = *home;
                    self.db.root_person = *root;
                    self.touch();
                    open = false;
                }
            }
            Dialog::About { id } => {
                window(&trf("About {program}", &[("program", &APP_NAME)]), *id).resizable(false).show(ctx, |ui| {
                    ui.label(RichText::new(format!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"))).strong().size(16.0));
                    ui.label(tr("A family history program for keeping individuals, marriages, notes and sources, and for printing charts and reports."));
                    ui.add_space(6.0);
                    if ui.button(tr("OK")).clicked() || escape {
                        open = false;
                    }
                });
            }
            Dialog::Keyboard { id } => {
                window("Keyboard", *id).resizable(false).show(ctx, |ui| {
                    egui::Grid::new("keys").num_columns(2).striped(true).show(ui, |ui| {
                        for (k, t) in [
                            ("F / M / C", "Father, mother or child"),
                            ("Arrow keys", "Select different individuals"),
                            ("Home / End", "Select the primary person, or the last one shown"),
                            ("Page Up / Page Down", "Father's or mother's line (Pedigree)"),
                            ("⌘S", "Switch between the three screens"),
                            ("⌘Home", "Go to the home person"),
                            ("⌘I / ⌘N / ⌘M", "Edit individual, notes, multimedia"),
                            ("⌘A / ⌘U / ⌘L", "Add individual, spouse, child"),
                            ("⌘T / ⌘H", "Add father, mother"),
                            ("⌘F / ⌘D / ⌘R", "Individual list, descendancy list, find by RIN"),
                            ("⌘P", "Print reports"),
                            ("⌘ double-click", "Move someone to the primary position"),
                        ] {
                            ui.label(RichText::new(tr(k)).strong());
                            ui.label(tr(t));
                            ui.end_row();
                        }
                    });
                    if ui.button(tr("OK")).clicked() || escape {
                        open = false;
                    }
                });
            }
            Dialog::Export { id, options, partial } => {
                let mut go = false;
                window("Export", *id).resizable(false).show(ctx, |ui| {
                    ui.label(RichText::new(tr("Export to GEDCOM 5.5 (UTF-8)")).strong());
                    ui.checkbox(&mut options.notes, tr("Notes"));
                    ui.checkbox(&mut options.sources, tr("Sources"));
                    ui.checkbox(&mut options.media, tr("Multimedia links"));
                    ui.checkbox(&mut options.contact, tr("Contact information"));
                    ui.checkbox(&mut options.lds, tr("LDS data"));
                    ui.checkbox(&mut options.full_living, tr("Full information on living"));
                    ui.add_enabled_ui(!options.full_living, |ui| ui.checkbox(&mut options.names_on_living, tr("Names on living")));
                    ui.separator();
                    ui.radio_value(partial, false, tr_n("The {n} individual", "All {n} individuals", self.db.people.len(), &[]));
                    let n = self.focus.as_ref().map(|f| f.len()).unwrap_or(0);
                    ui.add_enabled_ui(n > 0, |ui| ui.radio_value(partial, true, trf("Current focus list ({n})", &[("n", &n)])));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(tr("Export")).clicked() {
                            go = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if go {
                    if let Some(p) = rfd::FileDialog::new().add_filter("GEDCOM", &["ged"]).set_file_name("Family.ged").save_file() {
                        let mut o = options.clone();
                        o.subset = if *partial { self.focus.clone() } else { None };
                        let text = gedcom::export(&self.db, &o, APP_NAME);
                        match std::fs::write(&p, text) {
                            Ok(()) => self.message("Export", trf("Exported to {file}.", &[("file", &p.display())])),
                            Err(e) => self.message("Export", trf("The file could not be saved:\n{error}", &[("error", &e)])),
                        }
                        open = false;
                    }
                }
            }
            Dialog::Properties { id } => {
                window("File Properties", *id).resizable(false).show(ctx, |ui| {
                    egui::Grid::new("props").num_columns(2).show(ui, |ui| {
                        let path = self.path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "Not saved yet".into());
                        let size = self.path.as_ref().and_then(|p| std::fs::metadata(p).ok()).map(|m| format!("{} KB", m.len() / 1024)).unwrap_or_default();
                        let cites: usize = self.db.people.values().map(|p| p.citations.len() + p.birth.citations.len() + p.chr.citations.len() + p.death.citations.len() + p.burial.citations.len()).sum();
                        let media: usize = self.db.people.values().map(|p| p.media.len()).sum();
                        let notes = self.db.people.values().filter(|p| !p.notes.trim().is_empty()).count();
                        for (k, v) in [
                            ("File:", path),
                            ("Size:", size),
                            ("Individuals:", self.db.people.len().to_string()),
                            ("Marriages:", self.db.marriages.len().to_string()),
                            ("Sources:", self.db.sources.len().to_string()),
                            ("Repositories:", self.db.repositories.len().to_string()),
                            ("Citations:", cites.to_string()),
                            ("People with notes:", notes.to_string()),
                            ("Multimedia links:", media.to_string()),
                        ] {
                            ui.label(k);
                            ui.label(v);
                            ui.end_row();
                        }
                    });
                    if ui.button(tr("Close")).clicked() || escape {
                        open = false;
                    }
                });
            }
            Dialog::ConfirmRestore { id, from } => {
                let mut go = false;
                window("Restore", *id).resizable(false).show(ctx, |ui| {
                    ui.label(trf("Open the backup {file} as the current family? Save it with Save As to replace your file.", &[("file", &from.display())]));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Restore")).clicked() {
                            go = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if go {
                    let from = from.clone();
                    open = false;
                    match std::fs::read(&from).map_err(|e| e.to_string()).and_then(|b| serde_json::from_slice::<Db>(&b).map_err(|e| e.to_string())) {
                        Ok(db) => {
                            self.load_db(db, None);
                            self.dirty = true;
                        }
                        Err(e) => self.message("Restore", trf("This backup could not be read:\n{error}", &[("error", &e)])),
                    }
                }
            }
            Dialog::CheckRepair { id, found } => {
                let mut fix = false;
                window("Check/Repair", *id).default_width(520.0).show(ctx, |ui| {
                    if found.is_empty() {
                        ui.label(tr("No problems were found in the links of this file."));
                    } else {
                        ui.label(format!("{} problems were found:", found.len()));
                        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                            for f in found.iter() {
                                ui.label(f);
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        if !found.is_empty() && ui.button(tr("Repair")).clicked() {
                            fix = true;
                        }
                        if ui.button(tr("Close")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if fix {
                    self.db.check(true);
                    self.touch();
                    *found = self.db.clone().check(false);
                }
            }
            Dialog::ConfirmDeleteFile { id, path } => {
                let mut go = false;
                window("Delete File", *id).resizable(false).show(ctx, |ui| {
                    ui.label(trf("Permanently delete {file}? This cannot be undone.", &[("file", &path.display())]));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Delete")).clicked() {
                            go = true;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
                if go {
                    let p = path.clone();
                    open = false;
                    if self.path.as_ref() == Some(&p) {
                        self.load_db(Db::default(), None);
                    }
                    if let Err(e) = std::fs::remove_file(&p) {
                        self.message("Delete File", trf("The file could not be deleted:\n{error}", &[("error", &e)]));
                    }
                    self.recent.retain(|x| *x != p);
                }
            }
            Dialog::Reports { id, tab, start, gens, list_kind, year, include_dead, fields, title, preview, page } => {
                let dialog_id = *id;
                let mut pick = false;
                let mut made: Option<Report> = None;
                let mut failed = None;
                let start_name = self.db.name(*start);
                window("Print Reports", *id).default_width(720.0).default_height(620.0).show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (i, t) in REPORT_TABS.iter().enumerate() {
                            if ui.selectable_label(*tab == i, tr(t)).clicked() {
                                *tab = i;
                                *preview = None;
                            }
                        }
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(trf("Starting person: {name}", &[("name", &start_name)]));
                        if ui.button(tr("Search")).clicked() {
                            pick = true;
                        }
                    });
                    match *tab {
                        0 => {
                            ui.horizontal(|ui| {
                                ui.label(tr("Generations per page:"));
                                ui.add(egui::DragValue::new(gens).range(4..=6));
                            });
                        }
                        2 | 3 | 6 => {
                            ui.horizontal(|ui| {
                                ui.label(tr("Generations:"));
                                ui.add(egui::DragValue::new(gens).range(1..=99));
                            });
                        }
                        5 => {
                            let name = reports::LIST_KINDS[*list_kind];
                            egui::ComboBox::from_label(tr("Type of list")).selected_text(tr(name)).width(280.0).show_ui(ui, |ui| {
                                for (i, k) in reports::LIST_KINDS.iter().enumerate() {
                                    ui.selectable_value(list_kind, i, tr(k));
                                }
                            });
                        }
                        7 => {
                            ui.label(tr("Lists the pictures marked for the scrapbook, with their captions. Pictures themselves are viewed from Edit, Multimedia."));
                        }
                        8 => {
                            ui.horizontal(|ui| {
                                ui.label(tr("Year:"));
                                ui.add(egui::DragValue::new(year).range(1000..=3000));
                                ui.checkbox(include_dead, tr("Include people who have died"));
                            });
                        }
                        9 => {
                            ui.horizontal(|ui| {
                                ui.label(tr("Title:"));
                                ui.add(TextEdit::singleline(title).desired_width(260.0));
                            });
                            ui.horizontal_wrapped(|ui| {
                                for f in reports::CUSTOM_FIELDS {
                                    let mut on = fields.iter().any(|x| x == f);
                                    if ui.checkbox(&mut on, f).changed() {
                                        if on {
                                            fields.push(f.to_string());
                                        } else {
                                            fields.retain(|x| x != f);
                                        }
                                    }
                                }
                            });
                            let who = self.focus.as_ref().map(|f| f.len()).unwrap_or(self.db.people.len());
                            ui.label(tr_n("Prints {n} individual. Use Advanced Focus/Filter to choose fewer.", "Prints {n} individuals. Use Advanced Focus/Filter to choose fewer.", who, &[]));
                        }
                        _ => {}
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(tr("Preview")).clicked() {
                            made = Some(self.build_report(*tab, *start, *gens, *list_kind, *year, *include_dead, fields, title));
                        }
                        let have = preview.is_some();
                        if ui.add_enabled(have, egui::Button::new(tr("Save as PDF..."))).clicked() {
                            if let Some(r) = preview.as_ref() {
                                failed = save_report(r, false).err();
                            }
                        }
                        if ui.add_enabled(have, egui::Button::new(tr("Print..."))).on_hover_text(tr("Opens the report in your PDF viewer, where you can print it")).clicked() {
                            if let Some(r) = preview.as_ref() {
                                failed = save_report(r, true).err();
                            }
                        }
                        if ui.button(tr("Close")).clicked() || escape {
                            open = false;
                        }
                        if *tab <= 1 && ui.button(tr("Blank Form")).on_hover_text(tr("An empty form to fill in by hand")).clicked() {
                            made = Some(if *tab == 0 { reports::blank_pedigree(*gens as usize) } else { reports::blank_family_group() });
                        }
                    });
                    if let Some(r) = preview.as_ref() {
                        ui.separator();
                        let n = r.pages.len();
                        if *page >= n {
                            *page = n.saturating_sub(1);
                        }
                        ui.horizontal(|ui| {
                            if ui.add_enabled(*page > 0, egui::Button::new(tr("Previous Page"))).clicked() {
                                *page -= 1;
                            }
                            ui.label(trf("Page {page} of {pages}", &[("page", &(*page + 1)), ("pages", &n)]));
                            if ui.add_enabled(*page + 1 < n, egui::Button::new(tr("Next Page"))).clicked() {
                                *page += 1;
                            }
                        });
                        egui::ScrollArea::both().auto_shrink(false).max_height(460.0).show(ui, |ui| {
                            if let Some(p) = r.pages.get(*page) {
                                draw_page(ui, p, 1.0);
                            }
                        });
                    }
                });
                if pick {
                    self.push(|id| Dialog::new_picker(id, PickFor::ReportStart(dialog_id)));
                }
                if let Some(r) = made {
                    *preview = Some(r);
                    *page = 0;
                }
                if let Some(e) = failed {
                    self.message("Print Reports", trf("The report could not be saved:\n{error}", &[("error", &e)]));
                }
            }
            Dialog::UnsavedClose { id } => {
                window("Close", *id).resizable(false).show(ctx, |ui| {
                    ui.label(tr("This family has not been saved to a file. Save it before closing?"));
                    ui.horizontal(|ui| {
                        if ui.button(tr("Save As...")).clicked() {
                            self.pending.push(Cmd::SaveAs);
                            open = false;
                        }
                        if ui.button(tr("Close without saving")).clicked() {
                            self.load_db(Db::default(), None);
                            open = false;
                        }
                        if ui.button(tr("Cancel")).clicked() || escape {
                            open = false;
                        }
                    });
                });
            }
            _ => {}
        }
        open
    }

    #[allow(clippy::too_many_arguments)]
    fn build_report(&self, tab: usize, start: u32, gens: u32, list_kind: usize, year: i32, include_dead: bool, fields: &[String], title: &str) -> Report {
        let focus = self.focus.as_deref();
        match tab {
            0 => reports::pedigree(&self.db, start, gens as usize),
            1 => match self.current_marriage(start).or_else(|| self.current_parents(start)) {
                Some(m) => reports::family_group(&self.db, m),
                None => reports::individual_summary(&self.db, start),
            },
            2 => reports::ancestry(&self.db, start, gens as usize),
            3 => reports::descendancy(&self.db, start, gens),
            4 => reports::individual_summary(&self.db, start),
            5 => reports::list_report(&self.db, reports::LIST_KINDS[list_kind], focus),
            6 => reports::book(&self.db, start, gens),
            7 => {
                let mut rows = Vec::new();
                for p in self.db.people.values() {
                    for m in p.media.iter().filter(|m| m.in_scrapbook) {
                        rows.push(format!("{}: {} ({})", plain_name(&p.name), m.caption, m.file));
                    }
                }
                reports::lines_report(&self.db, "Scrapbook", &rows)
            }
            8 => reports::calendar(&self.db, year, include_dead),
            _ => {
                let who: Vec<u32> = match focus {
                    Some(f) => f.to_vec(),
                    None => {
                        let mut v: Vec<u32> = self.db.people.keys().copied().collect();
                        v.sort_by_cached_key(|r| sort_key(&self.db.people[r].name));
                        v
                    }
                };
                reports::custom_report(&self.db, title, fields, &who)
            }
        }
    }
}
